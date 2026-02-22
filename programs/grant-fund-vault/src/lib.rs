use anchor_lang::prelude::*;

declare_id!("BDzXLKCGd1NNMsFWwAmr6DwgWrbabNygU67WMnYtMeNX");

#[program]
pub mod grant_fund_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
