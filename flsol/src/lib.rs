use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::get_return_data, system_instruction};
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, Burn};

declare_id!("3u9MdW6xRvP9XSVPqby3BBpH2SsH48hw2McDrnEUp3U8");

#[program]
pub mod flsol {
    use super::*;

    /// Initialize the protocol with fees, treasury, limits, and cooldown.
    pub fn initialize(
        ctx: Context<Initialize>,
        fee_numerator: u64,
        fee_denominator: u64,
        treasury: Pubkey,
        treasury_fee_numerator: u64,
        treasury_fee_denominator: u64,
        max_flash_loan_amount: u64,
        cooldown_slots: u64,
    ) -> Result<()> {
        let cfg = &mut ctx.accounts.config;
        cfg.authority = *ctx.accounts.authority.key;
        cfg.fsol_mint = *ctx.accounts.fsol_mint.to_account_info().key;
        cfg.fee_numerator = fee_numerator;
        cfg.fee_denominator = fee_denominator;
        cfg.treasury = treasury;
        cfg.treasury_fee_numerator = treasury_fee_numerator;
        cfg.treasury_fee_denominator = treasury_fee_denominator;
        cfg.max_flash_loan_amount = max_flash_loan_amount;
        cfg.cooldown_slots = cooldown_slots;
        cfg.paused = false;
        cfg.fee_tiers = Vec::new();
        cfg.bump = ctx.bumps.config;
        cfg.vault_bump = ctx.bumps.vault;
        Ok(())
    }

