use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ProposalStatus {
    Draft = 0,
    Pending = 1,
    Approved = 2,
    Rejected = 3,
}

impl ProposalStatus {
    pub fn get_proposal_status(value: u8) -> Option<Self> {
        match value {
            0 => Some(ProposalStatus::Draft),
            1 => Some(ProposalStatus::Pending),
            2 => Some(ProposalStatus::Approved),
            3 => Some(ProposalStatus::Rejected),
            _ => None,
        }
    }
}

#[derive(InitSpace)]
#[account]
pub struct ProposalState {
    pub dao: Pubkey,
    pub proposer: Pubkey,
    pub total_amount: u64,
    pub milestone_count: u8,
    pub approval_status: u8,
    pub created_at: i64,
    // pub voting_deadline: i64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum MilestoneStatus {
    Pending = 0,
    Submitted = 1,
    Approved = 2,
    Completed = 3,
    Rejected = 4,
}

impl MilestoneStatus {
    pub fn get_milestone_status(value: u8) -> Option<Self> {
        match value {
            0 => Some(MilestoneStatus::Pending),
            1 => Some(MilestoneStatus::Submitted),
            2 => Some(MilestoneStatus::Approved),
            3 => Some(MilestoneStatus::Completed),
            4 => Some(MilestoneStatus::Rejected),
            _ => None,
        }
    }
}

#[derive(InitSpace)]
#[account]
pub struct Milestone {
    pub proposal: Pubkey,
    pub index: u8,
    pub amount: u64,
    pub status: u8,
    pub bump: u8,
}
