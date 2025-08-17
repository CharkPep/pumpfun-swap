use std::{
    env,
    fs::{self, File},
    path::Path,
};

use anyhow::Error;
use borsh::BorshDeserialize;
use pumpfun_global::{derive_bounding_curve, derive_pool};
use pumpfun_instructions::launchpad::{buy, create_token, Buy, CreateToken};
use pumpfun_instructions::{
    amm::{create_pool, CreatePool},
    launchpad::BoundingCurve,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
use tracing::{info, warn};

pub static DEV_NET: &str = "https://api.devnet.solana.com";
const DECIMALS_PER_MINT: u64 = 1_000_000;

pub struct TestData {
    pub payer: Keypair,
    pub mint: Keypair,
}

pub fn read_or_persist_keypair(path: impl AsRef<Path>) -> Result<Keypair, Error> {
    if !fs::exists(&path)? {
        let keypair = Keypair::new();
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }

        let fd = File::options().create(true).write(true).open(&path)?;
        serde_json::to_writer(fd, keypair.to_bytes().as_ref())?;
        return Ok(keypair);
    }

    let bytes = fs::read(path)?;
    let keypair: Vec<u8> = serde_json::from_slice(&bytes)?;
    let keypair = Keypair::from_bytes(&keypair)?;

    Ok(keypair)
}

pub async fn read_bounding_curve(rpc: &RpcClient, mint: &Pubkey) -> Result<BoundingCurve, Error> {
    let (bounding_curve, _) = derive_bounding_curve(mint);
    let mut bounding_curve = rpc.get_account_data(&bounding_curve).await?;
    Ok(BorshDeserialize::deserialize(&mut bounding_curve.as_ref())?)
}

async fn _setup(
    rpc: &RpcClient,
    payer: &Keypair,
    mint: &Keypair,
    user_base_ata: &Pubkey,
    user_quote_ata: &Pubkey,
    lamports_in_buy: u64,
    lamports_in_pool: u64,
) -> Result<(), Error> {
    let setup_instructions = vec![
        create_token(
            &payer.pubkey(),
            &mint.pubkey(),
            CreateToken {
                uri: "".to_owned(),
                name: "Test token".to_owned(),
                creater: payer.pubkey(),
                symbol: "TTS".to_owned(),
            },
        ),
        spl_associated_token_account::instruction::create_associated_token_account(
            &payer.pubkey(),
            &payer.pubkey(),
            &mint.pubkey(),
            &spl_token::id(),
        ),
        spl_associated_token_account::instruction::create_associated_token_account_idempotent(
            &payer.pubkey(),
            &payer.pubkey(),
            &spl_token::native_mint::id(),
            &spl_token::id(),
        ),
        solana_sdk::system_instruction::transfer(
            &payer.pubkey(),
            &user_quote_ata,
            lamports_in_pool,
        ),
        spl_token::instruction::sync_native(&spl_token::id(), &user_quote_ata)?,
    ];

    let recent_blockhash = rpc.get_latest_blockhash().await?;
    let mut tx0 = Transaction::new_with_payer(&setup_instructions, Some(&payer.pubkey()));
    tx0.sign(&[mint, payer], recent_blockhash);
    let sig0 = rpc.send_and_confirm_transaction(&tx0).await?;

    info!("Token setup complete: {}", sig0);

    let bounding_curve = read_bounding_curve(rpc, &mint.pubkey()).await?;
    let price = (bounding_curve.virtual_sol_reserves as f64 / LAMPORTS_PER_SOL as f64)
        / (bounding_curve.virtual_token_reserves as f64 / DECIMALS_PER_MINT as f64);
    let amount = (price * LAMPORTS_PER_SOL as f64 * lamports_in_buy as f64) as u64;
    info!("Amount out: {}", amount);

    let pool_instructions = vec![
        buy(
            &payer.pubkey(),
            &user_base_ata,
            &mint.pubkey(),
            &payer.pubkey(),
            Buy {
                amount: amount,
                max_sol_cost: lamports_in_pool,
            },
        ),
        create_pool(
            &payer.pubkey(),
            &mint.pubkey(),                // Base
            &spl_token::native_mint::id(), // Quote
            &user_base_ata,
            &user_quote_ata,
            CreatePool {
                index: 0,
                coin_creator: payer.pubkey(),
                base_amount_in: amount,
                quote_amount_in: lamports_in_pool,
            },
        ),
    ];

    let recent_blockhash = rpc.get_latest_blockhash().await?;
    let mut tx1 = Transaction::new_with_payer(&pool_instructions, Some(&payer.pubkey()));
    tx1.sign(&[payer], recent_blockhash);
    let sig1 = rpc.send_and_confirm_transaction(&tx1).await?;

    info!("Pool created: {}", sig1);

    Ok(())
}

async fn pool_exists(rpc: &RpcClient, payer: &Pubkey, mint: &Pubkey) -> bool {
    let (pool, _) = derive_pool(0, payer, &spl_token::native_mint::id(), mint);

    // For simplicity, assume that pool does not exists even if RPC fails.
    rpc.get_account(&pool).await.is_ok()
}

pub async fn setup(rpc: &RpcClient, sol_in_buy: u64, sol_in_pool: u64) -> Result<TestData, Error> {
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();

    let payer = env::var("PAYER").unwrap_or(
        Path::new(&std::env::var_os("HOME").expect("missing HOME directory"))
            .join(".config")
            .join("solana")
            .join("id.json")
            .to_str()
            .unwrap()
            .to_owned(),
    );

    warn!(
        "Using payer: {}, total: {} SOL",
        payer,
        (sol_in_buy + sol_in_pool) as f64 / LAMPORTS_PER_SOL as f64
    );

    let payer = read_or_persist_keypair(payer)?;
    let mint = read_or_persist_keypair(format!("testdata/generated/run_{}/mint.json", id))?;
    let user_base_ata =
        spl_associated_token_account::get_associated_token_address(&payer.pubkey(), &mint.pubkey());

    let user_quote_ata = spl_associated_token_account::get_associated_token_address(
        &payer.pubkey(),
        &spl_token::native_mint::ID,
    );

    info!("Mint ATA: {}", user_base_ata);

    // Some basic checks
    if pool_exists(rpc, &payer.pubkey(), &mint.pubkey()).await {
        panic!("Expected pool be uninitialized");
    }

    _setup(
        rpc,
        &payer,
        &mint,
        &user_base_ata,
        &user_quote_ata,
        sol_in_buy,
        sol_in_pool,
    )
    .await?;

    Ok(TestData { payer, mint })
}

pub async fn transfer_wsol(rpc: &RpcClient, payer: &Keypair, lamports: u64) -> Result<(), Error> {
    let wsol = spl_associated_token_account::get_associated_token_address(
        &payer.pubkey(),
        &spl_token::native_mint::id(),
    );
    let recent_blockhash = rpc.get_latest_blockhash().await?;
    let mut tx = Transaction::new_with_payer(
        &[
            solana_sdk::system_instruction::transfer(&payer.pubkey(), &wsol, lamports),
            spl_token::instruction::sync_native(&spl_token::id(), &wsol)?,
        ],
        Some(&payer.pubkey()),
    );

    tx.sign(&[payer], recent_blockhash);
    let sig = rpc.send_and_confirm_transaction(&tx).await?;
    info!(
        "Wrapped {} SOL, {}",
        lamports as f64 / LAMPORTS_PER_SOL as f64,
        sig
    );

    Ok(())
}

pub fn setup_logger() {
    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}
