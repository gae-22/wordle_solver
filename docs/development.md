# Development

## Build

```bash
cargo build
cargo build --release
```

## Test

```bash
cargo test
cargo test -- --nocapture
```

## Quality

```bash
cargo fmt
cargo clippy -- -D warnings
# Optional
# cargo audit
# cargo outdated
```

## Project structure

See `src/` and `.github/copilot-instructions.md` for architecture details.
