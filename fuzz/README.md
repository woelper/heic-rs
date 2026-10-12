# Fuzzing heic-rs

Three targets, run with [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) on a nightly
toolchain:

| target | what it feeds |
|---|---|
| `decode` | `heic_rs::decode` with `DecodeOptions::default()` |
| `probe` | `heic_rs::probe`, the container without pixels |
| `boxes` | `BoxIter`, and each box's payload walked again |

```sh
cargo install cargo-fuzz
./fuzz/seed.sh                                  # the test fixtures as a starting corpus
cargo +nightly fuzz run -a decode -- -max_total_time=300   # -a: overflow checks on
```

CI runs each target for a fixed time on every push and pull request (`.github/workflows/ci.yml`,
job `fuzz`). A crash there fails the build and uploads the input as an artifact; report it as
`SECURITY.md` describes if it is not yours to fix.

Any input that panics, hangs, or allocates past `DecodeOptions::max_pixels` is a bug.
