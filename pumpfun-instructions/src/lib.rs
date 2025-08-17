pub mod launchpad {
    use borsh::{BorshDeserialize, BorshSerialize};
    use pumpfun_global::{
        derive_associated_bounding_curve, derive_bounding_curve, derive_creator_vault,
        derive_metadata, derive_user_volume_accumulator, GLOBAL, MPL_TOKEN_PROGRAM,
        PUMP_FUN_LAUNCHPAD_EVENT_AUTHORITHY, PUMP_FUN_LAUNCHPAD_FEE_RECIPIENT,
        PUMP_FUN_LAUNCHPAD_GLOBAL_VOLUME_ACCUMULATOR, PUMP_FUN_LAUNCHPAD_PROGRAM,
        PUMP_FUN_MINT_AUTHORITY,
    };
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    };

    #[derive(BorshSerialize, BorshDeserialize, Debug)]
    pub struct BoundingCurve {
        pub virtual_token_reserves: u64,
        pub virtual_sol_reserves: u64,
        pub real_token_reserves: u64,
        pub real_sol_reserves: u64,
        pub token_total_supply: u64,
        pub complete: bool,
    }

    #[derive(BorshSerialize, BorshDeserialize, Debug)]
    pub struct CreateToken {
        pub name: String,
        pub symbol: String,
        pub uri: String,
        pub creater: Pubkey,
    }

    pub fn create_token(payer: &Pubkey, mint: &Pubkey, instruction: CreateToken) -> Instruction {
        let (bounding_curve, _) = derive_bounding_curve(mint);
        let (associated_bounding_curve, _) =
            derive_associated_bounding_curve(&bounding_curve, mint);
        let (metadata, _) = derive_metadata(mint);

        let accounts = vec![
            AccountMeta::new(*mint, true),
            AccountMeta::new_readonly(PUMP_FUN_MINT_AUTHORITY, false),
            AccountMeta::new(bounding_curve, false),
            AccountMeta::new(associated_bounding_curve, false),
            AccountMeta::new_readonly(GLOBAL, false),
            AccountMeta::new_readonly(MPL_TOKEN_PROGRAM, false),
            AccountMeta::new(metadata, false),
            AccountMeta::new(*payer, true),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new_readonly(solana_sdk::sysvar::rent::id(), false),
            AccountMeta::new_readonly(PUMP_FUN_LAUNCHPAD_EVENT_AUTHORITHY, false),
            AccountMeta::new_readonly(PUMP_FUN_LAUNCHPAD_PROGRAM, false),
        ];

        let mut data = vec![];
        data.extend(&[24, 30, 200, 40, 5, 28, 7, 119]);
        BorshSerialize::serialize(&instruction, &mut data).unwrap();

        Instruction {
            program_id: PUMP_FUN_LAUNCHPAD_PROGRAM,
            accounts,
            data,
        }
    }

    #[derive(BorshSerialize, Debug)]
    pub struct Buy {
        pub amount: u64,
        pub max_sol_cost: u64,
    }

    pub fn buy(
        payer: &Pubkey,
        payer_ata: &Pubkey,
        mint: &Pubkey,
        creator: &Pubkey,
        instruction: Buy,
    ) -> Instruction {
        let discriminator = &[102u8, 6, 61, 18, 1, 218, 235, 234];
        let (bounding_curve, _) = derive_bounding_curve(mint);
        let (associated_bounding_curve, _) =
            derive_associated_bounding_curve(&bounding_curve, mint);
        let (creator_vault, _) = derive_creator_vault(creator); // Creator's fee recipient for on curve operations
        let (user_volume_accumulator, _) = derive_user_volume_accumulator(&payer);
        let accounts = vec![
            AccountMeta::new_readonly(GLOBAL, false),
            AccountMeta::new(PUMP_FUN_LAUNCHPAD_FEE_RECIPIENT, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new(bounding_curve, false),
            AccountMeta::new(associated_bounding_curve, false),
            AccountMeta::new(*payer_ata, false),
            AccountMeta::new(*payer, true),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new(creator_vault, false),
            AccountMeta::new_readonly(PUMP_FUN_LAUNCHPAD_EVENT_AUTHORITHY, false),
            AccountMeta::new_readonly(PUMP_FUN_LAUNCHPAD_PROGRAM, false),
            AccountMeta::new(PUMP_FUN_LAUNCHPAD_GLOBAL_VOLUME_ACCUMULATOR, false),
            AccountMeta::new(user_volume_accumulator, false),
        ];

        let mut data = vec![];
        data.extend(discriminator);
        BorshSerialize::serialize(&instruction, &mut data).unwrap();

        Instruction {
            program_id: PUMP_FUN_LAUNCHPAD_PROGRAM,
            accounts,
            data,
        }
    }
}

pub mod amm {
    use borsh::BorshSerialize;
    use pumpfun_global::{
        derive_pool, derive_pool_ata, derive_pool_mint, derive_user_lp_ata, PUMPFUN_AMM_PROGRAM,
        PUMP_FUN_AMM_EVENT_AUTHORITY, PUMP_FUN_GLOBAL_CONFIG,
    };
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    };

    #[derive(BorshSerialize, Debug)]
    pub struct CreatePool {
        pub index: u16,
        pub base_amount_in: u64,
        pub quote_amount_in: u64,
        pub coin_creator: Pubkey,
    }

    pub fn create_pool(
        creator: &Pubkey,
        base: &Pubkey,
        quote: &Pubkey,
        user_base_ata: &Pubkey,
        user_quote_ata: &Pubkey,
        instruction: CreatePool,
    ) -> Instruction {
        let (pool, _) = derive_pool(instruction.index, creator, base, quote);
        let (lp_mint, _) = derive_pool_mint(&pool);
        let (user_lp_ata, _) = derive_user_lp_ata(creator, &lp_mint);
        let (pool_base_ata, _) = derive_pool_ata(&pool, &spl_token::id(), base);
        let (pool_quote_ata, _) = derive_pool_ata(&pool, &spl_token::id(), quote);
        let accounts = vec![
            AccountMeta::new(pool, false),
            AccountMeta::new(PUMP_FUN_GLOBAL_CONFIG, false),
            AccountMeta::new(*creator, true),
            AccountMeta::new_readonly(*base, false),
            AccountMeta::new_readonly(*quote, false),
            AccountMeta::new(lp_mint, false),
            AccountMeta::new(*user_base_ata, false),
            AccountMeta::new(*user_quote_ata, false),
            AccountMeta::new(user_lp_ata, false),
            AccountMeta::new(pool_base_ata, false),
            AccountMeta::new(pool_quote_ata, false),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
            AccountMeta::new_readonly(spl_token_2022::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new_readonly(PUMP_FUN_AMM_EVENT_AUTHORITY, false),
            AccountMeta::new_readonly(PUMPFUN_AMM_PROGRAM, false),
        ];

        let mut data = vec![];
        let discriminator = &[233, 146, 209, 142, 207, 104, 64, 188];
        data.extend(discriminator);
        BorshSerialize::serialize(&instruction, &mut data).unwrap();

        Instruction {
            program_id: PUMPFUN_AMM_PROGRAM,
            accounts,
            data,
        }
    }
}