    /// Stake SOL → mint 1:1 FLSOL.
    pub fn stake(ctx: Context<Stake>, lamports_amount: u64) -> Result<()> {
        // move SOL into vault PDA
        let ix = system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.vault.key(),
            lamports_amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.vault.to_account_info(),
            ],
        )?;

        // mint FLSOL
        let bump = ctx.accounts.config.bump;
        let seeds: &[&[u8]] = &[b"config".as_ref(), &[bump]];
        let signer = &[&seeds[..]];
        let cpi = MintTo {
            mint: ctx.accounts.fsol_mint.to_account_info(),
            to: ctx.accounts.user_fsol_account.to_account_info(),
            authority: ctx.accounts.config.to_account_info(),
        };
        token::mint_to(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi)
                .with_signer(signer),
            lamports_amount,
        )?;
        Ok(())
    }

    /// Unstake: burn FLSOL → withdraw proportional SOL (includes fees).
    pub fn unstake(ctx: Context<Unstake>, fsol_amount: u64) -> Result<()> {
        let vault_lamports = ctx.accounts.vault.to_account_info().lamports();
        let supply = ctx.accounts.fsol_mint.supply;
        require!(supply > 0, CustomError::ZeroSupply);

        let return_amount = vault_lamports
            .checked_mul(fsol_amount).unwrap()
            .checked_div(supply).unwrap();

        // burn FLSOL
        let burn_cpi = Burn {
            mint: ctx.accounts.fsol_mint.to_account_info(),
            from: ctx.accounts.user_fsol_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        token::burn(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), burn_cpi),
            fsol_amount,
        )?;

        // send SOL back
        **ctx.accounts.vault.to_account_info().try_borrow_mut_lamports()? -= return_amount;
        **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += return_amount;
        Ok(())
    }

    /// Harvest fees only, keep FLSOL balance.
    pub fn harvest(ctx: Context<Harvest>, fsol_amount: u64) -> Result<()> {
        let vault_lamports = ctx.accounts.vault.to_account_info().lamports();
        let supply = ctx.accounts.fsol_mint.supply;
        require!(supply > 0, CustomError::ZeroSupply);

        let total_value = vault_lamports
            .checked_mul(fsol_amount).unwrap()
            .checked_div(supply).unwrap();
        let owed = total_value.checked_sub(fsol_amount).unwrap();
        require!(owed > 0, CustomError::NothingToHarvest);

        **ctx.accounts.vault.to_account_info().try_borrow_mut_lamports()? -= owed;
        **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += owed;
        Ok(())
    }

    /// Flash-loan with fee split, max limit, cooldown, pause, callback check.
    pub fn flash_loan(
        ctx: Context<FlashLoan>,
        amount: u64,
        instruction_data: Vec<u8>,
    ) -> Result<()> {
        let cfg = &ctx.accounts.config;

        // pause guard
        require!(!cfg.paused, CustomError::Paused);
        // max limit guard
        require!(amount <= cfg.max_flash_loan_amount, CustomError::LoanTooBig);

        // cooldown guard
        let clock = Clock::get()?;
        let current_slot = clock.slot;
        let record = &mut ctx.accounts.flash_record;
        require!(
            current_slot > record.last_flash_slot.checked_add(cfg.cooldown_slots).unwrap(),
            CustomError::CooldownActive
        );

        // compute base fee or tiered fee
        let mut fee = amount
            .checked_mul(cfg.fee_numerator).unwrap()
            .checked_div(cfg.fee_denominator).unwrap();
        for tier in cfg.fee_tiers.iter().rev() {
            if amount >= tier.threshold {
                fee = amount
                    .checked_mul(tier.numerator).unwrap()
                    .checked_div(tier.denominator).unwrap();
                break;
            }
        }

        // mint the loaned FLSOL
        let bump = cfg.bump;
        let seeds: &[&[u8]] = &[b"config".as_ref(), &[bump]];
        let signer = &[&seeds[..]];
        let mint_cpi = MintTo {
            mint: ctx.accounts.fsol_mint.to_account_info(),
            to: ctx.accounts.user_fsol_account.to_account_info(),
            authority: ctx.accounts.config.to_account_info(),
        };
        token::mint_to(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), mint_cpi)
                .with_signer(signer),
            amount,
        )?;

        // invoke borrower CPI
        let ix = solana_program::instruction::Instruction {
            program_id: ctx.accounts.receiver_program.key(),
            accounts: ctx.remaining_accounts.iter().map(|a| {
                solana_program::instruction::AccountMeta {
                    pubkey: a.key(),
                    is_signer: a.is_signer,
                    is_writable: a.is_writable,
                }
            }).collect(),
            data: instruction_data,
        };
        anchor_lang::solana_program::program::invoke(&ix, &ctx.remaining_accounts)?;

        // require callback success byte == 1
        if let Some((_prog, data)) = get_return_data() {
            require!(data.get(0) == Some(&1u8), CustomError::CallbackFailed);
        } else {
            return Err(CustomError::NoCallback.into());
        }

        // burn the principal back
        let burn_cpi = Burn {
            mint: ctx.accounts.fsol_mint.to_account_info(),
            from: ctx.accounts.user_fsol_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        token::burn(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), burn_cpi),
            amount,
        )?;

        // split fee between vault and treasury
        let treasury_share = fee
            .checked_mul(cfg.treasury_fee_numerator).unwrap()
            .checked_div(cfg.treasury_fee_denominator).unwrap();
        let vault_share = fee.checked_sub(treasury_share).unwrap();

        // transfer vault share
        let ix_vault = system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.vault.key(),
            vault_share,
        );
        anchor_lang::solana_program::program::invoke(
            &ix_vault,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.vault.to_account_info(),
            ],
        )?;

        // transfer treasury share
        let ix_treasury = system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.treasury_account.key(),
            treasury_share,
        );
        anchor_lang::solana_program::program::invoke(
            &ix_treasury,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.treasury_account.to_account_info(),
            ],
        )?;

        // update last flash slot
        record.last_flash_slot = current_slot;

        Ok(())
    }

    /// Admin: update base flash-loan fee
    pub fn update_fees(
        ctx: Context<UpdateFees>,
        new_fee_numerator: u64,
        new_fee_denominator: u64,
    ) -> Result<()> {
        let cfg = &mut ctx.accounts.config;
        require!(ctx.accounts.authority.key() == cfg.authority, CustomError::Unauthorized);
        cfg.fee_numerator = new_fee_numerator;
        cfg.fee_denominator = new_fee_denominator;
        Ok(())
    }

    /// Admin: pause or unpause flash loans
    pub fn set_pause(ctx: Context<SetPause>, paused: bool) -> Result<()> {
        let cfg = &mut ctx.accounts.config;
        require!(ctx.accounts.authority.key() == cfg.authority, CustomError::Unauthorized);
        cfg.paused = paused;
        Ok(())
    }

    /// Admin: add a fee tier
    pub fn add_fee_tier(ctx: Context<ModifyTier>, threshold: u64, numerator: u64, denominator: u64) -> Result<()> {
        let cfg = &mut ctx.accounts.config;
        require!(ctx.accounts.authority.key() == cfg.authority, CustomError::Unauthorized);
        cfg.fee_tiers.push(FeeTier { threshold, numerator, denominator });
        Ok(())
    }

    /// Admin: clear all fee tiers
    pub fn clear_fee_tiers(ctx: Context<ModifyTier>) -> Result<()> {
        let cfg = &mut ctx.accounts.config;
        require!(ctx.accounts.authority.key() == cfg.authority, CustomError::Unauthorized);
        cfg.fee_tiers.clear();
        Ok(())
    }
}

/// Track last flash-loan slot for each user.
#[account]
pub struct FlashRecord {
    pub last_flash_slot: u64,
}

