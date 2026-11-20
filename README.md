# 🛡️ Aero Browser

**Aero Browser** is an ultra-fast, lightweight, pure-Rust web browser built from the ground up with **strictly zero dependencies on Chromium, Blink, or WebKit**. It serves as the official default browser for [NovaOS](https://github.com/firoze-hossain/novaos) and is designed with an independently embeddable rendering engine.

---

## 🌟 Highlights & Design Synthesis

Aero Browser harmoniously synthesizes the best interface paradigms and privacy features of the top three modern web browsers:

1. **🦁 Brave (Priority 1 — Uncompromising Privacy & Security)**:
   - Built-in **Brave Shields** ad & cross-site tracker defense engine.
   - Automatic cryptographic **HTTPS Upgrades**.
   - Strict script gating and canvas/audio fingerprinting mitigation.
   - Interactive Lion Shield flyout telemetry panel displaying real-time trackers blocked, bandwidth saved, and per-site toggles.

2. **⚡ Google Chrome (Priority 2 — Ergonomics & Developer Tools)**:
   - Sleek, rounded multi-tab strip with active orange accents, close buttons, and quick `+` tab creation.
   - Unified **Omnibox** address bar with live TLS security padlock and bookmark star toggle.
   - Chrome-inspired **DevTools Inspector** featuring live DOM tree traversal, styles viewer, interactive console, and network logs.

3. **🦊 Mozilla Firefox (Priority 3 — Readability & Customization)**:
   - Distraction-free **Reader Mode** content extractor with custom typography and themes (*Sepia*, *Light*, *Dark*, *Paper*).
   - Fast Bookmarks quick-dial bar with one-click navigation to key web destinations.

---

## 🏗️ Architecture & Package Modularity

Aero Browser is engineered as a clean multi-crate Cargo workspace to allow third-party developers and operating systems to embed the engine independently:

```text
aero-browser/
├── crates/
│   ├── aero-engine/    # Standalone pure-Rust layout, HTML/CSS parser & shields engine
│   │   ├── src/dom.rs          # HTML5 Document Object Model (DOM) tree
│   │   ├── src/html.rs         # Tokenizer and DOM tree builder
│   │   ├── src/css.rs          # CSS3 parser, specificity resolution & user-agent sheet
│   │   ├── src/style.rs        # Style cascade and Computed StyledNode hierarchy
│   │   ├── src/layout.rs       # Box Model, block/inline reflow & margin calculation
│   │   ├── src/font.rs         # Glyph rasterization & 32-bit alpha blending
│   │   ├── src/render.rs       # Display list builder & software rasterizer
│   │   ├── src/shields.rs      # Ad/tracker blocklist, HTTPS upgrade & privacy stats
│   │   ├── src/reader.rs       # Firefox-style reader mode article extraction
│   │   └── src/net.rs          # URL parser and internal protocol dispatch (about:*)
│   │
│   ├── aero-ui/        # Modern browser chrome, tab strip, omnibox & devtools library
│   │   ├── src/chrome.rs       # Tabs, address bar, navigation buttons & bookmark bar
│   │   ├── src/tab.rs          # Multi-tab session manager and navigation history
│   │   ├── src/shields_panel.rs# Interactive Brave Shields drop-down flyout
│   │   ├── src/devtools.rs     # Chrome-style Elements & Console inspector panel
│   │   ├── src/bookmarks.rs    # Quick-dial bookmarks storage
│   │   └── src/browser.rs      # High-level AeroBrowser coordinator
│   │
│   └── aero-app/       # Standalone desktop CLI and test application
│       └── src/main.rs         # Desktop preview & headless rasterization suite
│
└── novaos/             # NovaOS bare-metal ring-3 ELF integration
    ├── browser.rs              # Native NovaOS ring-3 executable
    ├── ffi.rs                  # NovaOS int 0x80 syscall bindings
    └── build.sh                # Static i686 ELF32 compiler targeting BROWSER.ELF
```

---

## 🚀 Quick Start

### Building and Running Host Suite
```bash
# Clone the repository
git clone https://github.com/firoze-hossain/aero-browser.git
cd aero-browser

# Run all unit and integration tests across engine and UI
cargo test --workspace

# Launch the desktop test application
cargo run -p aero-app -- https://novaos.org
```

### Compiling for NovaOS
```bash
# Compiles BROWSER.ELF and AERO.ELF and automatically stages into NovaOS fixtures
./novaos/build.sh
```

---

## 📄 License
Licensed under the [MIT License](LICENSE).
