# InLocker - detailed tech stack

## complete stack (2025/2026)

```
┌─────────────────────────────────────────┐
│         FRONTEND (Interface)            │
│  • React 19.2 + TypeScript 5.9          │
│  • TailwindCSS (styling)                │
│  • shadcn/ui (components)               │
│  • Zustand (state management)           │
└─────────────────────────────────────────┘
                  │
                  │ Tauri IPC
                  ▼
┌─────────────────────────────────────────┐
│         BACKEND (Logic)                 │
│  • Rust 1.91+                           │
│  • Tauri 2.8.5                          │
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
- **Rust**: rustc 1.91+ (MSRV)
- **Node.js**: v24 LTS "Krypton"
- **pnpm**: 10.20+
- **Tauri CLI**: 2.8.5+
- **Vite**: 7.0+

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
    "@tauri-apps/api": "^2.8.5",
    "zustand": "^5.0.3",
    "date-fns": "^4.1.0",
    "lucide-react": "^0.468.0"
  },
  "devDependencies": {
    "@types/react": "^19.2.0",
    "@types/react-dom": "^19.2.0",
    "@vitejs/plugin-react": "^4.3.4",
    "typescript": "^5.9.3",
    "tailwindcss": "^3.4.17",
    "vite": "^7.0.0",
    "vitest": "^2.1.8"
  }
}
```

### backend (Cargo.toml)
```toml
[dependencies]
tauri = { version = "2.8", features = ["notification"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.42", features = ["full"] }
zstd = "0.13"
ring = "0.17"
notify = "7.0"
chrono = "0.4"
thiserror = "2.0"
log = "0.4"
env_logger = "0.11"

[build-dependencies]
tauri-build = "2.0"
```

## justification of choices

### 1. tauri 2.8.5
**Why:**
- 50% less memory consumption vs Electron
- Binaries 30x smaller (2-3 MB vs 80+ MB)
- Startup <500ms
- Uses native webview (WebKit on macOS)
- Security by default
- **Latest stable version** (September 2025)

**Alternatives considered:**
- ❌ Electron: Too heavy
- ❌ Flutter: Great for mobile, overkill for desktop
- ❌ Native Swift: Loses portability

### 2. react 19.2 + typescript 5.9
**Why:**
- Mature ecosystem
- TypeScript prevents bugs
- Reusable components
- Large community
- **React 19.2** brings server actions and improved ref handling
- **TypeScript 5.9** has better performance and hover tooltips

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

### 7. vite 7.0
**Why:**
- Fastest build tool available
- ESM-only distribution (modern)
- Requires Node.js 20.19+, 22.12+ (we use Node 24 LTS)
- Perfect for Tauri + React setup
- **Latest major version** (2025)

### 8. node.js 24 lts "krypton"
**Why:**
- Long Term Support until April 2028
- Latest LTS version (October 2025)
- Best performance improvements
- Required for Vite 7.0

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
- Rust 1.91+
- Node.js 24 LTS+
- 4 GB RAM minimum

### for end user
- macOS 12.0+
- 100 MB free space
- Any Apple Silicon or Intel processor
