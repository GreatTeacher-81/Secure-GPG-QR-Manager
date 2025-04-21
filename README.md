# Secure GPG Manager (Rust Edition)

[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](http://unlicense.org/)

**A secure GPG management tool in Rust with a local web UI and QR code support for offline data transfer. Uses GPGME for reliable GnuPG interaction.**

This project provides robust GPG functionalities (encryption, decryption, signing, verification, key management) through a local web interface. It leverages the security benefits of Rust (memory safety) and the official GPGME library for reliable interaction with your existing GnuPG installation.

A primary goal is to facilitate secure data exchange, particularly with air-gapped or offline machines, by generating QR codes for GPG data (keys, messages) and providing a mechanism to process data scanned from QR codes via the client's browser camera.

**This project replaces previous script-based approaches that involved insecure custom cryptography (like manual OpenSSL passphrase handling), opting instead for direct and secure interaction with GnuPG via GPGME.**

## Core Features

*   **Secure GPG Operations:** Directly uses `gpgme` (GPG Made Easy library) to interact with the system's GnuPG installation for:
    *   Encryption (to specified recipients)
    *   Decryption (using available secret keys)
    *   Signing (Clearsign, Detached)
    *   Verification (Clearsign, Detached)
    *   Key Listing (Public & Secret)
    *   Key Import (Pasted or from file)
    *   Key Export (Public & Secret - *Use secret key export with caution*)
*   **Local Web Interface:** Runs a local web server (`axum`) providing a user interface accessible only from the machine running the application (or the local network if bound differently).
    *   Designed for interacting with the GPG functionalities without complex command-line usage.
    *   **Intended for local use, primarily for interacting with the host machine's GPG setup.**
*   **QR Code Generation:** Generates SVG QR codes for:
    *   Exported Public Keys
    *   Encrypted Messages (ASCII armored)
    *   Signed Messages (Clearsign/Detached ASCII armored)
    *   Facilitates transferring data to offline devices visually.
*   **QR Code Data Processing (Client-Side Scan):**
    *   The web interface includes a client-side QR code scanner (using the browser's camera via JavaScript - `html5-qrcode` library).
    *   Scanned data is sent back to the server for identification (Key, Message, Signature?).
    *   Allows initiating actions like "Import Key" or "Decrypt Message" based on scanned data.
*   **Memory Safety:** Built with Rust, significantly reducing the risk of memory corruption vulnerabilities common in C/C++.
*   **Secure Defaults:** The web server binds to `localhost` by default, and uses random high ports to avoid common scan ranges.

## Security Considerations

*   **GPGME Reliance:** This tool relies on your existing, correctly configured GnuPG installation and the `gpgme` library. The security of the underlying GPG operations depends on GnuPG itself.
*   **Passphrase Handling:** Securely handling GPG passphrases programmatically is complex. This implementation currently relies on `gpg-agent` or unprotected keys for operations requiring passphrases (decryption, signing, secret key export). **Do not run this on a server where `gpg-agent` might expose passphrases unintentionally.** Future improvements may involve more robust passphrase callbacks.
*   **Web Server Security:** The web server (`--web` mode) is intended for **local use only**. Binding it to non-localhost addresses (`--bind 0.0.0.0`) exposes it to your network and carries significant security risks if the network is not trusted. **No TLS/HTTPS is implemented by default.**
*   **QR Code Security:** While QR codes facilitate offline transfer, be mindful of "shoulder surfing" when displaying QR codes containing sensitive data. Ensure privacy when scanning QR codes.
*   **Hardware Vulnerabilities:** This software cannot protect against compromised hardware (e.g., backdoored CPUs, RAM exploits like Rowhammer). Use trusted hardware for sensitive operations.
*   **Client-Side JavaScript:** QR code scanning happens in the user's browser. Ensure you trust the JavaScript library used (`html5-qrcode` in this example).
*   **No Custom Crypto:** Unlike potential predecessor scripts, this tool **does not** implement its own cryptographic primitives for passphrase storage or encryption, avoiding common pitfalls and relying solely on GPG/GPGME.

## Prerequisites

*   **Rust:** Install Rust and Cargo (via `rustup`: [https://rustup.rs/](https://rustup.rs/)).
*   **GnuPG:** A working GnuPG installation (`gpg`, `gpg-agent`).
*   **GPGME Development Libraries:** You need the `gpgme` library and its development headers. Installation varies by OS:
    *   **Debian/Ubuntu:** `sudo apt-get update && sudo apt-get install libgpgme-dev`
    *   **Fedora/CentOS/RHEL:** `sudo dnf install gpgme-devel`
    *   **macOS (Homebrew):** `brew install gpgme`
*   **OpenSSL Development Libraries:** Required by some dependencies.
    *   **Debian/Ubuntu:** `sudo apt-get install libssl-dev`
    *   **Fedora/CentOS/RHEL:** `sudo dnf install openssl-devel`
    *   **macOS:** Usually handled by Homebrew or system libraries.

## Building

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/GreatTeacher-81/Secure-GPG-QR-Manager secure-gpg
    cd secure-gpg
    ```
2.  **Build the project:**
    *   **Debug build:** `cargo build`
    *   **Release build (recommended for use):** `cargo build --release`**
## Note  
1. This All Coded by Gemini
2. The Idea From Me Not From Gemini
3. This Tool Doesnt Recoded GPG in rust but Used GPG

The executable will be located at `target/debug/secure_gpg_qr` or `target/release/secure_gpg_qr`.

## Running (Web Interface Mode)

The primary way to use this tool is via the web interface:

```bash
# Run with default settings (localhost, random high port, default GPG home)
./target/release/secure_gpg_qr web

# Run on a specific port
./target/release/secure_gpg_qr web --port 8080

# Run binding to a different IP (Use with extreme caution!)
# ./target/release/secure_gpg_qr web --bind 0.0.0.0 --port 8080

# Specify a custom GPG home directory
./target/release/secure_gpg_qr web --gpg-dir /path/to/my/gpg/home
