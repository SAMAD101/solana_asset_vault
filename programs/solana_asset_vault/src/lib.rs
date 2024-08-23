use anchor_lang::prelude::*;

declare_id!("B3G43MVazHAwnxHpg5R1u2bCUkm7FnCaMgh2ikViL7aN");

#[program]
pub mod solana_asset_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
