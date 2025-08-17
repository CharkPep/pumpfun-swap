use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, declare_id, msg, program_error::ProgramError, pubkey::Pubkey,
};

declare_id!("6dXexJ3SwyRcmdRiqYMTURDx3AX7BTLaHa6ei9bSTEAz");

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub enum Instructions {
    ExecuteSwap(BuyInstruction),
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct BuyInstruction {
    /// `max_quote_amount_in` in the Pumpfun `buy` instruction.
    input_amount: u64,
    slippage_bps: u64,
}

#[derive(Debug)]
pub struct SwapPerformed {
    input_amount: u64,
    output_amount: u64,
}

impl BuyInstruction {
    pub fn new(input_amount: u64, slippage_bps: u64) -> Self {
        BuyInstruction {
            input_amount,
            slippage_bps,
        }
    }
}

pub(crate) mod pumpfun_cpi {
    use borsh::BorshSerialize;
    use solana_instruction::{AccountMeta, Instruction};
    use solana_program::pubkey::Pubkey;

    use super::Error;

    const BUY_DISCRIMINATOR: &[u8] = &[102, 6, 61, 18, 1, 218, 235, 234];

    #[derive(BorshSerialize, Debug)]
    pub struct Buy {
        pub base_amount_out: u64,
        pub max_quote_amount_in: u64,
    }

    pub fn buy(
        pool: &Pubkey,
        user: &Pubkey,
        global_config: &Pubkey,
        base: &Pubkey,
        quote: &Pubkey,
        user_base_ata: &Pubkey,
        user_quote_ata: &Pubkey,
        pool_base_ata: &Pubkey,
        pool_quote_ata: &Pubkey,
        protocol_fee_recipient: &Pubkey,
        protocol_fee_recipient_ata: &Pubkey,
        base_token_program: &Pubkey,
        quote_token_program: &Pubkey,
        system_program: &Pubkey,
        associated_token_program: &Pubkey,
        event_authority: &Pubkey,
        pumpfun_program: &Pubkey,
        coin_creator_vault_ata: &Pubkey,
        coin_creator_vault_authority: &Pubkey,
        global_volume_accumulator: &Pubkey,
        user_volume_accumulator: &Pubkey,
        instruction: Buy,
    ) -> Result<Instruction, Error> {
        let accounts = vec![
            AccountMeta::new_readonly(*pool, false),
            AccountMeta::new(*user, true),
            AccountMeta::new_readonly(*global_config, false),
            AccountMeta::new_readonly(*base, false),
            AccountMeta::new_readonly(*quote, false),
            AccountMeta::new(*user_base_ata, false),
            AccountMeta::new(*user_quote_ata, false),
            AccountMeta::new(*pool_base_ata, false),
            AccountMeta::new(*pool_quote_ata, false),
            AccountMeta::new_readonly(*protocol_fee_recipient, false),
            AccountMeta::new(*protocol_fee_recipient_ata, false),
            AccountMeta::new_readonly(*base_token_program, false),
            AccountMeta::new_readonly(*quote_token_program, false),
            AccountMeta::new_readonly(*system_program, false),
            AccountMeta::new_readonly(*associated_token_program, false),
            AccountMeta::new_readonly(*event_authority, false),
            AccountMeta::new_readonly(*pumpfun_program, false),
            AccountMeta::new(*coin_creator_vault_ata, false),
            AccountMeta::new_readonly(*coin_creator_vault_authority, false),
            AccountMeta::new(*global_volume_accumulator, false),
            AccountMeta::new(*user_volume_accumulator, false),
        ];

        assert!(accounts.len() == 21);
        let mut data = vec![];
        data.extend(BUY_DISCRIMINATOR);
        BorshSerialize::serialize(&instruction, &mut data).map_err(Error::BorshIoError)?;
        let instruction = Instruction {
            program_id: *pumpfun_program,
            data,
            accounts,
        };

        Ok(instruction)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PoolGlobalConfig {
    pub admin: Pubkey,
    pub lp_fee_basis_points: u64,
    pub protocol_fee_basis_points: u64,
    pub disable_flags: u8,
    pub protocol_fee_recipients: [Pubkey; 8],
    pub coin_creator_fee_basis_points: u64,
    pub admin_set_coin_creator_authority: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Pool {
    pub pool_bump: u8,
    pub index: u16,
    pub creator: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub pool_base_token_account: Pubkey,
    pub pool_quote_token_account: Pubkey,
    pub lp_supply: u64,
    pub coin_creator: Pubkey,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct PoolData {
    pub reserve_base: u64,
    pub reserve_quote: u64,

    pub global_config: PoolGlobalConfig,
    pub pool: Pool,
}

impl PoolData {
    fn new(
        pool: &AccountInfo,
        global_config: &AccountInfo,
        pool_base_ata: &AccountInfo,
        pool_quote_ata: &AccountInfo,
    ) -> Result<PoolData, Error> {
        if pool.data.as_ref().borrow().len() < 243 {
            return Err(Error::ProgramError(ProgramError::InvalidArgument));
        }
        let pool = Pool::try_from_slice(&pool.data.as_ref().borrow()[8..243])
            .map_err(Error::BorshIoError)?;

        if global_config.data.as_ref().borrow().len() < 353 {
            return Err(Error::ProgramError(ProgramError::InvalidArgument));
        }

        let global_config =
            PoolGlobalConfig::try_from_slice(&global_config.data.as_ref().borrow()[8..353])
                .map_err(Error::BorshIoError)?;

        Ok(PoolData {
            global_config,
            pool,
            reserve_base: **pool_base_ata.lamports.as_ref().borrow(),
            reserve_quote: **pool_quote_ata.lamports.as_ref().borrow(),
        })
    }

    pub fn base_out(&self, quote_amount_in: u64) -> Result<u64, ProgramError> {
        // Effective quote in amount
        let quote = self.apply_fees(quote_amount_in)?;

        // 1) Find pool reserve relation accounting for `quote_amount_in`
        // 2) Multiply on quote
        //
        // base_out = (reserve_base * quote) / (reserve_quote + quote)
        let num = (self.reserve_base as u128)
            .checked_mul(quote as u128)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        let denom = (self.reserve_quote as u128)
            .checked_add(quote as u128)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        let base_out = num
            .checked_div(denom)
            .ok_or(ProgramError::ArithmeticOverflow)? as u64;

        Ok(base_out)
    }

    fn apply_fees(&self, amount: u64) -> Result<u64, ProgramError> {
        let total_fee_bp = self
            .global_config
            .lp_fee_basis_points
            .checked_add(self.global_config.protocol_fee_basis_points)
            .and_then(|x| x.checked_add(self.global_config.coin_creator_fee_basis_points))
            .ok_or(ProgramError::ArithmeticOverflow)?;

        let fee_amount = (amount as u128)
            .checked_mul(total_fee_bp as u128)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .checked_div(10000u128)
            .ok_or(ProgramError::ArithmeticOverflow)? as u64;

        amount
            .checked_sub(fee_amount)
            .ok_or(ProgramError::ArithmeticOverflow)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ProgramError(#[from] ProgramError),

    #[error(transparent)]
    BorshIoError(borsh::io::Error),

    #[error("insufficient pool reserve")]
    InsufficientPoolReserve,

    #[error("slippage must be within 0 to 100 percent in basis points")]
    SlippageTooHigh,
}

impl Into<ProgramError> for Error {
    fn into(self) -> ProgramError {
        match self {
            Self::ProgramError(err) => err,
            Self::BorshIoError(err) => ProgramError::BorshIoError(err.to_string()),
            Self::InsufficientPoolReserve => ProgramError::InvalidInstructionData,
            Self::SlippageTooHigh => ProgramError::InvalidArgument,
        }
    }
}

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint {
    use borsh::BorshDeserialize;
    use solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::{entrypoint, ProgramResult},
        msg,
        program::invoke,
        program_error::ProgramError,
        pubkey::Pubkey,
    };

    use crate::{pumpfun_cpi, BuyInstruction, Error, Instructions, PoolData, SwapPerformed};

    entrypoint!(process_instruction);

    pub fn process_instruction(
        _: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = Instructions::try_from_slice(instruction_data)?;
        match instruction {
            Instructions::ExecuteSwap(instruction) => {
                msg!("Instruction: ExecuteSwap");
                execute_swap(accounts, instruction)
                    .map_err(|err| <Error as Into<ProgramError>>::into(err))?;
            }
        }

        Ok(())
    }

    pub fn read_pool(accounts: &[AccountInfo]) -> Result<PoolData, Error> {
        let mut accounts = accounts.iter();
        let pool = next_account_info(&mut accounts)?;
        let global_config = next_account_info(&mut accounts)?;
        let pool_base_ata = next_account_info(&mut accounts)?;
        let pool_quote_ata = next_account_info(&mut accounts)?;

        PoolData::new(pool, global_config, pool_base_ata, pool_quote_ata)
    }

    fn sub_slippage(amount: u64, slippage_bps: u64) -> Result<u64, ProgramError> {
        let slippage = ((amount as u128)
            .checked_mul(slippage_bps as u128)
            .and_then(|x| x.checked_div(10000u128))
            .ok_or(ProgramError::ArithmeticOverflow))? as u64;

        let amount = amount
            .checked_sub(slippage)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        Ok(amount)
    }

    fn execute_swap(accounts: &[AccountInfo], instruction: BuyInstruction) -> Result<(), Error> {
        let mut iter = accounts.iter();

        let pool = next_account_info(&mut iter)?;
        let user = next_account_info(&mut iter)?;
        let global_config = next_account_info(&mut iter)?;
        let base = next_account_info(&mut iter)?;
        let quote = next_account_info(&mut iter)?;
        let user_base_ata = next_account_info(&mut iter)?;
        let user_quote_ata = next_account_info(&mut iter)?;
        let pool_base_ata = next_account_info(&mut iter)?;
        let pool_quote_ata = next_account_info(&mut iter)?;
        let protocol_fee_recipient = next_account_info(&mut iter)?;
        let protocol_fee_recipient_ata = next_account_info(&mut iter)?;
        let base_token_program = next_account_info(&mut iter)?;
        let quote_token_program = next_account_info(&mut iter)?;
        let system_program = next_account_info(&mut iter)?;
        let associated_token_program = next_account_info(&mut iter)?;
        let event_authority = next_account_info(&mut iter)?;
        let pumpfun_program = next_account_info(&mut iter)?;
        let coin_creator_vault_ata = next_account_info(&mut iter)?;
        let coint_creator_vault_authority = next_account_info(&mut iter)?;
        let global_volume_accumulator = next_account_info(&mut iter)?;
        let user_volume_accumulator = next_account_info(&mut iter)?;

        if !user.is_signer {
            msg!("Missing user signature");
            return Err(Error::ProgramError(ProgramError::MissingRequiredSignature));
        }

        if instruction.slippage_bps >= 10_000 {
            msg!("Slippage too high: {}", instruction.slippage_bps);
            return Err(Error::SlippageTooHigh);
        }

        let pool_state = PoolData::new(pool, global_config, pool_base_ata, pool_quote_ata)?;
        msg!("Pool: {:?}", pool_state);

        // 1) Calculate expected base out
        // 2) Find what slippage is allowed
        // 3) Remove slippage from the base out
        let base_out = pool_state.base_out(instruction.input_amount)?;
        let base_out = sub_slippage(base_out, instruction.slippage_bps)?;

        let buy = pumpfun_cpi::Buy {
            base_amount_out: base_out,
            max_quote_amount_in: instruction.input_amount,
        };

        msg!("Buy instruction: {:?}", buy);

        let buy = pumpfun_cpi::buy(
            pool.key,
            user.key,
            global_config.key,
            base.key,
            quote.key,
            user_base_ata.key,
            user_quote_ata.key,
            pool_base_ata.key,
            pool_quote_ata.key,
            protocol_fee_recipient.key,
            protocol_fee_recipient_ata.key,
            base_token_program.key,
            quote_token_program.key,
            system_program.key,
            associated_token_program.key,
            event_authority.key,
            pumpfun_program.key,
            coin_creator_vault_ata.key,
            coint_creator_vault_authority.key,
            global_volume_accumulator.key,
            user_volume_accumulator.key,
            buy,
        )?;

        invoke(&buy, accounts)?;
        msg!(
            "{:?}",
            SwapPerformed {
                input_amount: instruction.input_amount,
                output_amount: base_out
            }
        );

        Ok(())
    }
}
