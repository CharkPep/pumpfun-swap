## Buid

The contract is located in the `pumpfun-cpi` (`pumpfun_amm`) crate, to build the contract run `cargo build-sbf`. Then deploy it to devnet with `solana program deploy <your_file.so>`.

## Test

Make sure you are using devnet config with the `solana cli`. Airdrop some SOL on it or make sure to have 0.015 SOL + fee to cover the test runs. To run tests run `cargo test --features=no-entrypoint`, to check test logs run `RUST_LOG=info cargo test --features=no-entrypoint -- --nocapture`.
