# 🎯 Wordle AI Solver (Rust)

[![Rust](https://img.shields.io/badge/Rust-stable-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Fast, modern Wordle solver with a beautiful TUI and an entropy-based engine.

Quick links: [Quick Start](#quick-start) • [Usage](#usage) • [Docs](#documentation) • [Contributing](#contributing) • [License](#license)

---

## ✨ Highlights

-   🖥️ Clean, responsive TUI (ratatui)
-   🧠 Entropy-based next-guess suggestions
-   ⚡ Sub-second ranking on typical lists
-   🧩 Clean Architecture, well-tested core

---

## 🚀 Quick Start

Prerequisites

-   Rust (stable) — install via https://rustup.rs/

Install and run

```bash
git clone https://github.com/gae-22/wordle_solver.git
cd wordle_solver
cargo run --release
```

---

## 🎮 Usage

Interactive (TUI)

```bash
cargo run --release
# or
cargo run --release -- interactive
```

CLI examples

```bash
# Get first guess
cargo run --release -- first-guess

# Solve with a fixed target (demo)
cargo run --release -- solve --target=CRANE

# Provide previous feedback
cargo run --release -- solve --guess ADIEU 20100
```

Input format

| Digit | Meaning        | Color |
| ----: | -------------- | :---: |
|     2 | Correct spot   |  🟩   |
|     1 | Wrong position |  🟨   |
|     0 | Absent         |  ⬜   |

### ⌨️ TUI key bindings

| Key               | Action                              |
| ----------------- | ----------------------------------- |
| a–z               | Type letters                        |
| 0–2               | Enter feedback                      |
| Enter             | Submit                              |
| Backspace/Del     | Delete/Clear                        |
| Esc/Tab           | Switch mode                         |
| h / f / s / r / q | Help / First / Stats / Reset / Quit |

---

## 📚 Documentation

Detailed guides live in the docs/ directory:

-   Features and TUI: [docs/features.md](docs/features.md)
-   Command-line and input details: [docs/usage.md](docs/usage.md)
-   Algorithm overview: [docs/algorithm.md](docs/algorithm.md)
-   Word lists and cache: [docs/wordlists.md](docs/wordlists.md)
-   Configuration: [docs/configuration.md](docs/configuration.md)
-   Development and testing: [docs/development.md](docs/development.md)
-   Benchmarks: [docs/benchmarks.md](docs/benchmarks.md)
-   Contributing: [docs/contributing.md](docs/contributing.md)

Architecture notes: [.github/copilot-instructions.md](.github/copilot-instructions.md)

---

## 📦 Word lists (short)

-   Binary cache `word_lists.wlf` (WLF1) in project root for fast load
-   Optional `word_lists.json` for inspection
-   Keeps only 5-letter lowercase words; removes duplicates

Customize sources with `word_sources.json` (project root). Default source: [dwyl/english-words](https://github.com/dwyl/english-words).

---

## 🔧 Quick configuration

Environment variables (examples):

```bash
export RUST_LOG=info
export WORDLE_CACHE_SIZE=10000
```

User config `~/.config/wordle_solver/config.toml` (excerpt):

```toml
[solver]
strategy = "entropy"
max_candidates = 50
```

---

## 🤝 Contributing

See [docs/contributing.md](docs/contributing.md). PRs welcome.

---

## 📄 License

MIT — see [LICENSE](LICENSE).

---

## 📊 Benchmarks (quick)

```bash
cargo bench
# Open HTML report (macOS)
open target/criterion/report/index.html
```

More details: [docs/benchmarks.md](docs/benchmarks.md)

---

Made with 🦀 Rust
