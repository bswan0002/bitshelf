# Scale measurements

Reproduce with `cargo build --release --locked && python3 scripts/benchmark.py`.
The script creates temporary 1,000- and 10,000-bit stores with 2 KiB bodies and
nested frontmatter, runs three samples, reports median wall time and maximum
per-process resident memory using wait4, and imports 100 additional bits
sequentially at each scale. Counts are initial corpus sizes; small test additions
remain during the run. Prune scans the full store and deletes ten expired bits
per sample. Output is discarded, but JSON encoding is included in measured time.

Local evidence: macOS 15.7.4 ARM64, Rust 1.98.1, optimized bs 0.1.0, local APFS.
Results are environment-specific measurements, not latency guarantees.

| Operation | 1,000 bits (ms) | 10,000 bits (ms) | 10,000-bit peak MiB |
| --- | ---: | ---: | ---: |
| List JSON | 43.27 | 411.27 | 100.67 |
| Search JSON | 45.00 | 438.99 | 112.02 |
| Add | 14.72 | 17.78 | 4.06 |
| Edit | 14.08 | 14.88 | 4.11 |
| Move | 18.70 | 18.91 | 4.05 |
| Prune | 87.10 | 442.68 | 70.56 |
| 100 sequential imports | 1389.61 | 1372.19 | 4.12 |

Raw results: [measurement JSON](benchmarks/0.1.0-macos-arm64.json).

Prune originally read shelf configuration again for every bit. Caching it for
initial selection reduced the observed 10,000-bit prune median from 506 to
443 ms; eligible removals still re-read retention and check original bytes.
Other discovery loads each shelf configuration once and reads each bit once.
Writers no longer scan history or rewrite a global state map, so sequential
imports are essentially independent of existing store size.

The first release targets personal stores around 10,000 notes of this size.
List/search intentionally retain metadata, bodies and original bytes in memory,
then encode JSON; peak memory grows with corpus size. A streaming implementation
could reduce duplication at much larger scales, but these measurements do not
justify an index, daemon, database or concurrency machinery.
