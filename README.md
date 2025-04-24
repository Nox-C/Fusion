# Fusion Configuration & Production Security

## Configuration

- All sensitive configuration (API keys, private keys, wallet addresses, etc.) must be provided via environment variables in production.
- For local development, copy `.env.example` to `.env` and fill in your own values. **Never commit `.env` with secrets to version control.**
- The main config file is `config/default.toml`. This file should NOT contain secrets in production—use environment variable placeholders (e.g., `${INFURA_API_KEY}`) instead.
- Environment variables are loaded automatically using the `dotenvy` crate for development and from the environment in production.

### Required Environment Variables
- `INFURA_API_KEY`
- `ALCHEMY_API_KEY`
- `NODEREAL_API_KEY`
- `PRIVATE_KEY`
- `PROFIT_WALLET`

See `.env.example` for a template.

## Production Security
- **Never commit secrets or sensitive values to version control.**
- Do not store secrets in `config/default.toml` in production.
- Ensure your deployment environment sets all required environment variables securely.
- Review your logs and error handling to avoid leaking secrets.

---

## Foundry

**Foundry is a blazing fast, portable and modular toolkit for Ethereum application development written in Rust.**

Foundry consists of:

-   **Forge**: Ethereum testing framework (like Truffle, Hardhat and DappTools).
-   **Cast**: Swiss army knife for interacting with EVM smart contracts, sending transactions and getting chain data.
-   **Anvil**: Local Ethereum node, akin to Ganache, Hardhat Network.
-   **Chisel**: Fast, utilitarian, and verbose solidity REPL.

## Documentation

https://book.getfoundry.sh/

## Usage

### Build

```shell
$ forge build
```

### Test

```shell
$ forge test
```

### Format

```shell
$ forge fmt
```

### Gas Snapshots

```shell
$ forge snapshot
```

### Anvil

```shell
$ anvil
```

### Deploy

```shell
$ forge script script/Counter.s.sol:CounterScript --rpc-url <your_rpc_url> --private-key <your_private_key>
```

### Cast

```shell
$ cast <subcommand>
```

### Help

```shell
$ forge --help
$ anvil --help
$ cast --help
```
