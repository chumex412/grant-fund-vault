import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import { TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { expect } from "chai";

import { GrantFundVault } from "../target/types/grant_fund_vault";

describe("Grant fund/Milestone vault", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.grantFundVault as Program<GrantFundVault>;

  let daoAuthority = provider.wallet.payer;

  let proposer: anchor.web3.Keypair;
  let proposalPda: anchor.web3.PublicKey;
  let proposalBump: number;
  let treasuryFunderAta: anchor.web3.PublicKey;
  let daoPda: anchor.web3.PublicKey;
  let daoBump: number;
  let treasuryMint: anchor.web3.PublicKey;
  let treasury: anchor.web3.PublicKey;
  let vault: anchor.web3.PublicKey;
  let vaultState: anchor.web3.PublicKey;
  let vaultStateBump: number;
  let vaultAuthorityPda: anchor.web3.PublicKey;
  let beneficiaryTokenAcct: anchor.web3.PublicKey;

  const totalProposalAmt = 1_000;
  const treasuryAmt = 2_000;
  const MILESTONE_COUNT = 3;

  before(async () => {
    await provider.connection.requestAirdrop(
      daoAuthority.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL,
    );

    await new Promise((resolve) => setTimeout(resolve, 1000));

    treasuryMint = await createMint(
      provider.connection,
      daoAuthority,
      daoAuthority.publicKey,
      null,
      0,
    );

    const daoSenderAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      treasuryMint,
      daoAuthority.publicKey,
    );

    treasuryFunderAta = daoSenderAccount.address;

    await mintTo(
      provider.connection,
      provider.wallet.payer,
      treasuryMint,
      treasuryFunderAta,
      daoAuthority,
      treasuryAmt,
    );
  });

  beforeEach(async () => {
    proposer = anchor.web3.Keypair.generate();

    await provider.connection.requestAirdrop(
      proposer.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL,
    );

    await new Promise((resolve) => setTimeout(resolve, 2000));
  });

  async function initializePdas() {
    [daoPda, daoBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("dao")],
      program.programId,
    );

    [treasury] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("treasury"), daoPda.toBuffer()],
      program.programId,
    );

    const beneficiaryAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      daoAuthority,
      treasuryMint,
      proposer.publicKey,
      true,
    );

    beneficiaryTokenAcct = beneficiaryAccount.address;
  }

  async function initializeVaultPdas() {
    [vaultAuthorityPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault_authority"), proposalPda.toBuffer()],
      program.programId,
    );

    vault = getAssociatedTokenAddressSync(
      treasuryMint,
      vaultAuthorityPda,
      true,
    );

    [vaultState, vaultStateBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault_state"), proposalPda.toBuffer()],
      program.programId,
    );
  }

  it("Create and approve a proposal grant", async () => {
    let milestones: anchor.web3.PublicKey[] = [];

    await initializePdas();

    // Initialize DAO
    const daoProgramMethod = program.methods.initializeDao(
      new anchor.BN(treasuryAmt),
    );

    await daoProgramMethod
      .accountsStrict({
        authority: daoAuthority.publicKey,
        senderAta: treasuryFunderAta,
        dao: daoPda,
        treasury: treasury,
        treasuryMint,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })

      .rpc();

    const daoAccount = await program.account.grantDao.fetch(daoPda);

    const proposalCount = new anchor.BN(daoAccount.proposalCount);

    [proposalPda, proposalBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("proposal"),
        daoPda.toBuffer(),
        proposalCount.toArrayLike(Buffer, "le", 8),
      ],
      program.programId,
    );

    await initializeVaultPdas();

    await program.methods
      .initializeProposal(new anchor.BN(totalProposalAmt))
      .accountsStrict({
        proposer: proposer.publicKey,
        dao: daoPda,
        proposal: proposalPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([proposer])
      .rpc();

    let summedAmt = 0;

    for (let i = 0; i < MILESTONE_COUNT; i += 1) {
      const [milestonePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("milestone"), proposalPda.toBuffer(), Buffer.from([i])],
        program.programId,
      );

      let count = i + 1;

      let amount =
        count < MILESTONE_COUNT
          ? Math.floor((count * totalProposalAmt) / 5)
          : totalProposalAmt - summedAmt;

      summedAmt += amount;

      await program.methods
        .createMilestone(new anchor.BN(amount))
        .accountsStrict({
          proposer: proposer.publicKey,
          proposal: proposalPda,
          milestone: milestonePda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([proposer])
        .rpc();

      milestones.push(milestonePda);
    }

    const proposalAccount = await program.account.proposalState.fetch(
      proposalPda,
    );

    expect(proposalAccount.milestoneCount).to.equal(milestones.length);
    expect(proposalAccount.dao.toBase58()).to.equal(daoPda.toBase58());
    expect(proposalAccount.totalAmount.toNumber()).to.equal(totalProposalAmt);
    expect(proposalAccount.bump).to.equal(proposalBump);

    await program.methods
      .submitProposal()
      .accountsStrict({
        proposer: proposer.publicKey,
        dao: daoPda,
        proposal: proposalPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([proposer])
      .rpc();

    const daoAccountInfo = await program.account.grantDao.fetch(daoPda);
    expect(daoAccountInfo.proposalCount.toNumber()).to.equal(1);

    await program.methods
      .approveProposal()
      .accountsStrict({
        proposer: proposer.publicKey,
        authority: daoAuthority.publicKey,
        dao: daoPda,
        proposal: proposalPda,
        treasuryMint,
        treasury,
        vaultState,
        vault,
        vaultAuthority: vaultAuthorityPda,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([proposer])
      .rpc();

    const vaultStateAccount = await program.account.vaultState.fetch(
      vaultState,
    );

    expect(vaultStateAccount.proposal.toBase58()).to.equal(
      proposalPda.toBase58(),
    );
    expect(vaultStateAccount.beneficiary.toBase58()).to.equal(
      proposer.publicKey.toBase58(),
    );
    expect(vaultStateAccount.totalAmount.toNumber()).to.equal(totalProposalAmt);
    expect(vaultStateAccount.releasedAmount.toNumber()).to.equal(0);
    expect(vaultStateAccount.bump).to.equal(vaultStateBump);

    for (let i = 0; i < milestones.length; i += 1) {
      await program.methods
        .submitMilestone(i)
        .accountsStrict({
          proposer: proposer.publicKey,
          dao: daoPda,
          proposal: proposalPda,
          milestone: milestones[i],
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([proposer])
        .rpc();

      await program.methods
        .approveMilestoneForRelease(i)
        .accountsStrict({
          authority: daoAuthority.publicKey,
          proposer: proposer.publicKey,
          proposal: proposalPda,
          dao: daoPda,
          treasury,
          treasuryMint,
          vaultAuthority: vaultAuthorityPda,
          vault,
          vaultState,
          milestone: milestones[i],
          beneficiaryToken: beneficiaryTokenAcct,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([proposer])
        .rpc();
    }

    // Check beneficiary funded in full
    const beneficiaryAccountBalance =
      await provider.connection.getTokenAccountBalance(beneficiaryTokenAcct);

    expect(Number(beneficiaryAccountBalance.value.amount)).to.equal(
      totalProposalAmt,
    );

    // Check closed

    const vaultInfo = await provider.connection.getAccountInfo(vault);

    expect(vaultInfo).to.be.null;
  });

  it("Create proposal and reject a proposal milestone to initiate a refund of the remaining balance", async () => {
    let milestones: anchor.web3.PublicKey[] = [];

    await initializePdas();

    const daoAccount = await program.account.grantDao.fetch(daoPda);

    const proposalCount = new anchor.BN(daoAccount.proposalCount);

    [proposalPda, proposalBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("proposal"),
        daoPda.toBuffer(),
        proposalCount.toArrayLike(Buffer, "le", 8),
      ],
      program.programId,
    );

    await initializeVaultPdas();

    await program.methods
      .initializeProposal(new anchor.BN(totalProposalAmt))
      .accountsStrict({
        proposer: proposer.publicKey,
        dao: daoPda,
        proposal: proposalPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([proposer])
      .rpc();

    const daoAccountInfo = await program.account.grantDao.fetch(daoPda);
    expect(daoAccountInfo.proposalCount.toNumber()).to.equal(1);

    let summedAmt = 0;

    for (let i = 0; i < MILESTONE_COUNT; i += 1) {
      const [milestonePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("milestone"), proposalPda.toBuffer(), Buffer.from([i])],
        program.programId,
      );

      let count = i + 1;

      let amount =
        count < MILESTONE_COUNT
          ? Math.floor((count * totalProposalAmt) / 5)
          : totalProposalAmt - summedAmt;

      summedAmt += amount;

      await program.methods
        .createMilestone(new anchor.BN(amount))
        .accountsStrict({
          proposer: proposer.publicKey,
          proposal: proposalPda,
          milestone: milestonePda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([proposer])
        .rpc();

      milestones.push(milestonePda);
    }

    const proposalAccount = await program.account.proposalState.fetch(
      proposalPda,
    );

    expect(proposalAccount.milestoneCount).to.equal(milestones.length);
    expect(proposalAccount.dao.toBase58()).to.equal(daoPda.toBase58());
    expect(proposalAccount.totalAmount.toNumber()).to.equal(totalProposalAmt);
    expect(proposalAccount.bump).to.equal(proposalBump);

    await program.methods
      .submitProposal()
      .accountsStrict({
        proposer: proposer.publicKey,
        dao: daoPda,
        proposal: proposalPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([proposer])
      .rpc();

    await program.methods
      .approveProposal()
      .accountsStrict({
        proposer: proposer.publicKey,
        authority: daoAuthority.publicKey,
        dao: daoPda,
        proposal: proposalPda,
        treasuryMint,
        treasury,
        vaultState,
        vault,
        vaultAuthority: vaultAuthorityPda,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([proposer])
      .rpc();

    const vaultStateAccount = await program.account.vaultState.fetch(
      vaultState,
    );

    expect(vaultStateAccount.proposal.toBase58()).to.equal(
      proposalPda.toBase58(),
    );
    expect(vaultStateAccount.beneficiary.toBase58()).to.equal(
      proposer.publicKey.toBase58(),
    );
    expect(vaultStateAccount.totalAmount.toNumber()).to.equal(totalProposalAmt);
    expect(vaultStateAccount.releasedAmount.toNumber()).to.equal(0);
    expect(vaultStateAccount.bump).to.equal(vaultStateBump);

    for (let i = 0; i < milestones.length; i += 1) {
      await program.methods
        .submitMilestone(i)
        .accountsStrict({
          proposer: proposer.publicKey,
          dao: daoPda,
          proposal: proposalPda,
          milestone: milestones[i],
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([proposer])
        .rpc();

      if (i < milestones.length - 1) {
        await program.methods
          .approveMilestoneForRelease(i)
          .accountsStrict({
            authority: daoAuthority.publicKey,
            proposer: proposer.publicKey,
            proposal: proposalPda,
            dao: daoPda,
            treasury,
            treasuryMint,
            vaultAuthority: vaultAuthorityPda,
            vault,
            vaultState,
            milestone: milestones[i],
            beneficiaryToken: beneficiaryTokenAcct,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([proposer])
          .rpc();
      } else {
        await program.methods
          .rejectMilestone(i)
          .accountsStrict({
            authority: daoAuthority.publicKey,
            proposer: proposer.publicKey,
            proposal: proposalPda,
            dao: daoPda,
            treasury,
            treasuryMint,
            vaultAuthority: vaultAuthorityPda,
            vault,
            vaultState,
            milestone: milestones[i],
            beneficiaryToken: beneficiaryTokenAcct,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([proposer])
          .rpc();

        break;
      }
    }

    // Checks that the beneficiary balance is less than proposed amount
    const beneficiaryAccountBalance =
      await provider.connection.getTokenAccountBalance(beneficiaryTokenAcct);
    expect(Number(beneficiaryAccountBalance.value.amount)).to.lessThan(
      totalProposalAmt,
    );

    // Check closed

    const vaultInfo = await provider.connection.getAccountInfo(vault);

    expect(vaultInfo).to.be.null;
  });
});
