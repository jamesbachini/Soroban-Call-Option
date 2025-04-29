# XLM Call Option Smart Contract

A Soroban smart contract implementing a call option for XLM with USDC as the strike currency. This contract allows users to create, purchase, and settle a European-style call option contract on the Stellar network.

## Key Features
- **Fixed Parameters**: 1 XLM collateral, 0.50 USDC strike price, 0.10 USDC premium
- **Expiration Date**: Settles automatically on January 1, 2026
- **Oracle-Based Settlement**: Uses external price feed for final settlement
- **Non-Transferrable**: Single buyer/seller structure per contract
- **Automatic Payouts**: Settles in either XLM or USDC based on expiry price

## Contract Flow
1. **Initialization**: Seller locks XLM collateral and sets parameters
2. **Purchase**: Buyer pays premium in USDC to acquire option rights
3. **Price Update**: Authorized oracle provides price feed before expiry
4. **Claim**: Automatic settlement executes based on XLM/USDC price at expiry

## Usage
```bash
cargo test
cargo build --target wasm32-unknown-unknown --release
stellar contract deploy  --wasm target/wasm32-unknown-unknown/release/sorocall.wasm --source YourWallet --network testnet
```

## Requirements
- [Soroban SDK](https://soroban.stellar.org)
- SEP-41 Token Implementation
- Stellar Classic Asset wrappers for XLM and USDC

## Installation
1. Clone repository
2. Install dependencies:
```bash
rustup target add wasm32-unknown-unknown
cargo install --locked stellar-cli --features opt
```
3. See usage.

## License
MIT