<div align="center">
  
  # LocaGate

  A lightweight, secure, and self-hostable tunnel proxy system to expose local servers to the internet.

  [![GitHub Stars](https://img.shields.io/github/stars/chinhps/LocaGate?style=for-the-badge&color=FEE75C&logo=github)](https://github.com/chinhps/LocaGate/stargazers)
  [![GitHub Forks](https://img.shields.io/github/forks/chinhps/LocaGate?style=for-the-badge&color=5865F2&logo=github)](https://github.com/chinhps/LocaGate/network/members)
  [![Build Status](https://img.shields.io/github/actions/workflow/status/chinhps/LocaGate/build.yml?branch=main&style=for-the-badge&logo=github-actions&logoColor=white&label=build)](https://github.com/chinhps/LocaGate/actions)
  [![Discord](https://img.shields.io/badge/Discord-Join%20Server-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/GqN7Xqpt)
  [![License](https://img.shields.io/github/license/chinhps/LocaGate?style=for-the-badge&color=57F287)](https://github.com/chinhps/LocaGate/blob/main/LICENSE)

  ### [Download Latest Release](https://github.com/chinhps/LocaGate/releases)

  **Windows** • **macOS** • **Linux** • **Cloudflare Workers**
</div>

---

**LocaGate** allows you to expose your local web servers (ports or virtual domains) to the public internet through a Cloudflare Worker relay. It bypasses firewalls and NATs without exposing your private IP address or requiring port forwarding on your router.

Built with **Tauri v2** and **Dioxus v0.6** for a blazing-fast native desktop experience, and **Cloudflare Workers (with Durable Objects & SQLite)** for a highly scalable and cost-effective relay backend.

---

## Features

- **Port Forwarding**: Expose local HTTP servers (e.g. `127.0.0.1:3000`) directly.
- **Virtual Domain Override**: Map custom hostnames (e.g. `shop.local`) to route local vhosts through the tunnel.
- **Request Monitoring**: Real-time traffic log monitoring with status codes, request paths, and execution times.
- **Hidden Dev Settings**: Reveal advanced configuration fields (Worker URL & Token) by clicking the `status: online` footer 3 times.
- **Easy Self-Hosting**: Deploy the relay backend with one click using the Cloudflare deployment button.
- **Automatic Multi-Platform Build**: Compile Windows, macOS, and Linux installers using GitHub Actions.

---

## Tech Stack

- **Desktop App**: 
  - Backend: **Tauri v2** (Rust) for system-level networking and WebSockets.
  - Frontend: **Dioxus v0.6** (Rust WASM) for a modern, reactive UI styled with vanilla CSS.
- **Relay Server**:
  - **Cloudflare Workers** (TypeScript) leveraging **Durable Objects** for persistent WebSocket routing and **SQLite** for database migration.

---

## Getting Started

### Prerequisites

To compile the desktop application locally, you must have the following installed:

1. **Rust Toolchain**: [Install Rust](https://www.rust-lang.org/tools/install)
2. **WASM Target**:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```
3. **Dioxus CLI**:
   ```bash
   cargo install dioxus-cli
   ```
4. **Tauri CLI**:
   ```bash
   cargo install tauri-cli
   ```

---

## Local Development

### 1. Run the Desktop App

To start the application in development mode with hot-reloading:

```bash
cargo tauri dev
```
This command automatically triggers Dioxus CLI (`dx serve`) to compile the WASM frontend and opens the Tauri native window.

### 2. Run the Worker Locally (Optional)

To test the Cloudflare Worker relay locally:

```bash
cd worker
npm install
npx wrangler dev
```

---

## Production Build

To bundle the application into a production-ready installer:

```bash
cargo tauri build
```

The compiled installers will be saved in:
- **Windows**: `target/release/bundle/msi/` and `target/release/bundle/nsis/`
- **macOS**: `target/release/bundle/dmg/`
- **Linux**: `target/release/bundle/deb/` and `target/release/bundle/appimage/`

---

## Cloudflare Worker Relay Setup

To host your own relay server on Cloudflare:

### Method A: One-Click Deploy (Recommended)

Click the button below to fork and deploy the worker repository to your Cloudflare account automatically:

[![Deploy to Cloudflare Workers](https://deploy.workers.cloudflare.com/button.svg)](https://deploy.workers.cloudflare.com/?url=https://github.com/chinhps/LocaGate)

### Method B: Manual Deploy via Wrangler

1. Navigate to the `worker` directory and install dependencies:
   ```bash
   cd worker
   npm install
   ```
2. Deploy the worker to your Cloudflare account:
   ```bash
   npx wrangler deploy
   ```
3. **Configure Authentication**:
   - Go to your Cloudflare Workers Dashboard.
   - Select the `locagate-relay` worker.
   - Go to **Settings > Variables**.
   - Add a new Environment Variable named `AUTH_TOKEN` and assign a secure secret key to protect your relay server from unauthorized usage.

---

## How to Use

1. Launch the **LocaGate** desktop app.
2. Activate **Dev Mode** by clicking the **`status: online`** text in the bottom-left sidebar **3 times**.
3. Go to **Settings** and input:
   - **Cloudflare Worker URL**: Your deployed worker URL (e.g. `https://locagate-relay.yourname.workers.dev`).
   - **Authentication Token**: The `AUTH_TOKEN` you set in the Cloudflare Dashboard.
4. Click **Save Settings**.
5. Return to **Add Tunnel** to register your local ports or virtual domains and click **Start**!

---

## Author & Socials

Created and maintained by **ChinhDev**. Feel free to reach out or connect:

- **Website / Portfolio**: [chính.vn](https://chinh.dev)
- **Instagram**: [@chinh.dev](https://www.instagram.com/chinh.dev)
- **Discord**: `chinhdev` (Join our [Discord Server](https://discord.gg/GqN7Xqpt))
- **GitHub**: [@chinhps](https://github.com/chinhps)
- **Email**: [phamchinh347@gmail.com](mailto:phamchinh347@gmail.com)

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
