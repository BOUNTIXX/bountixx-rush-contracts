# Bountixx Rush Contracts

Soroban contracts for Community Rush, Bountixx's sponsor-funded verified challenge layer.

## Planned contract modules

- `rush_vault`: sponsor funding, payout, cancellation, refund, and expiry.
- `rush_registry`: campaign identity, manifest hashes, and authorized verifier configuration.
- `result_attestation`: replay-safe recording of signed outcomes.

## Contract design principles

The contract settles a rule-bound sponsor reward. It does not evaluate arbitrary submissions. An authorized verifier signs deterministic results off-chain; the contract validates authorization and prevents duplicate settlement.

Read `PROJECT_CONTEXT.md` before contributing.

## Testnet release guardrail

Use `scripts/deploy-testnet.ps1` only after independent review. Its default is
a Testnet-only dry preflight; submitting a transaction requires an explicit
`-Execute -Confirmation DEPLOY_TESTNET`. It does not carry or write account
secrets.
