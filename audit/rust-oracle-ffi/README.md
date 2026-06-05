# CCA Rust Oracle FFI Handoff

This folder contains a small version of the Rust oracle used during the CCA v2 rounding review, plus two Foundry examples showing how Solidity tests call the oracle through `vm.ffi`.

The folder is meant to show the technique, not to reproduce every review artifact. The original review code included search tooling, mutation samples, whole-trace models, and one-off bug-hunt helpers. Those files are not included here.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml`, `Cargo.lock` | Rust crate manifest and lockfile. |
| `oracle.rs` | Exact rational CCA formulas and rounding projection helpers. |
| `main.rs` | CLI adapter that emits ABI-encoded `uint256` values for Foundry. |
| `DemandLibOracleExample.t.sol` | Drop-in CCA test comparing `DemandLib` outputs against the Rust oracle. |
| `BidAccountingOracleExample.t.sol` | Drop-in CCA test comparing bid settlement rounding against the Rust oracle. |
| `WHOLE_AUCTION_TRACE_NOTES.md` | Notes on the larger whole-auction trace layer from the original review. |

## What The Oracle Checks

The Rust crate starts from intended economic formulas instead of copying Solidity control flow.

The Rust oracle is intentionally not a Solidity-equivalent implementation. It does not attempt to reproduce storage layout, checkpoint traversal, or auction control flow. It computes the intended economic quantities independently, then projects them into the rounding direction expected from the Solidity entry point under test.

Included formulas:

- required demand at a price;
- clearability threshold at a price;
- price implied by demand;
- currency raised at the clearing tick, including the clearing-tick cap;
- public Q96/X7 downscale floor;
- full-fill bid currency spend and token fill projections;
- partial-fill bid currency spend and token fill projections.

The Rust side uses `num-bigint` and `num-rational` so it can compute exact values without `uint256` overflow limits. The CLI projects those exact values into the rounding direction expected from Solidity.

## How Foundry Calls The Oracle

Build the Rust binary:

```sh
cargo build --release --manifest-path audit/rust-oracle-ffi/Cargo.toml
```

The CLI prints ABI-encoded `uint256` output. Solidity tests can decode it directly:

```solidity
return abi.decode(vm.ffi(args), (uint256));
```

The examples default to this binary path:

```text
audit/rust-oracle-ffi/target/release/cca-oracle
```

You can override it when running from another repo:

```sh
CCA_ORACLE_BIN=/absolute/path/to/cca-oracle forge test --contracts audit/rust-oracle-ffi --match-path 'audit/rust-oracle-ffi/*.t.sol' --ffi --threads 1 -vv
```

## How To Run The Examples

From the CCA repo root:

```sh
cargo build --release --manifest-path audit/rust-oracle-ffi/Cargo.toml
forge test --contracts audit/rust-oracle-ffi --match-path 'audit/rust-oracle-ffi/*.t.sol' --ffi --threads 1 -vv
```

The tests also support an explicit binary override:

```sh
CCA_ORACLE_BIN=audit/rust-oracle-ffi/target/release/cca-oracle forge test --contracts audit/rust-oracle-ffi --match-path 'audit/rust-oracle-ffi/*.t.sol' --ffi --threads 1 -vv
```

## How To Add Another Check

1. Add the exact formula or projection helper in `oracle.rs`.
2. Add a CLI command for it in `main.rs`.
3. Build the Rust binary with `cargo build --release --manifest-path audit/rust-oracle-ffi/Cargo.toml`.
4. In Solidity, pass string arguments to `vm.ffi`.
5. Decode the result with `abi.decode(vm.ffi(args), (uint256))`.
6. Assert the Solidity output against the oracle output.

Keep the Rust command small. A good command takes scalar inputs, returns one scalar output, and names the projection direction in the command name, such as `required-demand-ceil` or `full-fill-tokens-floor`.

## What This Package Does Not Include

This package does not include:

- mutation testing;
- objective search scripts;
- the full whole-auction trace model;
- bug-hunt PoCs;
- all rounding review tests.

Those files helped during review, but they make a handoff package harder to read. The archived review bundle preserves them for follow-up.

## Verification Performed

Commands run on June 5, 2026:

```sh
cargo fmt --manifest-path audit/rust-oracle-ffi/Cargo.toml
cargo test --manifest-path audit/rust-oracle-ffi/Cargo.toml
cargo build --release --manifest-path audit/rust-oracle-ffi/Cargo.toml
```

Results:

- Rust oracle tests: 6 passed, 0 failed.
- Release build: passed.

The Foundry examples were run in place from the CCA repo root:

```sh
forge test --contracts audit/rust-oracle-ffi --match-path 'audit/rust-oracle-ffi/*.t.sol' --ffi --threads 1 -vv
```

Results:

- `DemandLibOracleExampleTest`: 4 passed, 0 failed.
- `BidAccountingOracleExampleTest`: 2 passed, 0 failed.

## Recommended Use

Use this package as a starting point for adding independent arithmetic checks to the CCA Solidity suite. For broad assurance, keep the exact economic formulas separate from any Solidity-equivalent helper code, and label each projection by its rounding direction.
