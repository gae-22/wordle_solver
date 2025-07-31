# Wordle Rust AI Solver

🎯 A high-performance Wordle AI solver built in Rust with entropy-based algorithms and an interactive TUI interface. This application uses information theory to find optimal guesses and solve Wordle puzzles efficiently with real-time candidate analysis.

## ✨ Features

-   🧠 **Intelligent AI Solver**: Uses entropy maximization to find optimal guesses
-   🖥️ **Interactive TUI**: Beautiful terminal interface with real-time feedback and candidate analysis
-   📈 **Real-time Candidate Display**: Shows top word candidates ranked by entropy during gameplay
-   ⚡ **High Performance**: Optimized Rust implementation with sub-second response times
-   📊 **Comprehensive Statistics**: Track remaining possibilities, entropy scores, and solver performance
-   🎯 **Multiple Modes**: Interactive TUI, command-line solver, and first-guess utility
-   🔧 **Robust Testing**: Unit tests, integration tests, and property-based testing
-   📈 **Performance Benchmarks**: Criterion-based benchmarking with HTML reports
-   🎨 **Modern UI**: Color-coded feedback with intuitive visual design

## Installation

### Prerequisites

-   Rust (latest stable version)
-   Cargo package manager

### Building from Source

```bash
git clone https://github.com/gae-22/wordle-solver.git
cd wordle_rust
cargo build --release
```

## Usage

### Interactive TUI Mode

Launch the interactive terminal interface:

```bash
cargo run --release -- interactive
# or simply
cargo run --release
```

**TUI Controls:**

-   Type your guess results in the format: `word result`
-   Press **Enter** to submit
-   Press **'q'** to quit
-   Press **'r'** to restart (when game is finished)
-   **Backspace** to edit input

**TUI Interface Layout:**

The interface displays:

1. **Header**: Application title and subtitle
2. **Wordle Board**: Visual representation of your guesses with color-coded feedback
3. **Current Suggestion**: AI's recommended next word
4. **Input Area**: Where you enter your guess results
5. **Top Candidates Panel**: Real-time list of best words ranked by entropy (NEW!)
6. **Statistics Panel**: Remaining possible words and game metrics
7. **Game Info**: Current game state and progress
8. **Guess History**: Complete history of your guesses and results```

## 📝 Input Format

The solver accepts feedback in the following format:

```
<word> <result_code>
```

**Result Code System:**

| Code | Meaning    | Wordle Color | Description                         |
| ---- | ---------- | ------------ | ----------------------------------- |
| `2`  | **HIT**    | 🟩 Green     | Letter is in the correct position   |
| `1`  | **BITE**   | 🟨 Yellow    | Letter exists but in wrong position |
| `0`  | **ABSENT** | ⬜ Gray      | Letter is not in the word           |

**Input Examples:**

```bash
adieu 20100  # 'a' correct, 'd' absent, 'i' wrong position, 'e' absent, 'u' absent
crane 22222  # All letters correct (puzzle solved!)
stare 01120  # 's' absent, 't' wrong position, 'a' wrong position, 'r' correct, 'e' absent
```

## 🧠 Algorithm & Strategy

The solver implements an advanced **entropy-based strategy** using information theory:

### Core Algorithm

1. **Entropy Maximization**: Each guess is selected to maximize information gain
2. **Constraint Propagation**: Efficiently filters possible words using feedback patterns
3. **Real-time Analysis**: Continuously ranks all candidates by their entropy scores
4. **Optimal Play**: Typically solves puzzles in 3-4 guesses with >95% success rate

### Key Components

| Component             | Description                                            |
| --------------------- | ------------------------------------------------------ |
| **WordleSolver**      | Main solver implementing entropy-based strategy        |
| **EntropyCalculator** | Computes information-theoretic metrics with caching    |
| **Feedback System**   | Processes and validates guess results                  |
| **Word Filtering**    | Efficiently narrows down possibilities                 |
| **TUI Interface**     | Real-time interactive terminal with candidate analysis |

## Project Structure

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Library root
├── app/                 # TUI application
│   ├── mod.rs           # Main app logic
│   ├── ui.rs            # User interface components
│   ├── events.rs        # Event handling
│   └── state.rs         # Application state
├── game/                # Wordle game logic
│   ├── mod.rs           # Game module
│   ├── wordle.rs        # Core game mechanics
│   ├── word_list.rs     # Legacy word list management
│   └── word_list_new.rs # New word list management (active)
├── solver/              # AI solver implementation
│   ├── mod.rs           # Solver module
│   ├── strategy.rs      # Main solver logic
│   ├── entropy.rs       # Entropy calculations
│   └── feedback.rs      # Feedback processing
└── utils/               # Utility functions
    ├── mod.rs           # Utils module
    └── helpers.rs       # Helper functions
```

## 🛠️ Development

### Prerequisites

-   **Rust** (latest stable version) - [Install Rust](https://rustup.rs/)
-   **Cargo** (comes with Rust)

### Quick Start

```bash
# Clone the repository
git clone https://github.com/gae-22/wordle-solver.git
cd wordle-solver

# Build and run
cargo run --release

# Or build first, then run
cargo build --release
./target/release/wordle_solver
```

### Benchmarks

```bash
# Run performance benchmarks
cargo bench

# Generate HTML report (opens in browser)
cargo bench --bench solver_bench

# View benchmark results
open target/criterion/report/index.html
```

### Code Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check
```

### Linting

```bash
# Run clippy linter
cargo clippy

# Fix auto-fixable issues
cargo clippy --fix
```

## 🤝 Contributing

We welcome contributions! Here's how to get started:

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/amazing-feature`
3. **Commit** your changes: `git commit -m 'Add amazing feature'`
4. **Push** to the branch: `git push origin feature/amazing-feature`
5. **Open** a Pull Request

### Development Guidelines

-   Follow Rust's official style guide (`cargo fmt`)
-   Run `cargo clippy` for linting
-   Add tests for new features
-   Update documentation as needed

## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

-   **Josh Wardle** - Creator of the original Wordle game
-   **Information Theory** - Inspiration for the entropy-based algorithm
-   **ratatui** - Excellent TUI framework for Rust
-   **Rust Community** - For the amazing ecosystem and tools

---

<div align="center">

**⭐ Star this repository if you found it helpful!**

[Report Bug](https://github.com/your-username/wordle_rust/issues) · [Request Feature](https://github.com/your-username/wordle_rust/issues) · [Documentation](https://github.com/your-username/wordle_rust/wiki)

</div>
