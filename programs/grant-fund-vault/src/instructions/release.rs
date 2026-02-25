use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{CloseAccount, Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked, close_account},
};

use crate::{GrantDAO, Milestone, MilestoneStatus, ProposalState, ProposalStatus, VaultState};

#[derive(Accounts)]

pub struct Release<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    #[account(mut)]
    proposer: Signer<'info>,
    #[account(
        mut,
        has_one = dao,
        constraint = proposal.approval_status == 2,
    )]
    proposal: Account<'info, ProposalState>,
    #[account(
        mut,
        constraint = dao.proposal_count >= 1,
        constraint = dao.authority == authority.key()
    )]
    dao: Account<'info, GrantDAO>,
    #[account(
        mut,
        token::mint = treasury_mint,
        token::authority = dao,
    )]
    treasury: InterfaceAccount<'info, TokenAccount>,
    treasury_mint: InterfaceAccount<'info, Mint>,
    /// CHECK: This is a PDA derived using seeds 
    /// [b"vault_authority", proposal.key()] and is only used as a signing authority
    /// for the vault token account. No data is read or written.
    #[account(
        seeds = [b"vault_authority", proposal.key().as_ref()],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = treasury_mint,
        associated_token::authority = vault_authority,
        associated_token::token_program = token_program
    )]
    vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        has_one = proposal,
        constraint = vault_state.beneficiary == proposal.proposer,
        constraint = vault_state.total_amount > 0
    )]
    vault_state: Account<'info, VaultState>,
    #[account(
        mut,
        has_one = proposal,
        constraint = proposal.milestone_count >= 1,
        constraint = milestone.status == 1, 
    )]
    milestone: Account<'info, Milestone>,
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = treasury_mint,
        associated_token::authority = proposer,
        associated_token::token_program = token_program
    )]
    beneficiary_token: InterfaceAccount<'info, TokenAccount>,

    associated_token_program: Program<'info, AssociatedToken>,
    system_program: Program<'info, System>,
    token_program: Interface<'info, TokenInterface>,
}

impl<'info> Release<'info> {
    pub fn approve_milestone(&mut self, index: u8) -> Result<()> {
        require!(self.milestone.index == index, CustomError::InvalidMilestone);

        require!(
            MilestoneStatus::get_milestone_status(self.milestone.status) == Some(MilestoneStatus::Submitted), 
            CustomError::MilestoneNotSubmitted
        );

        let milestone = &mut self.milestone;

        milestone.status = MilestoneStatus::Approved as u8;

        Ok(())
    }

    pub fn reject_milestone(&mut self, index: u8) -> Result<()> {
        require!(self.milestone.index == index, CustomError::InvalidMilestone);

        require!(
            MilestoneStatus::get_milestone_status(self.milestone.status) == Some(MilestoneStatus::Submitted), 
            CustomError::MilestoneNotSubmitted
        );

        let milestone = &mut self.milestone;

        milestone.status = MilestoneStatus::Rejected as u8;

        Ok(())
    }

    pub fn approve_release(&mut self) -> Result<()> {
        require!(
            MilestoneStatus::get_milestone_status(self.milestone.status) == Some(MilestoneStatus::Approved), 
            CustomError::MilestoneNotApproved
        );

        require!(
            self.vault_state.released_amount + self.milestone.amount <= self.proposal.total_amount, 
            CustomError::OverRelease
        );

        let proposal_key = self.proposal.key();
        let state_bump = self.vault_state.bump;
        let vault_state = &mut self.vault_state;

        let transfer_account = TransferChecked {
            to: self.beneficiary_token.to_account_info(),
            from: self.vault.to_account_info(),
            mint: self.treasury_mint.to_account_info(),
            authority: self.vault_authority.to_account_info(),
        };

        let seeds: &[&[u8]] = &[
            b"vault_authority",
            proposal_key.as_ref(),
            &[state_bump]
        ];

        let signer_seed = &[seeds];

        let cpi_context = CpiContext::new_with_signer(self.token_program.to_account_info(), transfer_account, signer_seed);

        transfer_checked(cpi_context,self.milestone.amount, self.treasury_mint.decimals)?;

        vault_state.released_amount += self.milestone.amount;

        self.milestone.status = MilestoneStatus::Completed as u8;

        if vault_state.released_amount >= self.proposal.total_amount {
            let close_accounts = CloseAccount {
                account: self.vault.to_account_info(),
                destination: self.proposer.to_account_info(),
                authority: self.vault_authority.to_account_info()
            };

            let close_cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), close_accounts, signer_seed);

            close_account(close_cpi_ctx)?;
        }


        Ok(())
    }

    pub fn refund(&mut self, index:u8) -> Result<()> {
         require!(self.milestone.index == index, CustomError::InvalidMilestone);
        require!(
            self.milestone.index < self.proposal.milestone_count,
            CustomError::InvalidMilestone
        );
        require!(
            MilestoneStatus::get_milestone_status(self.milestone.status) == Some(MilestoneStatus::Rejected), 
            CustomError::MilestoneNotRejected
        );

        let proposal_key = self.proposal.key();
        let state_bump = self.vault_state.bump;

        let transfer_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.treasury.to_account_info(),
            mint: self.treasury_mint.to_account_info(),
            authority: self.vault_authority.to_account_info(),
        };

        let seeds: &[&[u8]] = &[
            b"vault_authority",
            proposal_key.as_ref(),
            &[state_bump]
        ];

        let signer_seeds = &[seeds];

        let transfer_cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), transfer_accounts, signer_seeds);

        self.proposal.approval_status = ProposalStatus::Rejected as u8;

        let amount = self.proposal.total_amount - self.vault_state.released_amount;
        transfer_checked(transfer_cpi_ctx, amount, self.treasury_mint.decimals)?;

        let close_accounts = CloseAccount {
            account:self.vault.to_account_info(),
            destination: self.treasury.to_account_info(),
            authority: self.vault_authority.to_account_info(),
        };

        let close_cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), close_accounts, signer_seeds);

        close_account(close_cpi_ctx)

    }
}

#[error_code]
pub enum CustomError {
    #[msg("This proposal milestone is invalid")]
    InvalidMilestone,
    #[msg("No milestone added")]
    NoMilestones,
    #[msg("Proposal milestone wasn't submitted")]
    MilestoneNotSubmitted,
    #[msg("Proposal milestone isn't approved for fund release")]
    MilestoneNotApproved,
    #[msg("Proposal milestone must be rejected to continue")]
    MilestoneNotRejected,
    #[msg("You've hit the release limit for this proposal")]
    OverRelease  
}
