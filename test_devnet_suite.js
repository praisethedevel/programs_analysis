// SPDX-License-Identifier: PACARL-1.0
// Copyright (c) 2026 praisethedevel <xsolutionx@gmail.com>. All Rights Reserved.
// Governed by the Praisethedevel Universal Architecture, Compulsory Attribution & Commercial Royalty License.
// Non-commercial / evaluation use only. Commercial exploitation strictly requires prior written agreement.

const fs = require('fs');
const path = require('path');
const {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction
} = require('/home/joe/projects/solforge-studio/node_modules/@solana/web3.js');

const RPC_URL = 'https://api.devnet.solana.com';
const connection = new Connection(RPC_URL, 'confirmed');

const TOKEN_PROGRAM_ID = new PublicKey('CShaS9ezrJoEYBXsAk4bsUzL2pbxE14kAadgBCeXS7Z1');
const SYSCLOCK_PROGRAM_ID = new PublicKey('8VB675MfjjiaizM2VDBRQfk8GMKCiZVtMELqGdstgMTK');
const RAM_PROGRAM_ID = new PublicKey('5oFB4c7Answx5c2reA9w5u3DGcE1HHw2PfhkXTRFvc3m');

async function main() {
  console.log('=== STARTING LIVE SOLANA DEVNET ON-CHAIN TRANSACTION SUITE ===');
  const payerKeyBytes = JSON.parse(fs.readFileSync('/home/joe/projects/keypairs/funded.json', 'utf8'));
  const payer = await Keypair.fromSecretKey(Uint8Array.from(payerKeyBytes));

  console.log(`Payer: ${payer.publicKey.toBase58()}`);
  const balance = await connection.getBalance(payer.publicKey);
  console.log(`Payer Devnet balance: ${Number(balance) / 1e9} SOL\n`);

  const results = {
    network: 'devnet',
    payer: payer.publicKey.toBase58(),
    token: {},
    sysclock: {},
    ram: {}
  };

  // 1. TEST CUSTOM TOKEN PROGRAM ON DEVNET
  console.log('>>> [1/3] Testing CustomToken on Devnet...');
  const mintKp = await Keypair.generate();
  const aliceKp = await Keypair.generate();
  const bobKp = await Keypair.generate();
  const MINT_SIZE = 80;
  const ACCOUNT_SIZE = 80;

  const rentMint = await connection.getMinimumBalanceForRentExemption(MINT_SIZE);
  const rentAccount = await connection.getMinimumBalanceForRentExemption(ACCOUNT_SIZE);

  // 1.1 Init Mint
  {
    console.log('  1.1 Initializing Mint with 6 decimals & 100 bps fee...');
    const createMintIx = SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: mintKp.publicKey,
      lamports: rentMint,
      space: MINT_SIZE,
      programId: TOKEN_PROGRAM_ID
    });

    const initMintData = Buffer.alloc(4);
    initMintData.writeUInt8(0, 0); // Tag 0 = InitializeMint
    initMintData.writeUInt8(6, 1); // 6 decimals
    initMintData.writeUInt16LE(100, 2); // 100 bps (1%)

    const initMintIx = new TransactionInstruction({
      programId: TOKEN_PROGRAM_ID,
      keys: [
        { pubkey: mintKp.publicKey, isSigner: false, isWritable: true },
        { pubkey: payer.publicKey, isSigner: true, isWritable: false }
      ],
      data: initMintData
    });

    const tx = new Transaction().add(createMintIx, initMintIx);
    const sig = await sendAndConfirmTransaction(connection, tx, [payer, mintKp]);
    console.log(`    Mint: ${mintKp.publicKey.toBase58()}`);
    console.log(`    Tx: https://explorer.solana.com/tx/${sig}?cluster=devnet`);
    results.token.initMint = { sig, mint: mintKp.publicKey.toBase58() };
  }

  // 1.2 Init Alice Account
  {
    console.log('  1.2 Initializing Alice Account...');
    const createAliceIx = SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: aliceKp.publicKey,
      lamports: rentAccount,
      space: ACCOUNT_SIZE,
      programId: TOKEN_PROGRAM_ID
    });

    const initAliceIx = new TransactionInstruction({
      programId: TOKEN_PROGRAM_ID,
      keys: [
        { pubkey: aliceKp.publicKey, isSigner: false, isWritable: true },
        { pubkey: mintKp.publicKey, isSigner: false, isWritable: false },
        { pubkey: payer.publicKey, isSigner: false, isWritable: false }
      ],
      data: Buffer.from([1])
    });

    const tx = new Transaction().add(createAliceIx, initAliceIx);
    const sig = await sendAndConfirmTransaction(connection, tx, [payer, aliceKp]);
    console.log(`    Alice: ${aliceKp.publicKey.toBase58()}`);
    console.log(`    Tx: https://explorer.solana.com/tx/${sig}?cluster=devnet`);
    results.token.initAlice = { sig, account: aliceKp.publicKey.toBase58() };
  }

  // 1.3 Init Bob Account
  {
    console.log('  1.3 Initializing Bob Account...');
    const createBobIx = SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: bobKp.publicKey,
      lamports: rentAccount,
      space: ACCOUNT_SIZE,
      programId: TOKEN_PROGRAM_ID
    });

    const initBobIx = new TransactionInstruction({
      programId: TOKEN_PROGRAM_ID,
      keys: [
        { pubkey: bobKp.publicKey, isSigner: false, isWritable: true },
        { pubkey: mintKp.publicKey, isSigner: false, isWritable: false },
        { pubkey: payer.publicKey, isSigner: false, isWritable: false }
      ],
      data: Buffer.from([1])
    });

    const tx = new Transaction().add(createBobIx, initBobIx);
    const sig = await sendAndConfirmTransaction(connection, tx, [payer, bobKp]);
    console.log(`    Bob: ${bobKp.publicKey.toBase58()}`);
    console.log(`    Tx: https://explorer.solana.com/tx/${sig}?cluster=devnet`);
    results.token.initBob = { sig, account: bobKp.publicKey.toBase58() };
  }

  // 1.4 MintTo Alice
  {
    console.log('  1.4 Minting 1,000,000 tokens to Alice...');
    const mintData = Buffer.alloc(9);
    mintData.writeUInt8(2, 0);
    mintData.writeBigUInt64LE(1000000n, 1);

    const mintIx = new TransactionInstruction({
      programId: TOKEN_PROGRAM_ID,
      keys: [
        { pubkey: mintKp.publicKey, isSigner: false, isWritable: true },
        { pubkey: aliceKp.publicKey, isSigner: false, isWritable: true },
        { pubkey: payer.publicKey, isSigner: true, isWritable: false }
      ],
      data: mintData
    });

    const tx = new Transaction().add(mintIx);
    const sig = await sendAndConfirmTransaction(connection, tx, [payer]);
    console.log(`    Tx: https://explorer.solana.com/tx/${sig}?cluster=devnet`);
    results.token.mintTo = { sig, amount: 1000000 };
  }

  // 1.5 Transfer Alice -> Bob: 500,000 tokens (1% fee = 5,000)
  {
    console.log('  1.5 Transferring 500,000 tokens Alice -> Bob (1% fee)...');
    const transferData = Buffer.alloc(9);
    transferData.writeUInt8(3, 0);
    transferData.writeBigUInt64LE(500000n, 1);

    const transferIx = new TransactionInstruction({
      programId: TOKEN_PROGRAM_ID,
      keys: [
        { pubkey: aliceKp.publicKey, isSigner: false, isWritable: true },
        { pubkey: bobKp.publicKey, isSigner: false, isWritable: true },
        { pubkey: mintKp.publicKey, isSigner: false, isWritable: false },
        { pubkey: payer.publicKey, isSigner: true, isWritable: false }
      ],
      data: transferData
    });

    const tx = new Transaction().add(transferIx);
    const sig = await sendAndConfirmTransaction(connection, tx, [payer]);
    console.log(`    Tx: https://explorer.solana.com/tx/${sig}?cluster=devnet`);
    results.token.transfer = { sig, amount: 500000, expectedFee: 5000, expectedReceived: 495000 };
  }

  // 2. TEST CUSTOM SYSCLOCK ON DEVNET
  console.log('\n>>> [2/3] Testing CustomSysclock on Devnet...');
  const clockKp = await Keypair.generate();
  const CLOCK_STATE_SIZE = 72;
  const rentClock = await connection.getMinimumBalanceForRentExemption(CLOCK_STATE_SIZE);

  {
    console.log('  2.1 Creating and Initializing Sysclock account...');
    const createClockIx = SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: clockKp.publicKey,
      lamports: rentClock,
      space: CLOCK_STATE_SIZE,
      programId: SYSCLOCK_PROGRAM_ID
    });

    const initClockIx = new TransactionInstruction({
      programId: SYSCLOCK_PROGRAM_ID,
      keys: [
        { pubkey: clockKp.publicKey, isSigner: false, isWritable: true },
        { pubkey: payer.publicKey, isSigner: true, isWritable: false }
      ],
      data: Buffer.from([0])
    });

    const tx = new Transaction().add(createClockIx, initClockIx);
    const sig = await sendAndConfirmTransaction(connection, tx, [payer, clockKp]);
    console.log(`    Sysclock: ${clockKp.publicKey.toBase58()}`);
    console.log(`    Tx: https://explorer.solana.com/tx/${sig}?cluster=devnet`);
    results.sysclock.init = { sig, account: clockKp.publicKey.toBase58() };
  }

  {
    console.log('  2.2 Ticking Oracle: slot=525743400, timestamp=1732415000000, epoch=advance...');
    const tickData = Buffer.alloc(18);
    tickData.writeUInt8(1, 0);
    tickData.writeBigUInt64LE(525743400n, 1);
    tickData.writeBigInt64LE(1732415000000n, 9);
    tickData.writeUInt8(1, 17);

    const tickIx = new TransactionInstruction({
      programId: SYSCLOCK_PROGRAM_ID,
      keys: [
        { pubkey: clockKp.publicKey, isSigner: false, isWritable: true },
        { pubkey: payer.publicKey, isSigner: true, isWritable: false }
      ],
      data: tickData
    });

    const tx = new Transaction().add(tickIx);
    const sig = await sendAndConfirmTransaction(connection, tx, [payer]);
    console.log(`    Tx: https://explorer.solana.com/tx/${sig}?cluster=devnet`);
    results.sysclock.tick = { sig };
  }

  // 3. TEST CUSTOM RAM ON DEVNET
  console.log('\n>>> [3/3] Testing CustomRAM on Devnet...');
  const ramKp = await Keypair.generate();
  const RAM_TOTAL_SIZE = 296; // 40 header + 256 data
  const rentRam = await connection.getMinimumBalanceForRentExemption(RAM_TOTAL_SIZE);

  {
    console.log('  3.1 Initializing RAM account (296 bytes)...');
    const createRamIx = SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: ramKp.publicKey,
      lamports: rentRam,
      space: RAM_TOTAL_SIZE,
      programId: RAM_PROGRAM_ID
    });

    const initRamIx = new TransactionInstruction({
      programId: RAM_PROGRAM_ID,
      keys: [
        { pubkey: ramKp.publicKey, isSigner: false, isWritable: true },
        { pubkey: payer.publicKey, isSigner: true, isWritable: false }
      ],
      data: Buffer.from([0])
    });

    const tx = new Transaction().add(createRamIx, initRamIx);
    const sig = await sendAndConfirmTransaction(connection, tx, [payer, ramKp]);
    console.log(`    RAM: ${ramKp.publicKey.toBase58()}`);
    console.log(`    Tx: https://explorer.solana.com/tx/${sig}?cluster=devnet`);
    results.ram.init = { sig, account: ramKp.publicKey.toBase58() };
  }

  {
    const payloadStr = 'DEEPMIND_ANTIGRAVITY_CUSTOM_RAM_V1_VERIFIED_DEVNET';
    const payload = Buffer.from(payloadStr, 'utf8');
    console.log(`  3.2 Writing to RAM at offset 16 ("${payloadStr}")...`);

    const writeData = Buffer.alloc(5 + payload.length);
    writeData.writeUInt8(1, 0);
    writeData.writeUInt32LE(16, 1);
    payload.copy(writeData, 5);

    const writeIx = new TransactionInstruction({
      programId: RAM_PROGRAM_ID,
      keys: [
        { pubkey: ramKp.publicKey, isSigner: false, isWritable: true },
        { pubkey: payer.publicKey, isSigner: true, isWritable: false }
      ],
      data: writeData
    });

    const tx = new Transaction().add(writeIx);
    const sig = await sendAndConfirmTransaction(connection, tx, [payer]);
    console.log(`    Tx: https://explorer.solana.com/tx/${sig}?cluster=devnet`);
    results.ram.write = { sig, offset: 16, payload: payloadStr };
  }

  fs.writeFileSync(
    '/home/joe/programs_analysis/devnet_transactions_summary.json',
    JSON.stringify(results, null, 2)
  );

  console.log('\n=============================================================');
  console.log('🎉 ALL LIVE DEVNET TRANSACTIONS SUCCEEDED ON-CHAIN!');
  console.log('Results written to: /home/joe/programs_analysis/devnet_transactions_summary.json');
  console.log('=============================================================');
}

main().catch(err => {
  console.error('DEVNET RUN ERROR:', err);
  process.exit(1);
});
