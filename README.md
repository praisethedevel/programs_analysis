# Solana High-Performance Zero-Copy sBPF Program Suite

[![Build & Verify](https://github.com/praisethedevel/programs_analysis/actions/workflows/verify.yml/badge.svg)](https://github.com/praisethedevel/programs_analysis/actions)
[![License: PACARL-1.0](https://img.shields.io/badge/License-PACARL--1.0-red.svg)](https://github.com/praisethedevel/programs_analysis/blob/main/LICENSE)
[![Neodyme security.txt](https://img.shields.io/badge/security.txt-Neodyme%20v1%20Compliant-brightgreen)](https://github.com/praisethedevel/programs_analysis/blob/main/SECURITY.md)
[![Solana Devnet](https://img.shields.io/badge/Solana-Devnet%20Verified-blue)](https://explorer.solana.com/address/CShaS9ezrJoEYBXsAk4bsUzL2pbxE14kAadgBCeXS7Z1?cluster=devnet)

A suite of production-grade, zero-copy, memory-safe Solana sBPF on-chain programs engineered without runtime overhead. Built in Rust with `#![no_std]` support on-chain, Neodyme standard `security.txt` headers embedded in `.rodata`, deterministic byte-for-byte reproducible builds, and verified deployments on Solana Devnet.

---

## 🚀 Live Solana Devnet Deployments

| Program | Program ID | Byte Size | Devnet Explorer |
|---|---|---|---|
| **CustomToken** | `CShaS9ezrJoEYBXsAk4bsUzL2pbxE14kAadgBCeXS7Z1` | 17,024 B | [Explorer Link](https://explorer.solana.com/address/CShaS9ezrJoEYBXsAk4bsUzL2pbxE14kAadgBCeXS7Z1?cluster=devnet) |
| **CustomSysclock** | `8VB675MfjjiaizM2VDBRQfk8GMKCiZVtMELqGdstgMTK` | 8,416 B | [Explorer Link](https://explorer.solana.com/address/8VB675MfjjiaizM2VDBRQfk8GMKCiZVtMELqGdstgMTK?cluster=devnet) |
| **CustomRAM** | `5oFB4c7Answx5c2reA9w5u3DGcE1HHw2PfhkXTRFvc3m` | 8,488 B | [Explorer Link](https://explorer.solana.com/address/5oFB4c7Answx5c2reA9w5u3DGcE1HHw2PfhkXTRFvc3m?cluster=devnet) |

- **Deployer / Upgrade Authority**: `DeoN4qmakKDMfGHJ54FzQ4AGBXmNcfZRyuA5uvemsKWP`
- **Security Contact**: `xsolutionx@gmail.com` ([Neodyme RFC 9116](https://github.com/praisethedevel/programs_analysis/blob/main/security.txt))

---

## 📦 Programs Overview

### 1. `CustomToken` (Zero-Copy Token with Fee Withholding)
A minimal, zero-copy token program supporting built-in transfer fee basis points without dynamic allocations:
- **Mint Layout (`80 bytes`)**: `authority (32B) + supply (8B) + decimals (1B) + is_initialized (1B) + fee_basis_points (2B) + freeze_authority (32B) + padding (4B)`
- **Account Layout (`80 bytes`)**: `mint (32B) + owner (32B) + amount (8B) + state (1B) + padding (7B)`
- **Supported Instructions**:
  - `0`: `InitializeMint(decimals: u8, fee_bps: u16)`
  - `1`: `InitializeAccount()`
  - `2`: `MintTo(amount: u64)`
  - `3`: `Transfer(amount: u64)` (automatically computes and withholds fee)
  - `4`: `Burn(amount: u64)`
  - `5`: `FreezeAccount()`
  - `6`: `ThawAccount()`

### 2. `CustomSysclock` (On-Chain Time & Slot Oracle)
A lightweight clock oracle providing verified time, slot, and epoch tracking with CPI return data support via `sol_set_return_data`:
- **State Layout (`72 bytes`)**: `is_initialized (1B) + is_frozen (1B) + padding (6B) + slot (8B) + epoch (8B) + unix_timestamp_ms (8B) + tick_sequence (8B) + authority (32B)`
- **Supported Instructions**:
  - `0`: `InitializeClock()`
  - `1`: `Tick(slot: u64, unix_timestamp_ms: i64, advance_epoch: u8)`
  - `2`: `ReadClock()` (sets return data for cross-program callers)

### 3. `CustomRAM` (Direct Byte-Range Memory Bank)
A volatile, high-speed on-chain memory buffer allowing atomic arbitrary-offset writes:
- **Header Layout (`40 bytes`)**: `is_initialized (1B) + is_locked (1B) + padding (2B) + allocated_size (4B) + owner (32B)` followed by contiguous memory.
- **Supported Instructions**:
  - `0`: `InitRam()`
  - `1`: `WriteRam(offset: u32, data: &[u8])`
  - `2`: `LockRam()` (permanently freezes memory writes)

---

## 🔒 Security & `security.txt` Specification

Every compiled binary embeds an RFC 9116-compliant Neodyme `SECURITY.TXT` v1 record into the `.rodata` ELF segment, ensuring security researchers can identify the program maintainer directly from raw bytecode on-chain:

```
name: CustomToken (Zero-Copy SPL Token Rebuild)
project_url: https://github.com/praisethedevel/programs_analysis
contacts: email:xsolutionx@gmail.com,link:https://github.com/praisethedevel/programs_analysis/security
policy: https://github.com/praisethedevel/programs_analysis/blob/main/SECURITY.md
preferred_languages: en
source_code: https://github.com/praisethedevel/programs_analysis/tree/main/custom_suite/custom_token
source_release: custom-token@v0.1.0
auditors: Autonomous Forensic SBPF Engine
expiry: 2028-01-01
```

Verify the embedded security strings in any binary:
```bash
python3 verify_security_txt.py
```

---

## 🛡️ Deterministic Verified Builds

All three binaries are deterministically reproducible. The verification engine recompiles the crates in the workspace and verifies the SHA256 against [`verified_build_manifest.json`](./verified_build_manifest.json):

```bash
python3 verify_builds.py
```

Output:
```
=================================================================
SOLANA SBPF DETERMINISTIC VERIFIED BUILD ENGINE
=================================================================
[*] Target: CustomToken
    Program ID:     CShaS9ezrJoEYBXsAk4bsUzL2pbxE14kAadgBCeXS7Z1
    SHA256:         fc41af0bca21eace8faaa7a85c65f62f2e29039e50c5961d72b1286fbc012561
    Status:         ✅ VERIFIED_MATCH

[*] Target: CustomSysclock
    Program ID:     8VB675MfjjiaizM2VDBRQfk8GMKCiZVtMELqGdstgMTK
    SHA256:         a94f612d88e88aa3a2e1c891c999bdae94bd73127d9f50ec3348a9c3c52acc83
    Status:         ✅ VERIFIED_MATCH

[*] Target: CustomRAM
    Program ID:     5oFB4c7Answx5c2reA9w5u3DGcE1HHw2PfhkXTRFvc3m
    SHA256:         432a511691d09c53fe8cf7c9b16834878a339d498eecd3fa6a9d42cda1996f5f
    Status:         ✅ VERIFIED_MATCH
```

---

## 🧪 Testing & Fuzzing

### Unit Tests
```bash
cd custom_suite
cargo test
```

### High-Throughput Fuzzer (300,000 Iterations)
The custom pseudorandom fuzz runner executes randomized corrupt payloads, out-of-bounds offsets, uninitialized transitions, and authority permutations:
```bash
cargo run --release -p fuzz_runner
```
Result: **0 crashes across 300,000 iterations @ 2.97M exec/sec**. Telemetry stored in [`fuzz_results.json`](./fuzz_results.json).

### Live Devnet Execution
```bash
node test_devnet_suite.js
```
Full on-chain transaction receipts in [`devnet_transactions_summary.json`](./devnet_transactions_summary.json).

---

## 📜 Program IDLs
Anchor-compatible IDLs for client integration are located in [`idls/`](./idls/):
- [`idls/custom_token.json`](./idls/custom_token.json)
- [`idls/custom_sysclock.json`](./idls/custom_sysclock.json)
- [`idls/custom_ram.json`](./idls/custom_ram.json)

---

## 💻 TypeScript Client SDK

A TypeScript SDK is provided in [`sdk/`](./sdk/) for seamless integration:

```typescript
import { CustomTokenClient, CustomSysclockClient, CustomRAMClient } from '@praisethedevel/programs-suite';
import { Connection, Keypair } from '@solana/web3.js';

const connection = new Connection('https://api.devnet.solana.com');
const tokenClient = new CustomTokenClient(connection);

// Create Mint with 1% transfer fee
const ix = tokenClient.createInitMintInstruction(mintPubkey, payerPubkey, 6, 100);
```

---

## ⚖️ License
Governed by the **Praisethedevel Universal Proprietary Architecture, Compulsory Attribution, and Commercial Royalty License ([PACARL-1.0](https://github.com/praisethedevel/programs_analysis/blob/main/LICENSE))**.

- **Non-Commercial / Academic / Evaluative Use**: Permitted strictly under Section 1, subject to mandatory pre-deployment written notice, positive confirmation from the Author (`xsolutionx@gmail.com`), and full unmitigated attribution.
- **Commercial Exploitation**: Expressly prohibited without a bilateral commercial agreement and mandatory royalty contract.
- **Copyright (c) 2026 praisethedevel (`xsolutionx@gmail.com`). All Rights Reserved.**
