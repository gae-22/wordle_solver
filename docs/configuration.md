# Configuration

Environment variables (examples):

```bash
export WORDLE_WORDLIST=/path/to/custom_words.txt
export RUST_LOG=debug
export WORDLE_CACHE_SIZE=10000
export WORDLE_THREADS=8
```

User config: `~/.config/wordle_solver/config.toml`

```toml
[solver]
strategy = "entropy"
max_candidates = 50
cache_size = 10000

[ui]
theme = "dark"
animations = true
compact_mode = false

[performance]
threads = 0
simd = true
parallel_threshold = 100
```
