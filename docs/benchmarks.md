# Benchmarks

Run with Criterion:

```bash
cargo bench
# Detailed
cargo bench --bench solver_bench -- --verbose
# Open HTML report (macOS)
open target/criterion/report/index.html
```

Example results (Apple M1 Pro):

-   Entropy calc: ~158 µs
-   Best guess: ~2.45 ms
-   Filtering: ~12.6 µs
-   Full solve: ~8.4 ms
