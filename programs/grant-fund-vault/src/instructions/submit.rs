use anchor_lang::prelude::*;

use crate::{GrantDAO, Milestone, MilestoneStatus, ProposalState, ProposalStatus};

#[derive(Accounts)]
pub struct SubmitProposal<'info> {
    #[account(mut)]
    proposer: Signer<'info>,
    #[account(mut)]
    dao: Account<'info, GrantDAO>,
    #[account(
        mut,
        has_one = dao,
        has_one = proposer,
    )]
    proposal: Account<'info, ProposalState>,

    system_program: Program<'info, System>,
}

impl<'info> SubmitProposal<'info> {
    pub fn submit_proposal(&mut self) -> Result<()> {
        let proposal = &mut self.proposal;
        let dao = &mut self.dao;

        require!(
            ProposalStatus::get_proposal_status(proposal.approval_status)
                == Some(ProposalStatus::Pending),
            SubmitCustomError::InvalidProposal
        );
        require!(
            proposal.milestone_count > 0,
            SubmitCustomError::NoMilestones
        );

        dao.proposal_count = dao
            .proposal_count
            .checked_add(1)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        proposal.approval_status = 1;
        // proposal.voting_deadline = deadline;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct SubmitMilestone<'info> {
    #[account(mut)]
    proposer: Signer<'info>,
    #[account(mut)]
    dao: Account<'info, GrantDAO>,
    #[account(
        mut,
        has_one = dao,
        has_one = proposer,
    )]
    proposal: Account<'info, ProposalState>,
    #[account(
        mut,
        has_one = proposal,
        constraint = milestone.status == 0
    )]
    milestone: Account<'info, Milestone>,

    system_program: Program<'info, System>,
}

impl<'info> SubmitMilestone<'info> {
    pub fn submit_milestone(&mut self, index: u8) -> Result<()> {
        require!(
            self.milestone.index == index,
            SubmitCustomError::InvalidMilestone
        );
        require!(
            self.milestone.index < self.proposal.milestone_count,
            SubmitCustomError::InvalidMilestone
        );

        let proposal = &mut self.proposal;
        let milestone = &mut self.milestone;

        milestone.status = MilestoneStatus::Submitted as u8;

        Ok(())
    }
}

#[error_code]
pub enum SubmitCustomError {
    #[msg("Proposal is invalid")]
    InvalidProposal,
    #[msg("No milestone added")]
    NoMilestones,
    #[msg("This proposal milestone is invalid")]
    InvalidMilestone,
}
