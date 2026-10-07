document.addEventListener('DOMContentLoaded', async () => {
    const statusEl = document.getElementById('connection-status');
    const infoEl = document.getElementById('device-info');
    const qrContainer = document.getElementById('qr-container');
    
    statusEl.innerText = "Ready to connect";
    statusEl.style.color = "#3b82f6";

    try {
        // Fetch the local IP from Rust
        const { invoke } = window.__TAURI__.core;
        const ip = await invoke('get_local_ip');
        
        infoEl.innerText = `Scan QR with phone, or manually connect to ${ip}`;
        qrContainer.style.display = "flex";

        // The exact connection URL the phone expects
        // But since the user is over a tunnel, we'll encode both just to be safe, 
        // or just the IP address as a raw string so the phone can handle it.
        const urlToEncode = ip;

        new QRCode(qrContainer, {
            text: urlToEncode,
            width: 200,
            height: 200,
            colorDark : "#000000",
            colorLight : "#ffffff",
            correctLevel : QRCode.CorrectLevel.H
        });
    } catch (e) {
        console.error("Failed to generate QR code", e);
        infoEl.innerText = "Error: Could not determine local IP. Check network.";
    }
});
