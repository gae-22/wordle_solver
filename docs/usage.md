# Usage

## Interactive TUI

```bash
cargo run --release
# or
cargo run --release -- interactive
```

Input format:

-   Type a guess, then feedback digits per letter
-   Digits: 2=correct, 1=wrong position, 0=absent
-   Example: `adieu 20100`

Keys:

-   a-z to type, Enter to submit, Backspace/Delete to edit
-   Esc/Tab to switch mode
-   h (help), f (first guess), s (stats), r (reset), q (quit)

## CLI

```bash
# First guess suggestion
cargo run --release -- first-guess

# Solve a target (demo)
cargo run --release -- solve --target=CRANE

# Continue with prior feedback
cargo run --release -- solve --guess ADIEU 20100

# Benchmark
cargo run --release -- benchmark --count=1000
```
