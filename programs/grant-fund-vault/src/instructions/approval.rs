use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{GrantDAO, ProposalState, ProposalStatus, VaultState};

#[derive(Accounts)]
pub struct Approve<'info> {
    pub proposer: Signer<'info>,
    #[account(
        mut,
        constraint = authority.key() == dao.authority
    )]
    pub authority: Signer<'info>,
    #[account(
        mut,
        has_one = authority,
    )]
    pub dao: Account<'info, GrantDAO>,
    #[account(
        mut,
        has_one = dao,
        constraint = proposer.key() == proposal.proposer,
        constraint = proposal.milestone_count >= 1,
        constraint = proposal.approval_status == 1,
    )]
    pub proposal: Account<'info, ProposalState>,
    pub treasury_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        token::mint = treasury_mint,
        token::authority = dao,
    )]
    pub treasury: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = authority,
        seeds = [b"vault_state", proposal.key().as_ref()],
        bump,
        space = VaultState::DISCRIMINATOR.len() + VaultState::INIT_SPACE
    )]
    pub vault_state: Account<'info, VaultState>,
    #[account(
        init,
        payer = authority,
        associated_token::mint = treasury_mint,
        associated_token::authority = vault_authority,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: This is a PDA derived using seeds
    /// [b"vault_authority", proposal.key()] and is only used as a signing authority
    /// for the vault token account. No data is read or written.
    #[account(
        seeds = [b"vault_authority", proposal.key().as_ref()],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Approve<'info> {
    pub fn approve_proposal(&mut self, bump: &ApproveBumps) -> Result<()> {
        require!(
            self.proposal.milestone_count >= 1,
            ApprovalCustomError::InvalidMilestone
        );

        let vault_state = &mut self.vault_state;
        let proposal = &mut self.proposal;
        let proposer = &self.proposer;

        proposal.approval_status = 2;

        vault_state.proposal = proposal.key();
        vault_state.beneficiary = proposer.key();
        vault_state.total_amount = self.proposal.total_amount;
        vault_state.bump = bump.vault_state;
        vault_state.released_amount = 0;
        Ok(())
    }

    pub fn transfer_to_vault(&mut self) -> Result<()> {
        require!(
            ProposalStatus::get_proposal_status(self.proposal.approval_status)
                == Some(ProposalStatus::Approved),
            ApprovalCustomError::InvalidProposalStatus
        );

        let transfer_account = TransferChecked {
            from: self.treasury.to_account_info(),
            to: self.vault.to_account_info(),
            mint: self.treasury_mint.to_account_info(),
            authority: self.dao.to_account_info(),
        };

        let seeds: &[&[u8]] = &[b"dao", &[self.dao.bump]];

        let signer_seed = &[seeds];

        let cpi_context = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_account,
            signer_seed,
        );

        transfer_checked(
            cpi_context,
            self.proposal.total_amount,
            self.treasury_mint.decimals,
        )
    }
}

#[error_code]
pub enum ApprovalCustomError {
    #[msg("Invalid proposal status")]
    InvalidProposalStatus,
    #[msg("Invalid milestone")]
    InvalidMilestone,
}
