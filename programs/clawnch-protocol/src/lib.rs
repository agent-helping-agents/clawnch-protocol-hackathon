// Clawnch Protocol - Solana Smart Contracts
// Built with Anchor framework

use anchor_lang::prelude::*;

// TODO: Generate real keypair with: solana-keygen grind --starts-with CLWN:1
declare_id!("7LkJPbRAuz6Nn7ZfxzmTk8iaL63AehGv3pTziboKx1LW");

pub mod error;
pub mod state;

use error::ClawnchError;
use state::*;

#[program]
pub mod clawnch {
    use super::*;

    /// Initialize the protocol with fee config
    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        protocol_fee_bps: u16,
        creator_fee_bps: u16,
        buyback_fee_bps: u16,
        staking_fee_bps: u16,
    ) -> Result<()> {
        // Validate total fees = 100%
        require!(
            protocol_fee_bps + creator_fee_bps + buyback_fee_bps + staking_fee_bps == 10000,
            ClawnchError::InvalidFeeConfig
        );

        let fee_config = &mut ctx.accounts.fee_config;
        fee_config.authority = ctx.accounts.authority.key();
        fee_config.protocol_fee_bps = protocol_fee_bps;
        fee_config.creator_fee_bps = creator_fee_bps;
        fee_config.buyback_fee_bps = buyback_fee_bps;
        fee_config.staking_fee_bps = staking_fee_bps;
        fee_config.bump = ctx.bumps.fee_config;

        emit!(ConfigInitialized {
            authority: ctx.accounts.authority.key(),
            protocol_fee_bps,
            creator_fee_bps,
            buyback_fee_bps,
            staking_fee_bps,
        });

        Ok(())
    }

    /// Stake tokens to earn rewards
    pub fn stake(ctx: Context<StakeTokens>, amount: u64) -> Result<()> {
        require!(amount > 0, ClawnchError::InvalidAmount);

        // Transfer SOL from user to staking vault
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.staking_vault.to_account_info(),
            },
        );
        anchor_lang::system_program::transfer(cpi_context, amount)?;

        // Update vault state
        let vault = &mut ctx.accounts.staking_vault;
        vault.staked_amount = vault.staked_amount.checked_add(amount)
            .ok_or(ClawnchError::StakingOverflow)?;
        vault.last_update = Clock::get()?.unix_timestamp;

        emit!(TokensStaked {
            user: ctx.accounts.user.key(),
            amount,
            vault: ctx.accounts.staking_vault.key(),
        });

        Ok(())
    }

    /// Unstake tokens and claim rewards
    pub fn unstake(ctx: Context<UnstakeTokens>) -> Result<()> {
        let vault = &ctx.accounts.staking_vault;
        let staked_amount = vault.staked_amount;
        
        require!(staked_amount > 0, ClawnchError::NothingStaked);

        // Calculate rewards (simplified: 1% annual, proportional to duration)
        let now = Clock::get()?.unix_timestamp;
        let duration = (now - vault.last_update) as u64;
        let reward = staked_amount
            .checked_mul(duration)
            .and_then(|v| v.checked_div(365 * 24 * 3600 * 100))
            .unwrap_or(0);

        let total_payout = staked_amount.checked_add(reward)
            .ok_or(ClawnchError::StakingOverflow)?;

        // Transfer from vault to user (requires PDA signer)
        let _seeds = &[b"staking".as_ref(), &[ctx.accounts.staking_vault.bump]];
        
        **ctx.accounts.staking_vault.to_account_info().try_borrow_mut_lamports()? -= total_payout;
        **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += total_payout;

        // Reset vault
        let vault = &mut ctx.accounts.staking_vault;
        vault.staked_amount = 0;
        vault.last_update = now;

        emit!(TokensUnstaked {
            user: ctx.accounts.user.key(),
            principal: staked_amount,
            reward,
            vault: ctx.accounts.staking_vault.key(),
        });

        Ok(())
    }
}

// ==================== INSTRUCTION CONTEXTS ====================

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + FeeConfig::SPACE,
        seeds = [b"config"],
        bump
    )]
    pub fee_config: Account<'info, FeeConfig>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + StakingVault::SPACE,
        seeds = [b"staking", user.key().as_ref()],
        bump
    )]
    pub staking_vault: Account<'info, StakingVault>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UnstakeTokens<'info> {
    #[account(
        mut,
        seeds = [b"staking", user.key().as_ref()],
        bump = staking_vault.bump,
        has_one = user @ ClawnchError::Unauthorized
    )]
    pub staking_vault: Account<'info, StakingVault>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// ==================== EVENTS ====================

#[event]
pub struct ConfigInitialized {
    pub authority: Pubkey,
    pub protocol_fee_bps: u16,
    pub creator_fee_bps: u16,
    pub buyback_fee_bps: u16,
    pub staking_fee_bps: u16,
}

#[event]
pub struct TokensStaked {
    pub user: Pubkey,
    pub amount: u64,
    pub vault: Pubkey,
}

#[event]
pub struct TokensUnstaked {
    pub user: Pubkey,
    pub principal: u64,
    pub reward: u64,
    pub vault: Pubkey,
}
