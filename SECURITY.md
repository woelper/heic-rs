# Security Policy

## Supported versions

Security fixes land on the latest published 0.1.x release; older minor versions are not backported.

| Version | Supported |
|---|---|
| 0.1.4 and later 0.1.x | yes |
| 0.1.0 – 0.1.3 | no -- upgrade to 0.1.4 (panics on malformed boxes and HEVC streams, and an item-data amplification, fixed in 0.1.2 to 0.1.4) |

## Reporting a vulnerability

Please report privately, not in a public issue.

- Preferred: open a private security advisory at
  <https://github.com/tbraun96/heic-rs/security/advisories/new>.
- Alternatively, email tbraun96@gmail.com.

Include the input file if you can share it, or a generator for it if you cannot, plus the crate
version or commit, the `DecodeOptions` in use, and what you observed (panic, hang, allocation
blow-up, wrong pixels). A minimised reproducer is worth more than a description.

Expect an acknowledgement within a few days. Fixes are developed in a private fork and disclosed
alongside the patched release, with credit to the reporter unless you prefer otherwise.

## Threat model

**HEIF files are untrusted input.** That is the whole premise: this crate exists to be pointed at
files that arrived over a network, from a phone, or from a user upload, and it is written on the
assumption that any of them may be hostile or simply corrupt.

What that assumption buys:

- **`#![forbid(unsafe_code)]`** at the crate root. There is no `unsafe` block to audit, and the
  compiler enforces that. A parser bug therefore degrades into a panic or wrong output, not a
  memory-safety vulnerability: no out-of-bounds read, no use-after-free, no type confusion.
- **No `unwrap`, `expect`, or `panic!` in library code.** Malformed input returns an `Error`. A panic
  reachable from library code with any byte sequence is a bug worth reporting, even without a
  memory-safety consequence, because callers decoding untrusted uploads should not have to catch
  unwinds.
- **Every parser is bounds-checked.** Reads go through reader primitives that return
  `Error::Truncated` when the input ends early and `Error::Malformed` when a structure is internally
  inconsistent (a box that claims to extend past its parent, an `iloc` extent outside the file, a
  grid whose tile count does not match its declared rows and columns).
- **No I/O in the decoding path (SBIO).** The core takes `&[u8]` and returns pixels. It does not open
  files and never touches the network, so there is no path-traversal or SSRF surface. The only
  filesystem access in the crate is the `io` module behind the `std` feature, which reads the file
  the caller named and does nothing else.
- **Zero required dependencies**, so the supply-chain surface of a production build is this crate and
  the standard library. `cargo-deny` runs in CI against the advisory database.

### Resource exhaustion

The realistic attack against a safe decoder is making it allocate or compute too much.

- **`DecodeOptions::max_pixels`** caps the output pixel count, checked before any pixel buffer is
  allocated. It defaults to `Some(DEFAULT_MAX_PIXELS)`, where
  `pub const DEFAULT_MAX_PIXELS: u64 = 268_435_456;` (256 megapixels). If a file declares more, the
  decode is refused with an error rather than attempted.
- Lower it for hostile input. A service accepting uploads has no reason to allow 256 Mpx; set it to
  whatever your product actually displays. Setting it to `None` disables the check entirely and is
  appropriate only for files you control.
- Box sizes and item extents are validated against the actual input length before anything is
  allocated, so a header claiming a four-gigabyte box does not cause a four-gigabyte allocation.
- Grid geometry is validated against `max_pixels` too, so a small file describing an enormous grid is
  rejected at the derivation step rather than while compositing.

### Out of scope

- Encoding. This crate does not encode.
- Correctness of pixel output as a security property. Wrong pixels are a bug, not a vulnerability.
- Denial of service by a caller who explicitly set `max_pixels` to `None`.

## Fuzzing

The crate is fuzzed with [`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz) (libFuzzer). The
targets live in [`fuzz/`](fuzz/), a crate of their own that is not part of the published package:

- `decode` — drives `decode` over arbitrary bytes, with `max_pixels` held to four megapixels so a
  legitimately large image is refused rather than mistaken for a resource-exhaustion finding.
- `probe` — drives `probe`, the container parser without pixels: the code path most likely to be
  pointed at hostile input.
- `boxes` — drives the box walk every other parser stands on, and walks each box's payload again.

CI runs each target for two minutes on every push and pull request, from the test fixtures as a
seed corpus, and keeps any crashing input as a build artifact. `cargo-fuzz` needs a nightly toolchain,
which is why fuzzing is its own job rather than part of the stable and MSRV matrix.

```
cargo install cargo-fuzz
./fuzz/seed.sh
cargo +nightly fuzz run probe -- -max_total_time=300
```

The first finding was external: a `uuid` box too small for its own extended type panicked in 0.1.1,
found by fuzzing and reported privately by s17sh70x2jfl031ag. It is fixed in 0.1.2, and the `boxes`
target rediscovers it from an empty corpus when the fix is reverted -- which is how the harness was
shown to be able to find what it is for. The harness's own first finding came on its first CI run: items
assembled from items could be made to demand gigabytes from a few hundred bytes. Fixed in 0.1.3.
Five more panics in the HEVC decoder followed from the same reporter's fuzzing (fixed in 0.1.4); two of
them needed overflow checks to show, so CI now fuzzes with debug assertions on.

Because the decoding path takes `&[u8]` and performs no I/O, a fuzz target is a two-line harness with
no filesystem setup and no mocking. That is one of the concrete payoffs of the SBIO rule.
