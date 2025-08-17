use solana_sdk::pubkey::Pubkey;

pub static PUMP_FUN_LAUNCHPAD_PROGRAM: Pubkey =
    Pubkey::from_str_const("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");

pub static PUMPFUN_AMM_PROGRAM: Pubkey =
    Pubkey::from_str_const("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");

pub static PUMP_FUN_MINT_AUTHORITY: Pubkey =
    Pubkey::from_str_const("TSLvdd1pWpHVjahSpsvCXUbgwsL3JAcvokwaKt1eokM");

pub static PUMP_FUN_GLOBAL_CONFIG: Pubkey =
    Pubkey::from_str_const("ADyA8hdefvWN2dbGGWFotbzWxrAvLW83WG6QCVXvJKqw");

pub static PUMP_FUN_LAUNCHPAD_EVENT_AUTHORITHY: Pubkey =
    Pubkey::from_str_const("Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1");

pub static PUMP_FUN_AMM_EVENT_AUTHORITY: Pubkey =
    Pubkey::from_str_const("GS4CU59F31iL7aR2Q8zVS8DRrcRnXX1yjQ66TqNVQnaR");

pub static PUMP_FUN_LAUNCHPAD_GLOBAL_VOLUME_ACCUMULATOR: Pubkey =
    Pubkey::from_str_const("Hq2wp8uJ9jCPsYgNHex8RtqdvMPfVGoYwjvF1ATiwn2Y");

pub static PUMP_FUN_AMM_GLOBAL_VOLUME_ACCUMULATOR: Pubkey =
    Pubkey::from_str_const("C2aFPdENg4A2HQsmrd5rTw5TaYBX5Ku887cWjbFKtZpw");

// Note: It seems that pool.coin_creator is always System Program,
// if it changes the code will break
pub static PUMP_FUN_AMM_COIN_CREATOR_VAULT_AUTHORITY: Pubkey =
    Pubkey::from_str_const("8N3GDaZ2iwN65oxVatKTLPNooAVUJTbfiVJ1ahyqwjSk");

pub static PUMP_FUN_LAUNCHPAD_FEE_RECIPIENT: Pubkey =
    Pubkey::from_str_const("68yFSZxzLWJXkxxRGydZ63C6mHx1NLEDWmwN9Lb5yySg");

// Note: Global config value that may be different on the different clusters or updated at some
// point
pub static PUMP_FUN_AMM_FEE_RECIPIENT: Pubkey =
    Pubkey::from_str_const("12e2F4DKkD3Lff6WPYsU7Xd76SHPEyN9T8XSsTJNF8oT");

pub static GLOBAL: Pubkey = Pubkey::from_str_const("4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf");

pub static MPL_TOKEN_PROGRAM: Pubkey =
    Pubkey::from_str_const("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

pub static METADATA_PROGRAM: Pubkey =
    Pubkey::from_str_const("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

// Derive Platform PDA for on curve token's instructions fee
pub(crate) fn derive_global_volume_accumulator() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"global_volume_accumulator"], &PUMP_FUN_LAUNCHPAD_PROGRAM)
}

pub(crate) fn derive_event_authority() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"__event_authority"], &PUMP_FUN_LAUNCHPAD_PROGRAM)
}

pub(crate) fn derive_global_config() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"global_config"], &PUMPFUN_AMM_PROGRAM)
}

pub fn derive_bounding_curve(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"bonding-curve", mint.as_ref()],
        &PUMP_FUN_LAUNCHPAD_PROGRAM,
    )
}

pub fn derive_associated_bounding_curve(bounding_curve: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            bounding_curve.as_ref(),
            spl_token::id().as_ref(),
            mint.as_ref(),
        ],
        &spl_associated_token_account::id(),
    )
}

pub fn derive_metadata(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"metadata", METADATA_PROGRAM.as_ref(), mint.as_ref()],
        &METADATA_PROGRAM,
    )
}

pub fn derive_creator_vault(creator: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"creator-vault", creator.as_ref()],
        &PUMP_FUN_LAUNCHPAD_PROGRAM,
    )
}

pub fn derive_coin_creator_vault_authority(creator: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"creator_vault", creator.as_ref()], &PUMPFUN_AMM_PROGRAM)
}

pub fn derive_user_volume_accumulator(asscotiated_user: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"user_volume_accumulator", asscotiated_user.as_ref()],
        &PUMP_FUN_LAUNCHPAD_PROGRAM,
    )
}

// Note: common function would be great
pub fn derive_amm_user_volume_accumulator(asscotiated_user: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"user_volume_accumulator", asscotiated_user.as_ref()],
        &PUMPFUN_AMM_PROGRAM,
    )
}

pub fn derive_pool(index: u16, creator: &Pubkey, base: &Pubkey, quote: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"pool",
            &index.to_be_bytes(),
            creator.as_ref(),
            base.as_ref(),
            quote.as_ref(),
        ],
        &PUMPFUN_AMM_PROGRAM,
    )
}

pub fn derive_pool_mint(pool: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"pool_lp_mint", pool.as_ref()], &PUMPFUN_AMM_PROGRAM)
}

pub fn derive_user_lp_ata(creator: &Pubkey, lp_mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            creator.as_ref(),
            spl_token_2022::id().as_ref(),
            lp_mint.as_ref(),
        ],
        &spl_associated_token_account::id(),
    )
}

pub fn derive_pool_ata(pool: &Pubkey, token_program: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[pool.as_ref(), token_program.as_ref(), mint.as_ref()],
        &spl_associated_token_account::id(),
    )
}
