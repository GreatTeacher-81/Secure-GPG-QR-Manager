document.addEventListener('DOMContentLoaded', () => {
    const statusElement = document.getElementById('status');
    const publicKeysList = document.getElementById('public-keys-list');
    const secretKeysList = document.getElementById('secret-keys-list');
    const resultOutput = document.getElementById('result-output');
    const qrCodeOutput = document.getElementById('qr-code-output');
    const qrCodeDisplay = document.getElementById('qr-code-display');

    // --- QR Code Scanning Elements ---
    const qrReaderElement = document.getElementById('qr-reader');
    const qrResultElement = document.getElementById('qr-reader-results');
    const startScanBtn = document.getElementById('start-scan-btn');
    const stopScanBtn = document.getElementById('stop-scan-btn');
    const scannedDataDisplay = document.getElementById('scanned-data-display');
    const scannedDataType = document.getElementById('scanned-data-type');
    const importScannedKeyBtn = document.getElementById('import-scanned-key-btn');
    const decryptScannedMsgBtn = document.getElementById('decrypt-scanned-msg-btn');
    const verifyScannedMsgBtn = document.getElementById('verify-scanned-msg-btn');
    let html5QrCode = null; // Store the scanner instance
    let scannedQrData = ''; // Store the latest scanned data


    // Function to update status and key lists
    async function updateStatus() {
        try {
            statusElement.textContent = 'Fetching status...';
            const response = await fetch('/api/status');
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            const data = await response.json();
            if (data.success) {
                statusElement.textContent = 'Ready'; // Update as needed
                publicKeysList.innerHTML = data.public_keys.map(key => `<li>${escapeHtml(key)}</li>`).join('');
                secretKeysList.innerHTML = data.secret_keys.map(key => `<li>${escapeHtml(key)}</li>`).join('');
            } else {
                statusElement.textContent = 'Error loading status';
                console.error('Status API Error:', data.error);
            }
        } catch (error) {
            statusElement.textContent = 'Failed to connect';
            console.error('Error fetching status:', error);
        }
    }

    // Helper to handle API responses
    async function handleApiResponse(response) {
        resultOutput.innerHTML = ''; // Clear previous results
        qrCodeOutput.innerHTML = ''; // Clear previous QR
        qrCodeDisplay.innerHTML = '';

        try {
            if (!response.ok) {
                 // Try to parse error JSON from server
                 let errorMsg = `Request failed with status ${response.status}`;
                 try {
                      const errData = await response.json();
                      if (errData.error) {
                           errorMsg = `Error: ${errData.error}`;
                      }
                 } catch (e) { /* Ignore parsing error */ }
                throw new Error(errorMsg);
            }

            const data = await response.json();

            if (data.success) {
                resultOutput.innerHTML = `<h3>Operation Successful:</h3>`;
                if (data.data) {
                     // Display data safely - escape HTML
                     const dataContent = (typeof data.data === 'string') ? data.data : JSON.stringify(data.data, null, 2);
                     resultOutput.innerHTML += `<pre>${escapeHtml(dataContent)}</pre>`;
                } else {
                     resultOutput.innerHTML += `<p>Completed.</p>`;
                }

                if (data.qr_code) {
                    qrCodeOutput.innerHTML = `<h3>QR Code for Transfer:</h3>`;
                    qrCodeDisplay.innerHTML = data.qr_code; // Assuming SVG string
                    qrCodeOutput.appendChild(qrCodeDisplay);
                    qrCodeOutput.innerHTML += `<p>Scan this QR code with the other device.</p>`;

                }
            } else {
                resultOutput.innerHTML = `<p class="error">Error: ${escapeHtml(data.error || 'Unknown error')}</p>`;
            }
        } catch (error) {
            console.error('API Handling Error:', error);
            resultOutput.innerHTML = `<p class="error">Error: ${escapeHtml(error.message)}</p>`;
        }
    }


    // Generic form submission handler
    function handleFormSubmit(formId, endpoint) {
        const form = document.getElementById(formId);
        if (form) {
            form.addEventListener('submit', async (event) => {
                event.preventDefault();
                statusElement.textContent = 'Processing...';
                resultOutput.innerHTML = '<p>Working...</p>'; // Indicate activity
                qrCodeOutput.innerHTML = '';

                const formData = new FormData(form);
                const response = await fetch(endpoint, {
                    method: 'POST',
                    body: new URLSearchParams(formData) // Standard form encoding
                });
                await handleApiResponse(response);
                statusElement.textContent = 'Ready';
                updateStatus(); // Refresh key lists after potential changes
            });
        } else {
             console.warn(`Form with ID ${formId} not found.`);
        }
    }

    // Setup form handlers
    handleFormSubmit('export-form', '/api/export_key');
    handleFormSubmit('import-form', '/api/import_key');
    handleFormSubmit('encrypt-form', '/api/encrypt');
    handleFormSubmit('decrypt-form', '/api/decrypt');
    handleFormSubmit('sign-form', '/api/sign');
    handleFormSubmit('verify-form', '/api/verify');


    // --- QR Code Scanning Logic ---

    function onScanSuccess(decodedText, decodedResult) {
        // Handle the scanned code string.
        console.log(`Code matched = ${decodedText}`, decodedResult);
        qrResultElement.textContent = `Scan successful! Data length: ${decodedText.length}`;
        scannedQrData = decodedText; // Store the data
        processScannedData(decodedText); // Send to backend for analysis
        stopScanning(); // Stop scanning after success
    }

    function onScanFailure(error) {
        // handle scan failure, usually better to ignore and keep scanning.
        // console.warn(`Code scan error = ${error}`);
        qrResultElement.textContent = `Scanning... (${error})`;
    }

    async function processScannedData(data) {
         scannedDataDisplay.textContent = data; // Show raw data
         scannedDataType.textContent = 'Analyzing...';
         importScannedKeyBtn.style.display = 'none';
         decryptScannedMsgBtn.style.display = 'none';
         verifyScannedMsgBtn.style.display = 'none';

         try {
              const response = await fetch('/api/process_qr_data', {
                   method: 'POST',
                   headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
                   body: new URLSearchParams({ scanned_data: data })
              });
              const result = await response.json();
              if (result.success && result.data) {
                   scannedDataType.textContent = result.data.data_type || 'Unknown';
                   // Show relevant action buttons based on detected type
                   const type = result.data.data_type;
                   if (type === 'PGP Public Key') {
                        importScannedKeyBtn.style.display = 'inline-block';
                   } else if (type === 'PGP Encrypted Message') {
                        decryptScannedMsgBtn.style.display = 'inline-block';
                   } else if (type === 'PGP Signed Message' || type === 'PGP Detached Signature') {
                        // Verification might need more complex UI if detached
                        verifyScannedMsgBtn.style.display = 'inline-block';
                   }
              } else {
                   scannedDataType.textContent = 'Analysis Failed';
                   scannedDataDisplay.textContent = `Error: ${result.error || 'Unknown'}`;
              }
         } catch (error) {
              console.error("Error processing scanned data:", error);
              scannedDataType.textContent = 'Error';
              scannedDataDisplay.textContent = `Failed to process: ${error.message}`;
         }
    }


    function startScanning() {
        if (!html5QrCode) {
             html5QrCode = new Html5Qrcode("qr-reader");
        }
        const config = { fps: 10, qrbox: { width: 250, height: 250 } };
        // If you want to prefer back camera
        html5QrCode.start({ facingMode: "environment" }, config, onScanSuccess, onScanFailure)
            .then(() => {
                startScanBtn.style.display = 'none';
                stopScanBtn.style.display = 'inline-block';
                 qrResultElement.textContent = 'QR Scanner Started. Point camera at QR code.';
            })
            .catch(err => {
                console.error("Failed to start QR scanner:", err);
                qrResultElement.textContent = `ERROR: Could not start scanner - ${err}`;
                html5QrCode = null; // Reset if start failed
            });
    }

    function stopScanning() {
         if (html5QrCode && html5QrCode.isScanning) {
              html5QrCode.stop().then(ignore => {
                   // ignore success
                   console.log("QR Code scanning stopped.");
                    startScanBtn.style.display = 'inline-block';
                    stopScanBtn.style.display = 'none';
                    qrResultElement.textContent = 'Scanner stopped.';
              }).catch(err => {
                   // Stop failed, handle it.
                   console.error("Failed to stop QR scanner:", err);
                    qrResultElement.textContent = 'Failed to stop scanner.';
              });
         }
         // Clean up instance if needed, or keep it for restarting
         // html5QrCode = null;
    }


    startScanBtn.addEventListener('click', startScanning);
    stopScanBtn.addEventListener('click', stopScanning);

     // --- Action button handlers for scanned data ---
     importScannedKeyBtn.addEventListener('click', () => {
         if (scannedQrData) {
              // Populate the import form and submit it
              const importTextArea = document.getElementById('import-key-data');
              const importForm = document.getElementById('import-form');
              if (importTextArea && importForm) {
                   importTextArea.value = scannedQrData;
                   importForm.dispatchEvent(new Event('submit')); // Trigger submission
                   // Optionally clear scanned data display after action
                   // scannedDataDisplay.textContent = '';
                   // scannedDataType.textContent = 'N/A';
                   // importScannedKeyBtn.style.display = 'none';
              }
          } else {
              alert("No scanned data available to import.");
          }
     });

     decryptScannedMsgBtn.addEventListener('click', () => {
          if (scannedQrData) {
               const decryptTextArea = document.getElementById('decrypt-ciphertext');
               const decryptForm = document.getElementById('decrypt-form');
               if (decryptTextArea && decryptForm) {
                    decryptTextArea.value = scannedQrData;
                    decryptForm.dispatchEvent(new Event('submit'));
               }
          } else {
              alert("No scanned data available to decrypt.");
          }
     });

      verifyScannedMsgBtn.addEventListener('click', () => {
          if (scannedQrData) {
               const verifyTextArea = document.getElementById('verify-signed-data');
               const verifyForm = document.getElementById('verify-form');
               if (verifyTextArea && verifyForm) {
                    verifyTextArea.value = scannedQrData;
                    verifyForm.dispatchEvent(new Event('submit'));
               }
          } else {
               alert("No scanned data available to verify.");
          }
     });


    // --- Utility ---
    function escapeHtml(unsafe) {
        if (typeof unsafe !== 'string') return '';
        return unsafe
             .replace(/&/g, "&amp;")
             .replace(/</g, "&lt;")
             .replace(/>/g, "&gt;")
             .replace(/"/g, "&quot;")
             .replace(/'/g, "&#039;");
     }

    // Initial status update
    updateStatus();
});