#[account]
pub struct Config {
    pub authority: Pubkey,
    pub fsol_mint: Pubkey,
    pub fee_numerator: u64,
    pub fee_denominator: u64,
    pub treasury: Pubkey,
    pub treasury_fee_numerator: u64,
    pub treasury_fee_denominator: u64,
    pub max_flash_loan_amount: u64,
    pub cooldown_slots: u64,
    pub paused: bool,
    pub fee_tiers: Vec<FeeTier>,
    pub bump: u8,
    pub vault_bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct FeeTier {
    pub threshold: u64,
    pub numerator: u64,
    pub denominator: u64,
}

#[error_code]
pub enum CustomError {
    #[msg("You are not authorized")]              Unauthorized,
    #[msg("Flash loans are paused")]             Paused,
    #[msg("Loan amount exceeds max allowable")]  LoanTooBig,
    #[msg("Cooldown still active")]               CooldownActive,
    #[msg("No return callback data")]            NoCallback,
    #[msg("Callback verification failed")]       CallbackFailed,
    #[msg("Nothing to harvest")]                 NothingToHarvest,
    #[msg("Mint supply is zero")]                ZeroSupply,
}

#[derive(Accounts)]
#[instruction(
    fee_numerator: u64,
    fee_denominator: u64,
    treasury: Pubkey,
    treasury_fee_numerator: u64,
    treasury_fee_denominator: u64,
    max_flash_loan_amount: u64,
    cooldown_slots: u64
)]
pub struct Initialize<'info> {
    #[account(mut)] pub authority: Signer<'info>,
    #[account(
        init, payer = authority,
        seeds = [b"config"], bump,
        space = 8 + 32 + 32 + 8 + 8 + 32 + 8 + 8 + 8 + 1 + 4 + (3*24) + 1 + 1
    )]
    pub config: Account<'info, Config>,
    #[account(
        init, payer = authority,
        mint::decimals = 9,
        mint::authority = config,
        seeds = [b"mint"], bump
    )]
    pub fsol_mint: Account<'info, Mint>,
    #[account(
        init, payer = authority,
        seeds = [b"vault", config.key().as_ref()], bump,
        space = 0
    )]
    pub vault: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)] pub user: Signer<'info>,
    #[account(mut)] pub user_fsol_account: Account<'info, TokenAccount>,
    #[account(mut)] pub fsol_mint: Account<'info, Mint>,
    #[account(seeds = [b"config"], bump = config.bump)] pub config: Account<'info, Config>,
    #[account(mut, seeds = [b"vault", config.key().as_ref()], bump = config.vault_bump)]
    pub vault: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)] pub user: Signer<'info>,
    #[account(mut)] pub user_fsol_account: Account<'info, TokenAccount>,
    #[account(mut)] pub fsol_mint: Account<'info, Mint>,
    #[account(seeds = [b"config"], bump = config.bump)] pub config: Account<'info, Config>,
    #[account(mut, seeds = [b"vault", config.key().as_ref()], bump = config.vault_bump)]
    pub vault: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Harvest<'info> {
    #[account(mut)] pub user: Signer<'info>,
    #[account(mut)] pub vault: AccountInfo<'info>,
    #[account(seeds = [b"config"], bump = config.bump)] pub config: Account<'info, Config>,
    #[account(mut)] pub fsol_mint: Account<'info, Mint>,
}

#[derive(Accounts)]
pub struct FlashLoan<'info> {
    #[account(mut)] pub user: Signer<'info>,
    #[account(mut)] pub user_fsol_account: Account<'info, TokenAccount>,
    #[account(mut)] pub fsol_mint: Account<'info, Mint>,
    #[account(seeds = [b"config"], bump = config.bump)] pub config: Account<'info, Config>,
    #[account(mut, seeds = [b"vault", config.key().as_ref()], bump = config.vault_bump)]
    pub vault: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    #[account(
        init_if_needed,
        payer = user,
        seeds = [b"record", user.key().as_ref()],
        bump,
        space = 8 + 8
    )]
    pub flash_record: Account<'info, FlashRecord>,
    #[account(mut, address = config.treasury)]
    /// CHECK: verified by address constraint
    pub treasury_account: AccountInfo<'info>,
    /// Arbitrary CPI target
    pub receiver_program: AccountInfo<'info>,
    // remaining_accounts → passed to CPI
}

#[derive(Accounts)]
pub struct UpdateFees<'info> {
    #[account(mut)] pub authority: Signer<'info>,
    #[account(mut, has_one = authority)] pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct SetPause<'info> {
    #[account(mut)] pub authority: Signer<'info>,
    #[account(mut, has_one = authority)] pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct ModifyTier<'info> {
    #[account(mut)] pub authority: Signer<'info>,
    #[account(mut, has_one = authority)] pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct SetTreasury<'info> {
    #[account(mut)] pub authority: Signer<'info>,
    #[account(mut, has_one = authority)] pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct SetTreasuryFeeSplit<'info> {
    #[account(mut)] pub authority: Signer<'info>,
    #[account(mut, has_one = authority)] pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct SetMaxFlashLoan<'info> {
    #[account(mut)] pub authority: Signer<'info>,
    #[account(mut, has_one = authority)] pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct SetCooldown<'info> {
    #[account(mut)] pub authority: Signer<'info>,
    #[account(mut, has_one = authority)] pub config: Account<'info, Config>,
}
