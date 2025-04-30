#!/usr/bin/env bash
# check_env.sh -- Checks for required environment variables and config for Fusion arbitrage backend

REQUIRED_VARS=(APP_PRIVATE_KEY APP_PROFIT_WALLET APP_INFURA_API_KEY)
MISSING=()

for VAR in "${REQUIRED_VARS[@]}"; do
  if [[ -z "${!VAR}" ]]; then
    MISSING+=("$VAR")
  fi
done

if [[ ${#MISSING[@]} -gt 0 ]]; then
  echo "[ERROR] Missing required environment variables: ${MISSING[*]}"
  exit 1
else
  echo "[OK] All required environment variables are set."
fi

# Check contract address
ADDR_LINE=$(grep 'ARBITRAGE_EXECUTOR_MAINNET' src/arbitrage_executor_address.rs)
if [[ "$ADDR_LINE" == *"0x..."* || "$ADDR_LINE" == *"<PASTE_YOUR_MAINNET_CONTRACT_ADDRESS_HERE>"* ]]; then
  echo "[ERROR] ARBITRAGE_EXECUTOR_MAINNET is not set to a real contract address in src/arbitrage_executor_address.rs"
  exit 2
else
  echo "[OK] ArbitrageExecutor contract address is set."
fi

# Check ABI file
if [[ ! -f out/ArbitrageExecutor.sol/ArbitrageExecutor.json ]]; then
  echo "[ERROR] ABI file out/ArbitrageExecutor.sol/ArbitrageExecutor.json not found!"
  exit 3
else
  echo "[OK] ABI file found."
fi

# Check BSC RPC URL
BSC_RPC=$(grep '^bsc_rpc_url' config/default.toml | cut -d'=' -f2- | tr -d '" ')
if [[ -z "$BSC_RPC" ]]; then
  echo "[ERROR] bsc_rpc_url is not set in config/default.toml"
  exit 4
else
  echo "[OK] bsc_rpc_url is set: $BSC_RPC"
fi

exit 0
