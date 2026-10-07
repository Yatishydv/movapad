# MovaPad

**Your phone. Your touchpad.**

MovaPad turns your smartphone into a wireless touchpad/mouse for Windows and macOS — across any network.

## Architecture

```
Phone (Flutter) ←── WebRTC DataChannel ──→ Desktop (Tauri + Rust)
                         │
                    Signaling Server
                    (Node.js + WS)
```

## Monorepo Structure

```
apps/
  mobile/              Flutter app (Android + iOS)
  desktop/             Tauri + Rust (Windows + macOS)
  website/             Next.js product website
  signaling-server/    Node.js signaling + auth
packages/
  protocol/            Binary protocol (TypeScript)
  shared-types/        Shared TypeScript types
infrastructure/
  turn/                coturn configuration
  deployment/          Deployment scripts
docs/                  Architecture & protocol docs
```

## Development

### Prerequisites

- **Rust** ≥ 1.75
- **Node.js** ≥ 20
- **Flutter** ≥ 3.0
- **Tauri CLI** (`cargo install tauri-cli`)

### Getting Started

```bash
# Clone
git clone <repo-url>
cd movapad

# Install signaling server dependencies
cd apps/signaling-server && npm install

# Install protocol package
cd packages/protocol && npm install

# Run signaling server (dev)
cd apps/signaling-server && npm run dev

# Run desktop app (dev)
cd apps/desktop && cargo tauri dev

# Run mobile app
cd apps/mobile && flutter run
```

## License

Proprietary — All rights reserved.
