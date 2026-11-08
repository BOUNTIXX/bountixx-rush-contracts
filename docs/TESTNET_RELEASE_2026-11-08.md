# Rush Vault Testnet Release — 2026-11-08

## Deployment record

- Network: Stellar Testnet
- Contract ID: `CCPJVD6QMCJPD7XX6TE6GTTKOS3FOWWFGAMBZDPA2AHDWW5VOJVI6OF2`
- Contract: [Stellar Lab contract view](https://lab.stellar.org/r/testnet/contract/CCPJVD6QMCJPD7XX6TE6GTTKOS3FOWWFGAMBZDPA2AHDWW5VOJVI6OF2)
- WASM SHA-256: `3bd0843bdd251b43b697a9739673bcc72dfde59c2dd8082085928b583e532769`
- WASM build: non-optimized, 7500 bytes

## Transactions

- [WASM upload](https://stellar.expert/explorer/testnet/tx/931a06d34ea47b6995e79596110e24b677ada37f145ece6f9abe9ff26a59ab83)
- [Contract deployment](https://stellar.expert/explorer/testnet/tx/c6c1663422340eeb6b6c51eed2e20932a553a0e79f364402153ed72c8ff23be6)
- [Vault initialization](https://stellar.expert/explorer/testnet/tx/e7b8535123468035c1b542ab78d272d1c2909f01340ae78ad43977ab8b81410c)

## Public configuration verified after initialization

- Admin: `GDXT6YL4T2D6D6NGGCR2FUS2C5TU4S7CEMGN56PQFXKCF5FPYXAPIUVF`
- Verifier: `GB4TKRPYF5UKNDLTTA5V3IRVRPHPTMRY6GU5ASLXIPXN45CMQ7VRKVGO`

The associated private identities were created in the Windows secure store and
are not stored in this repository. This release deploys an empty vault only.
No campaign has been funded, no participant address has been collected, and no
XLM reward has been promised or settled.

## Follow-up gates

Before a funded fixture campaign, provision the verifier as a private service
with a durable atomic idempotency store, secret-managed Ed25519 signing key,
and trusted locked-manifest reader. Then exercise funding, a deterministic
attestation, settlement, and expiry/refund as separate Testnet transactions.
