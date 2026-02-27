use anchor_lang::prelude::*;

declare_id!("BDzXLKCGd1NNMsFWwAmr6DwgWrbabNygU67WMnYtMeNX");

pub mod instructions;
pub mod state;

pub use instructions::*;
pub use state::*;

#[program]
pub mod grant_fund_vault {
    use super::*;

    pub fn initialize_dao(ctx: Context<InitializeDao>, amount: u64) -> Result<()> {
        ctx.accounts.initialize_dao(&ctx.bumps)?;

        ctx.accounts.deposit(amount)
    }

    pub fn initialize_proposal(ctx: Context<InitializeProposal>, amount: u64) -> Result<()> {
        ctx.accounts.create_proposal(amount, &ctx.bumps)
    }

    pub fn create_milestone(ctx: Context<CreateMilestone>, amount: u64) -> Result<()> {
        ctx.accounts.create_milestone(amount)
    }

    pub fn submit_proposal(ctx: Context<SubmitProposal>) -> Result<()> {
        ctx.accounts.submit_proposal()
    }

    pub fn approve_proposal(ctx: Context<Approve>) -> Result<()> {
        ctx.accounts.approve_proposal(&ctx.bumps)?;

        ctx.accounts.transfer_to_vault()
    }

    pub fn submit_milestone(ctx: Context<SubmitMilestone>, index: u8) -> Result<()> {
        ctx.accounts.submit_milestone(index)
    }

    pub fn approve_milestone_for_release(ctx: Context<Release>, index: u8) -> Result<()> {
        ctx.accounts.approve_milestone(index)?;

        ctx.accounts.approve_release(&ctx.bumps)
    }

    pub fn reject_milestone(ctx: Context<Release>, index: u8) -> Result<()> {
        ctx.accounts.reject_milestone(index)?;

        ctx.accounts.refund(index, &ctx.bumps)
    }
}
