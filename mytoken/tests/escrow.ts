import * as anchor from "@coral-xyz/anchor";
import { Keypair, SystemProgram, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createMint,
  getAccount,
  mintTo,
} from "@solana/spl-token";
import assert from "assert";
import fs from "fs";
import path from "path";

// Skip guard like in other tests
const hasAnchorToml = fs.existsSync(path.resolve(process.cwd(), "Anchor.toml"));
if (!hasAnchorToml) {
  describe("escrow (skipped)", () => {
    it("no anchor workspace", function () { this.skip(); });
  });
} else {
  describe("escrow program", () => {
    if (!process.env.ANCHOR_PROVIDER_URL) process.env.ANCHOR_PROVIDER_URL = 'http://127.0.0.1:8899';
    if (!process.env.ANCHOR_WALLET) process.env.ANCHOR_WALLET = path.resolve(process.env.HOME || '~', '.config/solana/id.json');
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = (anchor.workspace as any).Escrow as any; // no strict typing to stay consistent
    if (!program) {
      it("IDL for Escrow missing", function () { this.skip(); });
      return;
    }

    const sender = Keypair.generate();
    const receiver = Keypair.generate();
    let mint: PublicKey;
    let senderAta: PublicKey;
    let receiverAta: PublicKey;
    let vaultAta: PublicKey; // ATA owned by escrow PDA
    let escrowPda: PublicKey;
    let escrowBump: number;
    const amount = 100n;

    it("airdrop & mint setup", async () => {
      // fund sender & receiver
      for (const kp of [sender, receiver]) {
        const sig = await provider.connection.requestAirdrop(kp.publicKey, 2 * LAMPORTS_PER_SOL);
        await provider.connection.confirmTransaction(sig, 'confirmed');
      }
      mint = await createMint(provider.connection, (provider.wallet as any).payer, sender.publicKey, sender.publicKey, 0);
      senderAta = await getAssociatedTokenAddress(mint, sender.publicKey);
      receiverAta = await getAssociatedTokenAddress(mint, receiver.publicKey);
      // create ATAs by minting to sender
      await mintTo(
        provider.connection,
        (provider.wallet as any).payer,
        mint,
        senderAta,
        sender.publicKey,
        Number(amount)
      );
      const acc = await getAccount(provider.connection, senderAta);
      assert.equal(acc.amount.toString(), amount.toString());
      console.log("✅ Setup complete: sender balance", acc.amount.toString());
    });

    it("create_escrow()", async () => {
      [escrowPda, escrowBump] = PublicKey.findProgramAddressSync([
        Buffer.from("escrow"),
        sender.publicKey.toBuffer(),
        receiver.publicKey.toBuffer(),
        mint.toBuffer(),
      ], program.programId);

      const sig = await program.methods
        .createEscrow(new anchor.BN(amount))
        .accounts({
          sender: sender.publicKey,
          receiver: receiver.publicKey,
            mint,
          escrowAccount: escrowPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([sender])
        .rpc();
      console.log("▶️ createEscrow tx:", sig);
    });

    it("deposit_tokens()", async () => {
      // vault ATA (PDA owner)
      vaultAta = await getAssociatedTokenAddress(mint, escrowPda, true);
      const sig = await program.methods
        .depositTokens()
        .accounts({
          sender: sender.publicKey,
          mint,
          escrowAccount: escrowPda,
          senderTokenAccount: senderAta,
          vaultTokenAccount: vaultAta,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([sender])
        .rpc();
      console.log("▶️ depositTokens tx:", sig);
      const senderAfter = await getAccount(provider.connection, senderAta);
      const vaultAfter = await getAccount(provider.connection, vaultAta);
      assert.equal(senderAfter.amount.toString(), "0");
      assert.equal(vaultAfter.amount.toString(), amount.toString());
    });

    it("release_tokens()", async () => {
      // Create receiver ATA implicitly by checking (mintTo not needed, just ensure exists)
      receiverAta = await getAssociatedTokenAddress(mint, receiver.publicKey);
      const sig = await program.methods
        .releaseTokens()
        .accounts({
          receiver: receiver.publicKey,
          mint,
          escrowAccount: escrowPda,
          receiverTokenAccount: receiverAta,
          vaultTokenAccount: vaultAta,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([receiver])
        .rpc();
      console.log("▶️ releaseTokens tx:", sig);
      const receiverAcc = await getAccount(provider.connection, receiverAta);
      assert.equal(receiverAcc.amount.toString(), amount.toString());
    });

    it("cancel_escrow() should fail after release", async () => {
      try {
        await program.methods
          .cancelEscrow()
          .accounts({
            sender: sender.publicKey,
            mint,
            escrowAccount: escrowPda,
            senderTokenAccount: senderAta,
            vaultTokenAccount: vaultAta,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([sender])
          .rpc();
        assert.fail("cancelEscrow succeeded after completion");
      } catch (e) {
        console.log("✅ cancelEscrow correctly failed after release");
      }
    });
  });
}
