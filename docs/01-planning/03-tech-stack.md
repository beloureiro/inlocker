# InLocker - detailed tech stack

## complete stack (2025/2026)

```
┌─────────────────────────────────────────┐
│         FRONTEND (Interface)            │
│  • React 19.2 + TypeScript 5.8          │
│  • TailwindCSS (styling)                │
│  • shadcn/ui (components)               │
│  • Zustand (state management)           │
└─────────────────────────────────────────┘
                  │
                  │ Tauri IPC
                  ▼
┌─────────────────────────────────────────┐
│         BACKEND (Logic)                 │
│  • Rust 1.91                            │
│  • Tauri 2.9.2                          │
│  • tokio (async runtime)                │
│  • serde (serialization)                │
└─────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│         CORE LIBRARIES                  │
│  • zstd 0.13 (compression)              │
│  • ring (cryptography)                  │
│  • notify (file watcher)                │
│  • chrono (dates)                       │
└─────────────────────────────────────────┘
```

## development tools

### build & package
- **Rust**: rustc 1.91.0
- **Node.js**: v23.11.1 (moving to v24 LTS)
- **pnpm**: 10.19.0
- **Tauri CLI**: 2.9.2
- **Vite**: 7.1.12

### code quality
- **ESLint**: Linting JavaScript/TypeScript
- **Prettier**: Code formatting
- **cargo clippy**: Linting Rust
- **cargo fmt**: Formatting Rust

### testing
- **Vitest**: Unit tests frontend
- **Testing Library**: Component tests
- **cargo test**: Rust tests
- **cargo tarpaulin**: Code coverage

## main dependencies

### frontend (package.json)
```json
{
  "dependencies": {
    "react": "^19.2.0",
    "react-dom": "^19.2.0",
    "@tauri-apps/api": "^2.9.0",
    "@tauri-apps/plugin-opener": "^2.5.2",
    "zustand": "^5.0.8",
    "date-fns": "^4.1.0",
    "lucide-react": "^0.552.0"
  },
  "devDependencies": {
    "@types/node": "^24.9.2",
    "@types/react": "^19.2.2",
    "@types/react-dom": "^19.2.2",
    "@vitejs/plugin-react": "^4.7.0",
    "autoprefixer": "^10.4.21",
    "postcss": "^8.5.6",
    "typescript": "^5.8.3",
    "tailwindcss": "^3.4.18",
    "vite": "^7.1.12"
  }
}
```

### backend (Cargo.toml)
```toml
[dependencies]
tauri = { version = "^2.9", features = [] }
tauri-plugin-opener = "^2.5"
tauri-plugin-notification = "^2.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1.42", features = ["full"] }
zstd = "0.13"
ring = "0.17"
notify = "7.0"
chrono = "0.4"
thiserror = "2.0"
log = "0.4"
env_logger = "0.11"

[build-dependencies]
tauri-build = { version = "^2.5", features = [] }
```

## justification of choices

### 1. tauri 2.9.2
**Why:**
- 50% less memory consumption vs Electron
- Binaries 30x smaller (2-3 MB vs 80+ MB)
- Startup <500ms
- Uses native webview (WebKit on macOS)
- Security by default
- **Latest stable version** (November 2025)

**Alternatives considered:**
- ❌ Electron: Too heavy
- ❌ Flutter: Great for mobile, overkill for desktop
- ❌ Native Swift: Loses portability

### 2. react 19.2 + typescript 5.8
**Why:**
- Mature ecosystem
- TypeScript prevents bugs
- Reusable components
- Large community
- **React 19.2** brings server actions and improved ref handling
- **TypeScript 5.8** provides excellent type safety and developer experience

**Alternatives considered:**
- ⚠️ Svelte: Lighter, but smaller ecosystem
- ⚠️ Vue: Excellent, but less adopted than React

### 3. rust 1.91
**Why:**
- Zero-cost abstractions
- Memory safety without garbage collector
- Native C/C++ performance
- Excellent for file operations
- **Latest stable version**

**No alternatives needed:**
- Rust is the obvious choice for Tauri backend

### 4. zstd 0.13 (compression)
**Why:**
- Up to 2x faster than gzip
- Better compression ratio
- Streaming support
- Modern standard (used by Facebook, Linux kernel)
- **Latest Rust crate version**

**Alternatives considered:**
- ❌ zip/gzip: Slower
- ❌ lz4: Faster but compresses less
- ❌ brotli: Better compression but much slower

### 5. ring (cryptography)
**Why:**
- Audited and secure
- Used by Cloudflare, Google
- Simple API
- AEAD support (AES-GCM)

**Alternatives considered:**
- ⚠️ RustCrypto: Valid, but ring is more battle-tested

### 6. zustand 5.0 (state management)
**Why:**
- Simple and minimalist
- Less boilerplate than Redux
- TypeScript first-class
- <1 KB gzipped
- **Latest version**

**Alternatives considered:**
- ❌ Redux: Too verbose
- ⚠️ Jotai: Excellent, but Zustand is more straightforward

### 7. vite 7.1.12
**Why:**
- Fastest build tool available
- ESM-only distribution (modern)
- Excellent hot-reload performance for development
- Perfect for Tauri + React setup
- **Latest stable version** (2025)

### 8. node.js 23.11.1
**Why:**
- Current stable version with excellent performance
- Compatible with Vite 7.1.12
- Will migrate to Node 24 LTS when available
- Supports all modern ES modules features

## final file structure

```
InLocker/
├── docs/                          # Documentation
│   ├── 01-value-proposition.md
│   ├── 02-architecture.md
│   ├── 03-tech-stack.md
│   └── 04-roadmap.md
│
├── src/                           # Source code
│   ├── main.tsx                   # React entry point
│   ├── App.tsx                    # Main component
│   │
│   ├── ui/                        # Interface components
│   │   ├── components/
│   │   │   ├── FolderSelector.tsx
│   │   │   ├── ScheduleConfig.tsx
│   │   │   ├── Dashboard.tsx
│   │   │   └── BackupHistory.tsx
│   │   └── store/
│   │       └── useBackupStore.ts
│   │
│   ├── core/                      # Core logic
│   │   └── types.ts               # Shared types
│   │
│   └── services/                  # Services/APIs
│       └── tauri.ts               # Tauri API wrapper
│
├── src-tauri/                     # Rust backend
│   ├── src/
│   │   ├── main.rs                # Entry point
│   │   ├── backup/
│   │   │   ├── mod.rs
│   │   │   ├── scheduler.rs
│   │   │   ├── compressor.rs
│   │   │   └── crypto.rs
│   │   ├── commands.rs            # Tauri commands
│   │   └── state.rs               # App state
│   │
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── icons/
│
├── config/                        # Configuration
│   └── .env.example
│
├── tests/                         # Tests
│   ├── unit/
│   └── integration/
│
├── package.json
├── tsconfig.json
├── tailwind.config.js
├── vite.config.ts
└── README.md
```

## system requirements

### for development
- macOS 12.0+ (Monterey)
- Xcode Command Line Tools
- Rust 1.91.0+
- Node.js 23.11.1+ (or v24 LTS)
- pnpm 10.19.0+
- 4 GB RAM minimum

### for end user
- macOS 12.0+
- 100 MB free space
- Any Apple Silicon or Intel processor
