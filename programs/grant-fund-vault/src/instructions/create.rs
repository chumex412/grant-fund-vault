use anchor_lang::prelude::*;

use crate::{GrantDAO, Milestone, MilestoneStatus, ProposalState, ProposalStatus};

#[derive(Accounts)]
pub struct InitializeProposal<'info> {
    #[account(mut)]
    proposer: Signer<'info>,
    #[account(mut)]
    dao: Account<'info, GrantDAO>,
    #[account(
        init,
        payer = proposer,
        space = ProposalState::DISCRIMINATOR.len() + ProposalState::INIT_SPACE,
        seeds = [b"proposal", dao.key().as_ref(), &dao.proposal_count.to_le_bytes()],
        bump
    )]
    proposal: Account<'info, ProposalState>,

    system_program: Program<'info, System>,
}

impl<'info> InitializeProposal<'info> {
    pub fn create_proposal(
        &mut self,
        total_amount: u64,
        bump: &InitializeProposalBumps,
    ) -> Result<()> {
        let proposal = &mut self.proposal;
        let dao = &mut self.dao;
        let proposer = &self.proposer;

        require!(total_amount > 0, CreateCustomError::InvalidAmount);

        proposal.dao = dao.key();
        proposal.milestone_count = 0;
        proposal.proposer = proposer.key();
        proposal.bump = bump.proposal;
        proposal.created_at = Clock::get()?.unix_timestamp;
        proposal.total_amount = total_amount;
        proposal.approval_status = ProposalStatus::Pending as u8;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateMilestone<'info> {
    #[account(mut)]
    proposer: Signer<'info>,
    #[account(mut)]
    proposal: Account<'info, ProposalState>,
    #[account(
        init,
        payer = proposer,
        space = Milestone::DISCRIMINATOR.len() + Milestone::INIT_SPACE,
        seeds = [b"milestone", proposal.key().as_ref(), &[proposal.milestone_count]],
        bump
    )]
    milestone: Account<'info, Milestone>,
    system_program: Program<'info, System>,
}

impl<'info> CreateMilestone<'info> {
    pub fn create_milestone(&mut self, amount: u64) -> Result<()> {
        require!(
            ProposalStatus::get_proposal_status(self.proposal.approval_status)
                == Some(ProposalStatus::Pending),
            CreateCustomError::InvalidProposal
        );
        let proposal = &mut self.proposal;
        let milestone = &mut self.milestone;

        milestone.proposal = proposal.key();
        milestone.index = proposal.milestone_count;
        milestone.status = MilestoneStatus::Pending as u8;
        milestone.amount = amount;

        proposal.milestone_count += 1;

        Ok(())
    }
}

#[error_code]
pub enum CreateCustomError {
    #[msg("This proposal is invalid")]
    InvalidProposal,
    #[msg("No milestone added")]
    NoMilestones,
    #[msg("This proposal milestone is invalid")]
    InvalidMilestone,
    #[msg("An amount is required")]
    InvalidAmount,
}
