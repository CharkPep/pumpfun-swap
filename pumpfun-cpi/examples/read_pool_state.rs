use anyhow::Error;
use borsh::BorshDeserialize;
use pumpfun_amm::{Pool, PoolGlobalConfig};
use pumpfun_global::{derive_coin_creator_vault_authority, PUMP_FUN_AMM_FEE_RECIPIENT};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;

#[tokio::main()]
async fn main() -> Result<(), Error> {
    let rpc = RpcClient::new_with_commitment(
        "https://api.mainnet-beta.solana.com".to_owned(),
        CommitmentConfig::finalized(),
    );

    let pool = Pubkey::from_str_const("7KUhhvmmCMZiGdTRg1k1v7fujqtKPY9cUnjmWtZwEn34");
    let global_config = Pubkey::from_str_const("ADyA8hdefvWN2dbGGWFotbzWxrAvLW83WG6QCVXvJKqw");

    let pool_data = rpc.get_account_data(&pool).await?;
    let pool: Pool = BorshDeserialize::try_from_slice(&pool_data[8..243])?;
    println!("{:?}", pool);

    let global_config_data = rpc.get_account_data(&global_config).await?;
    let global_config: PoolGlobalConfig =
        BorshDeserialize::try_from_slice(&global_config_data[8..353])?;
    println!("{:?}", global_config);

    Ok(())
}
