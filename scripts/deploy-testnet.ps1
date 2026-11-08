[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)]
  [string] $AdminIdentity,

  [Parameter(Mandatory = $true)]
  [string] $AdminAddress,

  [Parameter(Mandatory = $true)]
  [string] $VerifierAddress,

  [string] $RpcUrl = "https://soroban-testnet.stellar.org",

  [string] $NetworkPassphrase = "Test SDF Network ; September 2015",

  [switch] $Execute,

  [string] $Confirmation
)

$ErrorActionPreference = "Stop"

if ($NetworkPassphrase -ne "Test SDF Network ; September 2015") {
  throw "This release script is intentionally Testnet-only. Refusing a non-Testnet network passphrase."
}

if (-not (Get-Command stellar -ErrorAction SilentlyContinue)) {
  throw "The Stellar CLI is required. Install the pinned, reviewed CLI before continuing."
}

$workspaceRoot = Split-Path -Parent $PSScriptRoot
$wasmPath = Join-Path $workspaceRoot "target\wasm32v1-none\release\rush_vault.wasm"

Push-Location $workspaceRoot
try {
  & stellar contract build --optimize=false
  if ($LASTEXITCODE -ne 0) { throw "Soroban build failed." }
} finally {
  Pop-Location
}

if (-not (Test-Path -LiteralPath $wasmPath)) {
  throw "Expected contract artifact was not produced: $wasmPath"
}

$wasmHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $wasmPath).Hash.ToLowerInvariant()
Write-Host "Testnet deployment preflight passed."
Write-Host "WASM: $wasmPath"
Write-Host "WASM SHA-256: $wasmHash"
Write-Host "Admin address: $AdminAddress"
Write-Host "Verifier address: $VerifierAddress"
Write-Host "RPC: $RpcUrl"

if (-not $Execute) {
  Write-Host "Dry run only. Re-run with -Execute -Confirmation DEPLOY_TESTNET after independent review."
  exit 0
}

if ($Confirmation -cne "DEPLOY_TESTNET") {
  throw "Refusing deployment without -Confirmation DEPLOY_TESTNET."
}

$contractId = & stellar contract deploy `
  --wasm $wasmPath `
  --source-account $AdminIdentity `
  --rpc-url $RpcUrl `
  --network-passphrase $NetworkPassphrase `
  --alias bountixx-rush-vault
if ($LASTEXITCODE -ne 0 -or -not $contractId) { throw "Contract deployment failed." }

$contractId = $contractId.Trim()
Write-Host "Deployed contract ID: $contractId"

& stellar contract invoke `
  --id $contractId `
  --source-account $AdminIdentity `
  --rpc-url $RpcUrl `
  --network-passphrase $NetworkPassphrase `
  --send yes `
  -- initialize `
  --admin $AdminAddress `
  --verifier $VerifierAddress
if ($LASTEXITCODE -ne 0) { throw "Contract deployment succeeded but initialize failed. Do not fund this contract until initialization is resolved." }

Write-Host "Rush Vault initialized on Stellar Testnet. Record the contract ID, initialization transaction, artifact hash, admin address, and verifier address in a release note."
