use crate::error::{AppError, Result};
use gpgme::{Context, Data, Key, Protocol};
use std::io::{Read, Write};
use std::path::PathBuf;
use zeroize::Zeroizing; // Import the trait

// --- Configuration ---
lazy_static::lazy_static! {
    // Global GPGME context (consider thread safety if used across web reqs)
    // Using a Mutex might be safer for web server use cases.
    static ref GPG_CTX: parking_lot::Mutex<Result<Context>> = parking_lot::Mutex::new(init_gpg_context(None));
}

pub fn set_gpg_homedir(homedir: Option<String>) -> Result<()> {
    let mut ctx_guard = GPG_CTX.lock();
    *ctx_guard = init_gpg_context(homedir);
    // Check if initialization was successful
     match *ctx_guard {
         Ok(_) => Ok(()),
         Err(ref e) => Err(AppError::Config(format!("Failed to re-initialize GPG context: {}", e))),
     }
}

fn init_gpg_context(homedir: Option<String>) -> Result<Context> {
    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
    if let Some(dir) = homedir {
        // Ensure the directory exists? gpgme might handle this.
        // Consider canonicalizing the path
        let path = PathBuf::from(dir);
        if !path.exists() {
             // Optionally create it, or error out
             return Err(AppError::Config(format!("GPG home directory not found: {}", path.display())));
        }
         println!("Setting GPG home directory: {}", path.display()); // Logging
        ctx.set_engine_home_dir(path)?;
    }
    ctx.set_armor(true); // Default to ASCII armor for most operations
    Ok(ctx)
}

fn with_gpg_ctx<F, R>(f: F) -> Result<R>
where
    F: FnOnce(&mut Context) -> Result<R>,
{
    let mut guard = GPG_CTX.lock();
    match *guard {
        Ok(ref mut ctx) => f(ctx),
        Err(ref e) => Err(AppError::Config(format!("GPG Context not initialized: {}", e))), // Clone or format the error
    }
}

// --- Key Management ---

pub fn list_keys(secret_only: bool) -> Result<Vec<String>> {
    with_gpg_ctx(|ctx| {
        let mode = if secret_only {
            gpgme::KeyListMode::SECRET
        } else {
            gpgme::KeyListMode::empty() // Public keys by default
        };
        let mut keys = Vec::new();
        for key_result in ctx.keys_with_mode(mode)? {
            let key = key_result?;
            // Format key information as needed
            let uid = key
                .user_ids()
                .next()
                .map(|uid| uid.id().unwrap_or("<no uid>"))
                .unwrap_or("<no user id>");
            let fpr = key.fingerprint().unwrap_or("<no fpr>");
            keys.push(format!("{} {}", fpr, uid));
        }
        Ok(keys)
    })
}

pub fn export_key(key_id: &str, secret: bool) -> Result<String> {
    with_gpg_ctx(|ctx| {
        let mut output = Vec::new();
        let mut armored_output = Data::from_armor_writer(&mut output)?;

        if secret {
             // Need passphrase handling here if key is protected
             println!("Warning: Exporting secret keys requires careful handling.");
            ctx.export_secret_keys_writer(&[key_id], &mut armored_output)?;
        } else {
            ctx.export_keys_writer(&[key_id], &mut armored_output)?;
        }
        drop(armored_output); // Ensure data is flushed

        String::from_utf8(output).map_err(|e| AppError::Operation(format!("UTF8 Error: {}", e)))
    })
}

pub fn import_key(key_data: &str) -> Result<String> {
     with_gpg_ctx(|ctx| {
         let mut input_data = Data::from_bytes(key_data.as_bytes())?;
         let import_result = ctx.import(&mut input_data)?;
         // Provide more detailed result information
         Ok(format!(
             "Keys imported: {}, Unchanged: {}, New User IDs: {}, New Subkeys: {}",
             import_result.imported(),
             import_result.unchanged(),
             import_result.new_user_ids(),
             import_result.new_sub_keys()
         ))
     })
}

// --- Crypto Operations ---

