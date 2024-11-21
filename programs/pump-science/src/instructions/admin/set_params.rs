use crate::{
    errors::ContractError,
    events::*,
    state::{fee_vault::FeeVault, global::*},
};
use anchor_lang::prelude::*;

#[event_cpi]
#[derive(Accounts)]
#[instruction( params: GlobalSettingsInput)]
pub struct SetParams<'info> {
    #[account(mut,
    constraint = authority.key() == global.global_authority.key() @ ContractError::InvalidGlobalAuthority
    )]
    authority: Signer<'info>,

    #[account(
        mut,
        seeds = [Global::SEED_PREFIX.as_bytes()],
        constraint = global.initialized == true @ ContractError::NotInitialized,
        bump,
    )]
    global: Box<Account<'info, Global>>,

    #[account(
        mut,
        seeds = [FeeVault::SEED_PREFIX.as_bytes()],
        bump,
    )]
    fee_vault: Box<Account<'info, FeeVault>>,

    #[account()]
    /// CHECK: This is not dangerous because we don't read or write from this account
    new_authority: Option<UncheckedAccount<'info>>,

    #[account()]
    /// CHECK: This is not dangerous because we don't read or write from this account
    new_migration_authority: Option<UncheckedAccount<'info>>,

    #[account()]
    /// CHECK: This is not dangerous because we don't read or write from this account
    new_withdraw_authority: Option<UncheckedAccount<'info>>,

    system_program: Program<'info, System>,
}

impl SetParams<'_> {
    pub fn handler(ctx: Context<SetParams>, params: GlobalSettingsInput) -> Result<()> {
        let global = &mut ctx.accounts.global;

        global.update_authority(GlobalAuthorityInput {
            global_authority: if let Some(new_authority) = ctx.accounts.new_authority.as_ref() {
                Some(*new_authority.key)
            } else {
                None
            },
            migration_authority: if let Some(new_migration_authority) =
                ctx.accounts.new_migration_authority.as_ref()
            {
                Some(*new_migration_authority.key)
            } else {
                None
            },
        });
        global.update_settings(params.clone());

        if let Some(fee_recipients) = params.fee_recipients {
            ctx.accounts.fee_vault.update_fee_recipients(fee_recipients);
        }

        emit_cpi!(global.into_event());
        msg!("Updated global state");

        Ok(())
    }
}
