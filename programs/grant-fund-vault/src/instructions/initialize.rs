use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{GrantDAO, ProposalState};

#[derive(Accounts)]
pub struct InitializeDao<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        associated_token::mint = treasury_mint,
        associated_token::authority = authority,
        associated_token::token_program = token_program,
        constraint = sender_ata.owner == authority.key(),
        constraint = sender_ata.mint == treasury_mint.key()
    )]
    pub sender_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = authority,
        space = GrantDAO::DISCRIMINATOR.len() + GrantDAO::INIT_SPACE,
        seeds = [b"dao"],
        bump
    )]
    pub dao: Account<'info, GrantDAO>,
    #[account(
        init_if_needed,
        payer = authority,
        token::mint = treasury_mint,
        token::authority = dao,
        seeds = [b"treasury", dao.key().as_ref()],
        bump
    )]
    pub treasury: InterfaceAccount<'info, TokenAccount>,
    #[account(mint::token_program = token_program)]
    pub treasury_mint: InterfaceAccount<'info, Mint>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> InitializeDao<'info> {
    pub fn initialize_dao(&mut self, bump: &InitializeDaoBumps) -> Result<()> {
        let dao = &mut self.dao;
        let authority = &self.authority;

        dao.authority = authority.key();
        dao.treasury = self.treasury.key();
        dao.treasury_mint = self.treasury_mint.key();
        dao.bump = bump.dao;
        dao.proposal_count = 0;

        Ok(())
    }

    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let transfer_account = TransferChecked {
            from: self.sender_ata.to_account_info(),
            to: self.treasury.to_account_info(),
            mint: self.treasury_mint.to_account_info(),
            authority: self.authority.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_account);

        transfer_checked(cpi_ctx, amount, self.treasury_mint.decimals)
    }
}

#[derive(Accounts)]
pub struct GrantFunding<'info> {
    #[account(mut)]
    proposal: Account<'info, ProposalState>,
    dao: Account<'info, GrantDAO>,
}

impl<'info> GrantFunding<'info> {
    pub fn approve() -> Result<()> {
        Ok(())
    }

    pub fn deposit() -> Result<()> {
        Ok(())
    }
}
