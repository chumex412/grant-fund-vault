use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[account]
pub struct GrantDAO {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub treasury_mint: Pubkey,
    pub proposal_count: u64,
    pub bump: u8,
}

#[derive(InitSpace)]
#[account]
pub struct VaultState {
    pub proposal: Pubkey,
    pub beneficiary: Pubkey,
    pub total_amount: u64,
    pub released_amount: u64,
    pub bump: u8,
    pub seed: u64,
}
