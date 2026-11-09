# Bountixx Rush Contracts

Soroban contracts for Community Rush, Bountixx's sponsor-funded verified challenge layer.

## Current contract

`rush_vault` is the first-release contract. It locks a sponsor-funded token
amount against a campaign ID and manifest hash, records the verifier active at
funding time, settles once with that verifier's Stellar authorization, and
allows the sponsor to refund only after the stated ledger deadline. It exposes
public campaign/config reads and supports verifier rotation for future
campaigns. Its scope intentionally excludes arbitrary submission scoring,
wallet private-key handling, and Mainnet deployment.

## Contract design principles

The contract settles a rule-bound sponsor reward. It does not evaluate arbitrary submissions. An authorized verifier signs deterministic results off-chain; the contract validates authorization and prevents duplicate settlement.

Read `PROJECT_CONTEXT.md` before contributing.

## Testnet release guardrail

Use `scripts/deploy-testnet.ps1` only after independent review. Its default is
a Testnet-only dry preflight; submitting a transaction requires an explicit
`-Execute -Confirmation DEPLOY_TESTNET`. It does not carry or write account
secrets.
