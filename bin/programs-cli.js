#!/usr/bin/env node
// SPDX-License-Identifier: PACARL-1.0
// Copyright (c) 2026 praisethedevel <xsolutionx@gmail.com>. All Rights Reserved.

const path = require('path');
module.paths.push('/home/joe/projects/solforge-studio/node_modules');
module.paths.push(path.join(__dirname, '../node_modules'));

const fs = require('fs');
const { Connection, PublicKey } = require('@solana/web3.js');
const { CustomTokenClient, CustomSysclockClient, CustomRAMClient } = require('../sdk/dist/index.js');

const RPC_URL = process.env.SOLANA_RPC_URL || 'https://api.devnet.solana.com';
const connection = new Connection(RPC_URL, 'confirmed');

const tokenClient = new CustomTokenClient(connection);
const sysclockClient = new CustomSysclockClient(connection);
const ramClient = new CustomRAMClient(connection);

function printHelp() {
  console.log(`
PRAISETHEDEVEL SBPF ZERO-COPY CLI
Governed by PACARL-1.0 (c) 2026 praisethedevel <xsolutionx@gmail.com>

Usage:
  programs-cli status                      Show deployed program IDs and cluster health
  programs-cli cu-report                   Display verified compute unit benchmarks
  programs-cli token read-mint <PUBKEY>    Inspect zero-copy Mint state
  programs-cli token read-acct <PUBKEY>    Inspect zero-copy Token Account state
  programs-cli clock read <PUBKEY>         Query on-chain Oracle Clock state
  programs-cli ram read <PUBKEY> [offset] [len]  Inspect on-chain RAM bank memory
`);
}

async function main() {
  const args = process.argv.slice(2);
  const cmd = args[0];

  if (!cmd || cmd === 'help' || cmd === '--help' || cmd === '-h') {
    printHelp();
    return;
  }

  if (cmd === 'status') {
    const manifest = JSON.parse(fs.readFileSync(path.join(__dirname, '../verified_build_manifest.json'), 'utf8'));
    console.log('=== SOLANA SBPF CLUSTER STATUS ===');
    console.log(`Cluster:     Devnet (${RPC_URL})`);
    console.log(`Deployer:    ${manifest.cluster_deployments.devnet.deployer}`);
    console.log(`Status:      ${manifest.cluster_deployments.devnet.status}\n`);
    for (const prog of manifest.programs) {
      console.log(`[*] ${prog.name}:`);
      console.log(`    Program ID:   ${prog.program_id}`);
      console.log(`    Size:         ${prog.size_bytes} bytes`);
      console.log(`    SHA256:       ${prog.sha256}`);
      console.log(`    Verified:     ${prog.verification_status}`);
      console.log(`    Devnet URL:   ${prog.devnet.explorer_url}\n`);
    }
    return;
  }

  if (cmd === 'cu-report') {
    const benchPath = path.join(__dirname, '../BENCHMARKS.md');
    console.log(fs.readFileSync(benchPath, 'utf8'));
    return;
  }

  if (cmd === 'token') {
    const sub = args[1];
    const pubkeyStr = args[2];
    if (!sub || !pubkeyStr) {
      printHelp();
      return;
    }
    const pubkey = new PublicKey(pubkeyStr);

    if (sub === 'read-mint') {
      const mint = await tokenClient.fetchMint(pubkey);
      if (!mint) {
        console.error(`[-] Could not find or parse Mint at ${pubkeyStr}`);
        return;
      }
      console.log('=== ZERO-COPY MINT STATE ===');
      console.log(`Mint Authority:   ${mint.mintAuthority.toBase58()}`);
      console.log(`Total Supply:     ${mint.supply.toString()}`);
      console.log(`Decimals:         ${mint.decimals}`);
      console.log(`Is Initialized:   ${mint.isInitialized}`);
      console.log(`Fee Basis Points: ${mint.feeBasisPoints} (${(mint.feeBasisPoints / 100).toFixed(2)}%)`);
      console.log(`Freeze Authority: ${mint.freezeAuthority.toBase58()}`);
      return;
    }

    if (sub === 'read-acct') {
      const acct = await tokenClient.fetchAccount(pubkey);
      if (!acct) {
        console.error(`[-] Could not find or parse Token Account at ${pubkeyStr}`);
        return;
      }
      console.log('=== ZERO-COPY TOKEN ACCOUNT STATE ===');
      console.log(`Mint:             ${acct.mint.toBase58()}`);
      console.log(`Owner:            ${acct.owner.toBase58()}`);
      console.log(`Amount:           ${acct.amount.toString()}`);
      console.log(`State:            ${acct.state === 1 ? 'Initialized' : acct.state === 2 ? 'Frozen' : 'Uninitialized'}`);
      return;
    }
  }

  if (cmd === 'clock' && args[1] === 'read') {
    const pubkeyStr = args[2];
    if (!pubkeyStr) {
      printHelp();
      return;
    }
    const pubkey = new PublicKey(pubkeyStr);
    const clock = await sysclockClient.fetchClock(pubkey);
    if (!clock) {
      console.error(`[-] Could not find or parse Sysclock at ${pubkeyStr}`);
      return;
    }
    console.log('=== ZERO-COPY SYSCLOCK ORACLE STATE ===');
    console.log(`Is Initialized:   ${clock.isInitialized}`);
    console.log(`Is Frozen:        ${clock.isFrozen}`);
    console.log(`Slot:             ${clock.slot.toString()}`);
    console.log(`Epoch:            ${clock.epoch.toString()}`);
    console.log(`Timestamp (ms):   ${clock.unixTimestampMs.toString()} (${new Date(Number(clock.unixTimestampMs)).toISOString()})`);
    console.log(`Tick Sequence:    ${clock.tickSequence.toString()}`);
    console.log(`Authority:        ${clock.authority.toBase58()}`);
    return;
  }

  if (cmd === 'ram' && args[1] === 'read') {
    const pubkeyStr = args[2];
    const offset = parseInt(args[3] || '0', 10);
    const len = parseInt(args[4] || '64', 10);
    if (!pubkeyStr) {
      printHelp();
      return;
    }
    const pubkey = new PublicKey(pubkeyStr);
    const header = await ramClient.fetchRamHeader(pubkey);
    if (!header) {
      console.error(`[-] Could not find or parse RAM at ${pubkeyStr}`);
      return;
    }
    console.log('=== ZERO-COPY RAM BANK HEADER ===');
    console.log(`Is Initialized:   ${header.isInitialized}`);
    console.log(`Is Locked:        ${header.isLocked}`);
    console.log(`Allocated Size:   ${header.allocatedSize} bytes`);
    console.log(`Owner:            ${header.owner.toBase58()}`);

    const mem = await ramClient.readMemoryRange(pubkey, offset, len);
    if (mem) {
      console.log(`\nMemory Slice [+${offset}..+${offset + len}]:`);
      console.log(`Hex:  ${mem.toString('hex')}`);
      console.log(`Text: "${mem.toString('utf8').replace(/[^\x20-\x7E]/g, '.')}"`);
    }
    return;
  }

  printHelp();
}

main().catch(err => {
  console.error('CLI Error:', err);
  process.exit(1);
});
