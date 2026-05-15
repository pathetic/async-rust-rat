# 🚀 Async Rust RAT - Feature Implementation Roadmap

## 🛡️ Persistence & Stealth
- [ ] **Registry Run Key Persistence**: Add a "Startup" toggle to the builder that creates a `Run` key in `HKCU`.
- [ ] **Scheduled Task Persistence**: Implement logic to create a hidden Windows Task that triggers on user login.
- [ ] **WMI Event Persistence**: Use advanced WMI event subscriptions to relaunch the client if it's closed.
- [ ] **Process Injection**: Inject the client DLL/shellcode into a legitimate process like `svchost.exe` or `explorer.exe`.
- [ ] **Anti-VM/Sandbox Checks**: Extend the current anti-VM checks to look for Mac addresses (OUI), specific drivers, and disk size.
- [ ] **Self-Destruct Command**: A "Uninstall" button that removes all files, registry keys, and traces of the client.
- [ ] **Dynamic API Loading**: Instead of static imports, resolve Windows APIs at runtime to evade simple IAT (Import Address Table) scanning.
- [ ] **Obfuscated Communication**: Use XOR or custom encryption for the network traffic to hide common RAT patterns from Firewalls.
- [ ] **Fileless Execution**: A builder option to generate a PowerShell stager that loads the Rust client directly into memory.
- [ ] **Mutex Enforcement**: Prevent multiple instances of the client from running simultaneously on the same machine.

## 🔍 Data Gathering & Post-Exploitation
- [ ] **Browser Credential Recovery**: Extract saved passwords and cookies from Chrome, Edge, and Firefox.
- [ ] **Discord Token Stealer**: Automatically find and upload Discord authentication tokens.
- [ ] **Keylogger**: Implement a low-level keyboard hook to capture keystrokes in real-time.
- [ ] **Clipboard Monitor**: Notify the server whenever the user copies new text or images.
- [ ] **WiFi Password Extractor**: Recover all saved WiFi SSIDs and their plaintext passwords.
- [ ] **Software Inventory**: List all installed software and their versions (useful for identifying vulnerabilities).
- [ ] **Crypto Wallet Searcher**: Scan the filesystem for common wallet files (e.g., `wallet.dat`).
- [ ] **Active Window Tracking**: Log which application the user is currently using and for how long.
- [ ] **Audio Recorder**: Record audio from the default microphone for a specified duration.
- [ ] **Network Map**: Perform an ARP scan of the local network to find other devices on the LAN.

## 🎮 Remote Interaction
- [ ] **Advanced PowerShell Terminal**: A full interactive PowerShell console with tab-completion support.
- [ ] **BSOD Trigger**: A "Nuclear" option to force a Blue Screen of Death using `RtlAdjustPrivilege`.
- [ ] **Desktop Wallpaper Changer**: Remotely set the victim's wallpaper to a custom image (with upload support).
- [ ] **Input Disabler**: Temporarily block the victim's mouse and keyboard during "maintenance."
- [ ] **Text-to-Speech**: Make the computer speak custom text using the Windows SAPI.
- [ ] **Message Box Spammer**: Send multiple custom alerts or error messages to the user.
- [ ] **Remote Script Execution**: Upload and execute Python, VBS, or Batch scripts.
- [ ] **Lock Screen**: Instantly lock the Windows session.
- [ ] **Website Opener**: Force the default browser to open a specific URL (useful for phishing or ads).

## 🌐 Networking & Server Enhancements
- [ ] **Tor Integration**: Route the C2 traffic through Tor to hide the server's IP (Onion routing).
- [ ] **Domain Fronting**: Use CDNs to mask the C2 domain as a legitimate site like `google.com`.
- [ ] **Multi-Server Fallback**: Allow the client to have a list of backup C2 IPs if the primary one is down.
- [ ] **Client Geo-Tagging**: Show the client's city and ISP on the world map automatically.
- [ ] **Desktop Notifications**: The server should send a Windows notification when a "New Client" connects.
- [ ] **Mass Command Execution**: Select multiple clients and send a command (e.g., mass screenshot) to all at once.
- [ ] **Plugin System**: Allow users to write custom `.js` or `.rs` plugins for the Tauri server.
- [ ] **Dark/Light Mode Sync**: Automatically match the UI theme to the operator's system settings.
- [ ] **File Manager "Drag & Drop"**: Support dragging files from the local PC directly into the remote file manager.
- [ ] **Reverse Proxy (SOCKS5)**: Allow the operator to browse the internet using the victim's IP address (see: [notunnel](https://crates.io/crates/notunnel)).