pub fn encrypt(
    plaintext: &str,
    recipients: &[&str],
    // Add signer options if needed
) -> Result<String> {
    if recipients.is_empty() {
        return Err(AppError::InvalidInput("No recipients specified for encryption.".to_string()));
    }
    with_gpg_ctx(|ctx| {
        // Find recipient keys
        let keys: Vec<Key> = recipients
            .iter()
            .map(|r| ctx.find_keys([r]).map_err(AppError::from))
            .collect::<Result<Vec<_>>>()? // Collect results, propagate first error
            .into_iter()
            .flatten() // Flatten Vec<KeyIter> into KeyIter
            .collect::<std::result::Result<Vec<_>,_>>() // Collect keys, propagate gpgme error
            .map_err(AppError::from)?;

        if keys.len() < recipients.len() {
             // This check might be too simple if UserIDs match multiple keys
             println!("Warning: Not all recipient identifiers matched a key.");
             // Consider failing here if strict matching is required
             if keys.is_empty() {
                  return Err(AppError::InvalidInput("None of the specified recipients could be found.".to_string()));
             }
        }


        let mut input_data = Data::from_bytes(plaintext.as_bytes())?;
        let mut output = Vec::new();
        // Ensure output is ASCII armored
        ctx.set_armor(true);
        ctx.encrypt(&keys, gpgme::EncryptFlags::empty(), &mut input_data, Data::from_armor_writer(&mut output)?)?;


        String::from_utf8(output).map_err(|e| AppError::Operation(format!("UTF8 Error: {}", e)))
    })
}

pub fn decrypt(ciphertext: &str) -> Result<String> {
     with_gpg_ctx(|ctx| {
         // Passphrase handling needed if private key is protected
         // ctx.set_passphrase_cb(...); // More complex setup needed

         let mut input_data = Data::from_bytes(ciphertext.as_bytes())?;
         let mut output = Vec::new();

         // Decrypt usually requires passphrase callback setup for protected keys
         // For now, assume unprotected or agent handles it
         ctx.decrypt(&mut input_data, Data::from_writer(&mut output)?)?;

         String::from_utf8(output).map_err(|e| AppError::Operation(format!("UTF8 Error: {}", e)))
     })
}

pub fn sign(
    plaintext: &str,
    signer_key_id: &str,
    mode: gpgme::SignMode,
    passphrase: Option<Zeroizing<String>>,
) -> Result<String> {
     with_gpg_ctx(|ctx| {
         // --- Passphrase Handling ---
         // This is tricky with gpgme. It often relies on pinentry or an agent.
         // For programmatic passphrase:
         if let Some(pass) = passphrase {
             // Register a callback or try to set it directly if the context allows
             // This part needs careful implementation based on gpgme specifics and security needs.
             // Placeholder: Assuming agent handles it or key is unprotected for now.
             println!("Warning: Programmatic passphrase handling for signing is complex and potentially insecure. Relying on agent or unprotected key.");
             // DO NOT use set_passphrase directly if possible, it's often insecure.
             // The callback mechanism is preferred but requires more setup.
         }
         // --- End Passphrase Handling ---

         let mut input_data = Data::from_bytes(plaintext.as_bytes())?;
         let mut output = Vec::new();
         ctx.set_armor(true); // Ensure output is armored for clearsign/detached

         // Ensure the signing key is set if needed (though sign_ext should use default-key if set)
         // Find the key to be sure it exists?
         let keys_result = ctx.find_secret_keys([signer_key_id]);
         match keys_result {
              Ok(mut iter) => {
                  if iter.next().is_none() {
                       return Err(AppError::InvalidInput(format!("Signer secret key '{}' not found.", signer_key_id)));
                  }
              }
              Err(e) => return Err(AppError::GpgME(e)),
         }

         // Use sign_ext for more control, including specifying the signer
         ctx.sign_ext(mode, &mut input_data, Data::from_armor_writer(&mut output)?, &[signer_key_id])?;


         String::from_utf8(output).map_err(|e| AppError::Operation(format!("UTF8 Error: {}", e)))
     })
}

pub fn verify(signed_data: &str) -> Result<String> {
    with_gpg_ctx(|ctx| {
        let mut input_data = Data::from_bytes(signed_data.as_bytes())?;
        let mut plaintext_output: Option<Data> = None; // We don't capture plaintext here, just verify

        // Verify the signature
        let verification_result = ctx.verify(&mut input_data, plaintext_output.as_mut(), None)?; // Pass None for plaintext_output

        // Check signatures
        let mut summary = String::new();
        if let Some(signature) = verification_result.signatures().next() {
             // Check signature status
            if signature.status().is_ok() {
                 let fpr = signature.fingerprint().unwrap_or("<no fpr>");
                 summary.push_str(&format!("Signature valid. Key Fingerprint: {}\n", fpr));
                 // You can add more details like signer user ID if needed
                 summary.push_str(&format!("Summary: {:?}\n", signature.summary()));
            } else {
                 summary.push_str(&format!("Signature invalid or untrusted. Status: {:?}\n", signature.status()));
                 summary.push_str(&format!("Summary: {:?}\n", signature.summary()));
                 return Err(AppError::Operation(summary)); // Return error on bad signature
            }
        } else {
             summary.push_str("No signature found in the provided data.");
             return Err(AppError::InvalidInput(summary));
        }

        Ok(summary) // Return summary string on success
    })
}

// Add functions for generate_key, delete_key etc. following similar patterns
// Remember to handle passphrases securely for key generation/deletion.