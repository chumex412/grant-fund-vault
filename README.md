# Grant Funding Vault

A Grant Funding vault contract built with Anchor. It implements a simple granding funding process where:

- A DAO holds funds in a treasury wallet, which is a programmatic vault.

- The vault holds Native or SPL or ERC-20 tokens.

- The developer submits a proposal outlining the scope, timeline, and milestone deliverables.
  And wallet address for payment

- The community reviews and votes, however, I skipped the due process/protocols required for proposal review and approval.

- Funds are allocated after milestone approval or refunded after being rejected.

## Table of contents

- [Overview](#overview)
  - [Guidelines](#guidelines)
  - [Prerequisite](#prerequisite)
  - [Challenge](#challenge)
  - [Solution](#solution)
  - [Running Tests](#running-tests)
  - [Screenshot](#screenshot)
  - [Miscellanous](#miscellanous)

## Overview

### Guidelines

- After cloning the project, install the dependencies by running `yarn install` in the CLI.
- Run the command `anchor build` generate the necessary codes for testing.
- Run the command `anchor test`.

### Prerequisite

- Anchor CLI (version 0.32.1 or later) installed via AVM.
- Surfpool CLI installed (for enhanced local testing and runbooks: brew install surfpool on macOS, or from source surfpool).
- Solana CLI tools.
- Node.js/Yarn for tests.

### Challenge

The aim of the contract is to reduce the time between proposal submission, approval, and the disbursement of funds. This eliminates the need for additional voting besides the DAO representative and delays while waiting for the release of the funds after approval.

### Solution

My solution was designing and implementing a Grant funding milestone vault contract that included the following:

- Accounts

- Proposal stages

- Milestone stages

- Contract flow

**Accounts**

```
DAO {
    authority: Pubkey,
    treasury: Pubkey,
    treasury_mint: Pubkey,
    proposal_count: u64,
    bump: u8,
}
```

```
GrantProposal {
    dao: Pubkey,
    proposer: Pubkey,
    total_amount: u64,
    milestone_count: u8,
    pub approval_status: u8,
    created_at: i64,
    bump: u8,
}
```

```
Milestone {
    proposal: Pubkey,
    index: u8,
    amount: u64,
    completed: bool,
    released: bool,
    bump: u8,
}
```

```
VaultState {
    proposal: Pubkey,
    beneficiary: Pubkey,
    total_amount: u64,
    released_amount: u64,
    bump: u8,
}
```

**Proposal stages**

- PENDING
- SUBMITTED
- APPROVED
- REJECTED

**Milestone stages**

- PENDING
- SUBMITTED
- APPROVED
- RELEASED
- REJECTED

**Contract flow**

1. Initialize DAO: This creates the DAO and funds the DAO treasury with the appropraite amount.

2. Proposal and Milestone creation: The proposer creates a proposal and attaches milestones to the proposal.

3. Proposal submission: The proposer submits the created proposal for review and approval. This step ensures that the proposal has milestone attached via the `milestone_count` on the `Proposal Account`.

4. Proposal approval: The DAO reviews and approves the proposal for milestones implementation. This step creates a unique vault for the proposal and transfers the total proposal amount from the treasury to the vault.

5. Milestone submission and approval: The proposer submits each milestone for review and approval.

6. Milestone approval: This step involves milestone approval and the transfer of a portion of the proposal amount via the milestone's `amount` field. A `vault authority` authorizes the transfer of the amount to the beneficiary's token account.

7. Milestone rejection: If a milestone is rejected at any stage, the proposal process is aborted and DAO treasury gets a refund of the remaining amount.

### Running Tests

To run the integration tests against a Surfpool local validator:

1. In one terminal window, start Surfpool in your project directory: surfpool start This launches a local Surfnet validator and deploys your program.

2. In a new terminal window (in the same directory), run the Anchor tests against it: anchor test --skip-local-validator This will execute the tests in tests/escrow.ts, covering initialize, deposit, withdraw, and close scenarios.

For standard local testing without Surfpool, just run anchor test.

### Screenshot

![A screenshot of the passing test](https://res.cloudinary.com/da8vqkdmt/image/upload/v1772191517/Screen_Shot_2026-02-27_at_12.23.31_PM_do3b3w.png)

### Miscellanous

Program ID - BDzXLKCGd1NNMsFWwAmr6DwgWrbabNygU67WMnYtMeNX
