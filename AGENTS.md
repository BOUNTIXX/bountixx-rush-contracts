# Bountixx Rush Contracts Instructions

Read `PROJECT_CONTEXT.md`, `README.md`, and `docs/TESTNET.md` before making changes. When this repository is in the Drips workspace, also read `../Bountixx_HANDOFF.md`. This repository owns only Soroban contract behavior. Do not put web UI, wallet private keys, or subjective challenge scoring on-chain.

Every state change must have tests for authorization, cancellation/refund, expiry, and duplicate-settlement behavior. Deploy Testnet first. Mainnet is never an implicit goal.
