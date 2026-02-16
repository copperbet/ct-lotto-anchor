import * as anchor from "@coral-xyz/anchor";
import { CtLottoAnchor } from "../target/types/ct_lotto_anchor";
import {
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";

import { nanoid } from "nanoid";
import { sha256 } from "@noble/hashes/sha2";

const hexToU8_8 = (hex) => {
  // Remove 0x if present
  hex = hex.replace(/^0x/, "");

  // Ensure even-length hex (required for parsing)
  if (hex.length % 2 !== 0) {
    hex = "0" + hex;
  }

  const bytes = [];

  for (let i = 0; i < hex.length; i += 2) {
    const byte = parseInt(hex.substring(i, i + 2), 16);

    if (isNaN(byte)) {
      throw new Error("Invalid hex string");
    }

    bytes.push(byte);
  }

  // Pad to 8 bytes
  while (bytes.length < 8) {
    bytes.push(0);
  }

  // Ensure exactly 8 bytes
  return bytes.slice(0, 8);
}

describe("ct-lotto-anchor", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.ctAnchorLotto as anchor.Program<CtLottoAnchor>;
  const provider = anchor.getProvider();

  const constants = {
    lotterySeed: nanoid(5),
    lamportsPerTicket: LAMPORTS_PER_SOL * 0.001,
    ticket_code_start_hex: "0",
    ticket_code_end_hex: "f",
    lotteryNumbersToPurchase: ["0", "1", "3", "f"],
    winningNumber: "f",
    platformFeePercentage: 2,
  };

  // PDAs
  const accounts = {
    configuration: PublicKey.findProgramAddressSync(
      [Buffer.from("configuration")],
      program.programId
    )[0],

    lottery: PublicKey.findProgramAddressSync(
      [Buffer.from("lottery"), Buffer.from(constants.lotterySeed)],
      program.programId
    )[0],

    admin: provider.wallet.publicKey,

    sbFeedresult: new PublicKey(
      "Bj1QJT7JM2jp3AbPB74vEpvUmik1KD6xZbC9rbQEDTft"
    ),
  };

  let purchaseSignature: string = "4doUjvgFCErsnZZwXDx3MA585DAdvWbynyYvWfzQg2b85ozq3udMPFmKnBppZooaoQo1pmefyPBwggMx8teNnzUo";
  let bundlePda: PublicKey;

  it("Create configuration PDA", async () => {
    const tx = await program.methods
      .createConfigurationPda()
      .accounts({
        configuration: accounts.configuration,
        admin: accounts.admin,
      })
      .rpc();

    console.log("Configuration created:", tx);
  });

  it("Create lottery PDA", async () => {
    const tx = await program.methods
      .createLotteryPda(
        constants.lotterySeed,
        new anchor.BN(constants.lamportsPerTicket),
        constants.ticket_code_start_hex,
        constants.ticket_code_end_hex,
        new anchor.BN(constants.platformFeePercentage)
      )
      .accounts({
        configuration: accounts.configuration,
        lottery: accounts.lottery,
        admin: accounts.admin,
        switchboardFeedBtcBlockDecimal: accounts.sbFeedresult,
      })
      .rpc();

    console.log("Lottery created:", tx);
  });

  it("Send SOL to lottery PDA", async () => {
    const cost =
      constants.lamportsPerTicket * constants.lotteryNumbersToPurchase.length;

    const tx = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: accounts.admin,
        toPubkey: accounts.lottery,
        lamports: cost,
      })
    );

    tx.feePayer = accounts.admin;
    tx.recentBlockhash = (await provider.connection.getLatestBlockhash())
      .blockhash;

    purchaseSignature = await sendAndConfirmTransaction(
      provider.connection,
      tx,
      [provider.wallet.payer]
    );

    console.log("SOL transfer signature:", purchaseSignature);
  });

  it("Create Transaction Bundle PDA (with numbers)", async () => {
    // create tx_sig_hash (32 bytes)
    const tx_sig_hash = sha256(purchaseSignature);

    bundlePda = PublicKey.findProgramAddressSync(
      [
        Buffer.from("bundle"),
        Buffer.from(constants.lotterySeed),
        Buffer.from(tx_sig_hash),
      ],
      program.programId
    )[0];

    const tx = await program.methods
      .createTransactionBundle(
        constants.lotterySeed,
        Array.from(tx_sig_hash), // [u8; 32]
        accounts.admin, // owner
        constants.lotteryNumbersToPurchase.map(hexToU8_8)
      )
      .accounts({
        configuration: accounts.configuration,
        lottery: accounts.lottery,
        bundle: bundlePda,
        admin: accounts.admin,
      })
      .rpc();

    console.log("Transaction Bundle created:", tx);
    console.log("Bundle PDA:", bundlePda.toBase58());
  });

  it("Close Lottery (stop purchases)", async () => {
    const tx = await program.methods
      .closeLottery()
      .accounts({
        configuration: accounts.configuration,
        lottery: accounts.lottery,
        admin: accounts.admin,
      })
      .rpc();

    console.log("Lottery closed:", tx);
  });

  const winnerIndex = constants.lotteryNumbersToPurchase.indexOf(
    constants.winningNumber
  );

  if (winnerIndex >= 0) {
    it("Reward winner using bundle", async () => {
      const tx = await program.methods
        .rewardTransactionBundle(hexToU8_8(constants.winningNumber))
        .accounts({
          configuration: accounts.configuration,
          lottery: accounts.lottery,
          bundle: bundlePda,
          switchboardFeedBtcBlockDecimal: accounts.sbFeedresult,
          owner: accounts.admin,
          admin: accounts.admin
        })
        .rpc();

      console.log("Rewarded:", tx);
    });
  } else {
    it("Refund all tickets (single group refund)", async () => {
      const tx = await program.methods
        .refundTransactionBundle(false)
        .accounts({
          configuration: accounts.configuration,
          lottery: accounts.lottery,
          bundle: bundlePda,
          owner: accounts.admin,
          admin: accounts.admin,
        })
        .rpc();

      console.log("Refunded:", tx);
    });
  }

  it("Close Transaction Bundle PDA", async () => {
    const tx = await program.methods
      .closeTransactionBundle()
      .accounts({
        configuration: accounts.configuration,
        bundle: bundlePda,
        admin: accounts.admin,
      })
      .rpc();

    console.log("Bundle closed:", tx);
  });

  it("Close Lottery PDA", async () => {
    const tx = await program.methods
      .closeLotteryPda()
      .accounts({
        configuration: accounts.configuration,
        lottery: accounts.lottery,
        admin: accounts.admin,
      })
      .rpc();

    console.log("Lottery closed:", tx);
  });

  it("Close Configuration PDA", async () => {
    const tx = await program.methods
      .closeConfigurationPda()
      .accounts({
        configuration: accounts.configuration,
        admin: accounts.admin,
      })
      .rpc();

    console.log("Configuration closed:", tx);
  });
});
