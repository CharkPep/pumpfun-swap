mod common;

use anyhow::Error;
use borsh::BorshSerialize;
use common::TestData;
use pumpfun_global::{
    derive_amm_user_volume_accumulator, derive_pool, PUMPFUN_AMM_PROGRAM,
    PUMP_FUN_AMM_COIN_CREATOR_VAULT_AUTHORITY, PUMP_FUN_AMM_EVENT_AUTHORITY,
    PUMP_FUN_AMM_FEE_RECIPIENT, PUMP_FUN_AMM_GLOBAL_VOLUME_ACCUMULATOR, PUMP_FUN_GLOBAL_CONFIG,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_instruction::{AccountMeta, Instruction};
use solana_sdk::{native_token::LAMPORTS_PER_SOL, signer::Signer, transaction::Transaction};

#[tokio::test]
async fn test_huge_slippage_swap() -> Result<(), Error> {
    common::setup_logger();
    let rpc =
        RpcClient::new_with_commitment(common::DEV_NET.to_owned(), CommitmentConfig::finalized());

    let TestData { payer, mint } = common::setup(
        &rpc,
        (0.005 * LAMPORTS_PER_SOL as f64) as u64,
        (0.005 * LAMPORTS_PER_SOL as f64) as u64,
    )
    .await?;

    common::transfer_wsol(&rpc, &payer, (0.005 * LAMPORTS_PER_SOL as f64) as u64).await?;
    let (pool, _) = derive_pool(
        0,
        &payer.pubkey(),
        &mint.pubkey(),
        &spl_token::native_mint::id(),
    );

    let (user_volume_accumulator, _) = derive_amm_user_volume_accumulator(&payer.pubkey());

    let accounts = vec![
        AccountMeta::new_readonly(pool, false),
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new_readonly(PUMP_FUN_GLOBAL_CONFIG, false),
        AccountMeta::new_readonly(mint.pubkey(), false),
        AccountMeta::new_readonly(spl_token::native_mint::id(), false),
        AccountMeta::new(
            spl_associated_token_account::get_associated_token_address(
                &payer.pubkey(),
                &mint.pubkey(),
            ),
            false,
        ),
        AccountMeta::new(
            spl_associated_token_account::get_associated_token_address(
                &payer.pubkey(),
                &spl_token::native_mint::id(),
            ),
            false,
        ),
        AccountMeta::new(
            spl_associated_token_account::get_associated_token_address(&pool, &mint.pubkey()),
            false,
        ),
        AccountMeta::new(
            spl_associated_token_account::get_associated_token_address(
                &pool,
                &spl_token::native_mint::id(),
            ),
            false,
        ),
        AccountMeta::new_readonly(PUMP_FUN_AMM_FEE_RECIPIENT, false),
        AccountMeta::new(
            spl_associated_token_account::get_associated_token_address(
                &PUMP_FUN_AMM_FEE_RECIPIENT,
                &spl_token::native_mint::id(),
            ),
            false,
        ),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        AccountMeta::new_readonly(PUMP_FUN_AMM_EVENT_AUTHORITY, false),
        AccountMeta::new_readonly(PUMPFUN_AMM_PROGRAM, false),
        AccountMeta::new(
            spl_associated_token_account::get_associated_token_address(
                &PUMP_FUN_AMM_COIN_CREATOR_VAULT_AUTHORITY,
                &spl_token::native_mint::id(),
            ),
            false,
        ),
        AccountMeta::new_readonly(PUMP_FUN_AMM_COIN_CREATOR_VAULT_AUTHORITY, false),
        AccountMeta::new(PUMP_FUN_AMM_GLOBAL_VOLUME_ACCUMULATOR, false),
        AccountMeta::new(user_volume_accumulator, false),
    ];

    assert!(accounts.len() == 21, "{}", accounts.len());
    let instruction = pumpfun_amm::Instructions::ExecuteSwap(pumpfun_amm::BuyInstruction::new(
        (0.005 * LAMPORTS_PER_SOL as f64) as u64,
        10_000, // 100%
    ));

    let mut data = vec![];
    BorshSerialize::serialize(&instruction, &mut data)?;

    let instructions = &[Instruction {
        data,
        accounts,
        program_id: pumpfun_amm::id(),
    }];

    let mut tx = Transaction::new_with_payer(instructions, Some(&payer.pubkey()));
    let recent_blockhash = rpc.get_latest_blockhash().await?;
    tx.sign(&[payer], recent_blockhash);
    // Transaction failed, no funds can be deposited on the ATAs, no on-chain changes made.
    assert!(
        rpc.send_and_confirm_transaction(&tx).await.is_err(),
        "expected transaction to fail"
    );

    Ok(())
}
