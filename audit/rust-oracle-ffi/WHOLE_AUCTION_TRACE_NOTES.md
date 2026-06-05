# Whole-Auction Trace Example

The original review also used whole-auction trace differentials. Those tests drove a real Solidity auction through block changes, bids, checkpoints, and settlement, then compared the resulting fields against a Rust trace model.

This handoff package does not include that trace harness as a runnable example. The trace code depends on review-only Solidity scaffolding and a larger Rust model, so including it here would make the package harder to review. The archived original work includes the trace files if the team wants to inspect that layer:

- `test/math/AuctionTraceRolloverSmoke.t.sol`
- `test/utils/AuctionTraceHarness.sol`
- `review/math-oracle/src/auction.rs`
- `review/math-oracle/src/trace.rs`
- `review/math-oracle/src/differential.rs`

The trace layer is separate from the local arithmetic FFI examples. The local examples compare individual formulas and settlement projections. The trace layer compares full auction state after a sequence of external calls.
