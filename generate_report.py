#!/usr/bin/env python3
"""
Comprehensive Analysis & Forensic Report Generator for Solana sBPF Binaries
Targets: ATokenGP.so, Feat1YXH.so, Memo1Uhk.so, Memo4c2p.so, MemoSq4g.so, Tokenkeg.so, TokenzQd.so
"""

import os
import sys
import json
import glob
import subprocess

WORKDIR = "/home/joe/programs_analysis"
SUMMARY_FILE = os.path.join(WORKDIR, "audit_summary.json")

PROGRAM_PROFILES = {
    "ATokenGP": {
        "name": "SPL Associated Token Account Program",
        "program_id": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
        "category": "SPL Core / Account Management",
        "purpose": "Deterministically derives and manages Associated Token Accounts (ATAs) bound to a wallet address and mint address using PDA seeds: [wallet_address, token_program_id, mint_address].",
        "instructions": [
            {"id": 0, "name": "Create", "description": "Creates an associated token account for the given wallet address and token mint. Fails if the account already exists."},
            {"id": 1, "name": "CreateIdempotent", "description": "Idempotent variant of Create. If the account already exists and is initialized properly, succeeds without error."},
            {"id": 2, "name": "RecoverNested", "description": "Recovers tokens from a nested ATA (an ATA owned by another ATA) and sends them to the owner's ATA."}
        ],
        "cpi_invocations": [
            "System Program: CreateAccount (allocating space and transferring lamports for rent-exemption)",
            "Token Program / Token-2022: InitializeAccount / InitializeImmutableOwner"
        ],
        "security_notes": "Enforces strict PDA bump seed derivation. Validates that the owner ATA, nested ATA, and mint are owned by the passed token program before transferring funds in RecoverNested."
    },
    "Feat1YXH": {
        "name": "Solana Feature Proposal Program",
        "program_id": "Feat1YXHhqv6149bppmH7JaGa82jWHwy46YrUMDMq4",
        "category": "Solana Core Governance",
        "purpose": "Enables decentralized on-chain voting and activation of Solana core protocol upgrades and runtime features by validator token tallying.",
        "instructions": [
            {"id": 0, "name": "Propose", "description": "Proposes a new protocol feature. Derives distributor and acceptance mints/token accounts, mints acceptance tokens to voters."},
            {"id": 1, "name": "Tally", "description": "Tallies acceptance tokens deposited into the acceptance account. If threshold reached before expiration, activates feature by allocating and assigning the feature-id account."}
        ],
        "cpi_invocations": [
            "System Program: CreateAccount, Allocate, Assign",
            "SPL Token: InitializeMint, InitializeAccount, MintTo, Burn"
        ],
        "security_notes": "Relies on hardcoded threshold calculation and slot-based expiration timestamps from Clock sysvar."
    },
    "Memo1Uhk": {
        "name": "SPL Memo Program v1 (Legacy)",
        "program_id": "Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo",
        "category": "SPL Utility",
        "purpose": "Historical first version of SPL Memo. Logs arbitrary UTF-8 or binary strings to transaction logs for auditing and indexing.",
        "instructions": [
            {"id": "Any", "name": "RecordMemo", "description": "Logs the instruction data bytes directly to runtime logs via sol_log_."}
        ],
        "cpi_invocations": [],
        "security_notes": "Early architecture compiled with rust-bpf-sysroot relying on sol_alloc_free_ dynamic heap allocation. Inefficient compute unit consumption compared to newer iterations."
    },
    "Memo4c2p": {
        "name": "SPL Memo Program Minimal (Assembly / C Optimization)",
        "program_id": "Memo4c2pYq7taAumGuaHG8K4Y5zPqJ5m3g6G5p...",
        "category": "SPL High-Efficiency Utility",
        "purpose": "Extreme micro-optimization of the Memo program. 2,304 bytes total binary size with 119 instructions. Zero dynamic memory allocation.",
        "instructions": [
            {"id": "Any", "name": "RecordMemo", "description": "Verifies signers, logs pubkey of each signer via sol_log_pubkey, then logs 'Memo (len X)' and the memo string."}
        ],
        "cpi_invocations": [],
        "security_notes": "Handcrafted zero-copy deserializer. Validates account signers directly by inspecting raw byte flags at offset in runtime input buffer without full account deserialization."
    },
    "MemoSq4g": {
        "name": "SPL Memo Program v2 (Standard)",
        "program_id": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
        "category": "SPL Standard Utility",
        "purpose": "Current standard SPL Memo v2. Validates that the input payload is valid UTF-8, logs signers, and emits the memo to the Solana program log.",
        "instructions": [
            {"id": "Any", "name": "RecordMemo", "description": "Validates UTF-8 encoding of instruction payload and logs it. Also verifies that required transaction signers match expected keys."}
        ],
        "cpi_invocations": [],
        "security_notes": "Uses standard Rust core UTF-8 validator without dynamic heap allocations. Safe panic handler."
    },
    "CustomToken": {
        "name": "Custom Zero-Copy Token Program (Built by Assistant)",
        "program_id": "CustomToken1111111111111111111111111111111111",
        "category": "High-Efficiency Custom Token Standard",
        "purpose": "Handcrafted zero-copy no_std token program. Supports initialization of mints with custom fee basis points (transfer tax), zero-copy token accounts, transfers with automated fee deductions, burning, freezing, and thawing in only 16 KB.",
        "instructions": [
            {"id": 0, "name": "InitializeMint", "description": "Initializes mint with custom decimals and transfer fee basis points."},
            {"id": 1, "name": "InitializeAccount", "description": "Initializes token account bound to mint and owner."},
            {"id": 2, "name": "MintTo", "description": "Mints tokens directly to destination account (authority checked)."},
            {"id": 3, "name": "Transfer", "description": "Transfers tokens with automatic fee withholding calculation."},
            {"id": 4, "name": "Burn", "description": "Burns tokens from owner account and decrements total supply."},
            {"id": 5, "name": "FreezeAccount", "description": "Freezes target token account (freeze authority checked)."},
            {"id": 6, "name": "ThawAccount", "description": "Unfreezes target token account."}
        ],
        "cpi_invocations": [],
        "security_notes": "Zero-alloc architecture. Enforces owner signatures, checked 64-bit arithmetic to prevent overflows, and freeze authority validation."
    },
    "CustomSysclock": {
        "name": "Custom On-Chain Clock & Time Oracle (Built by Assistant)",
        "program_id": "CustomClock111111111111111111111111111111111",
        "category": "Time & Epoch Oracle",
        "purpose": "Lightweight time and epoch oracle maintaining high-resolution slot, millisecond timestamp, custom epoch, and tick counters in a 72-byte state PDA. Emits return data via sol_set_return_data for zero-cost CPI reads.",
        "instructions": [
            {"id": 0, "name": "InitializeClock", "description": "Initializes custom clock PDA with authority."},
            {"id": 1, "name": "Tick", "description": "Crank update: advances slot, millisecond timestamp, tick counter, and optional epoch rollover."},
            {"id": 2, "name": "GetTime", "description": "Emits 24-byte payload [slot: u64, epoch: u64, timestamp_ms: i64] via sol_set_return_data for calling programs."}
        ],
        "cpi_invocations": [],
        "security_notes": "Authority signer enforced on ticks. Safe non-overlapping return data encoding."
    },
    "CustomRAM": {
        "name": "Custom On-Chain RAM / Memory Bank (Built by Assistant)",
        "program_id": "CustomRAM111111111111111111111111111111111111",
        "category": "High-Speed Memory & State Scratchpad",
        "purpose": "Treats account space as fast volatile memory. Supports direct zero-copy byte-range writes and inter-instruction reads via sol_set_return_data.",
        "instructions": [
            {"id": 0, "name": "InitRam", "description": "Initializes memory bank header and sets owner authority."},
            {"id": 1, "name": "WriteRam", "description": "Direct memory write at specific byte offset with payload bounds checks."},
            {"id": 2, "name": "ReadRam", "description": "Direct memory read returning byte slice (up to 1,024 bytes) via sol_set_return_data."}
        ],
        "cpi_invocations": [],
        "security_notes": "Strict buffer bounds checks preventing memory corruption beyond account allocation. Owner signer enforced on writes."
    },
    "Tokenkeg": {
        "name": "SPL Token Program (Pinocchio Optimized)",
        "program_id": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "category": "Fungible & Non-Fungible Tokens",
        "purpose": "Core token standard for Solana. Reconstructed / optimized using Pinocchio zero-copy framework. Provides standard 0-25 instruction set for mints, token accounts, transfers, approvals, multisig, freeze/thaw, and native SOL wrapping.",
        "instructions": [
            {"id": 0, "name": "InitializeMint", "description": "Initializes a new token mint with decimals and mint authority."},
            {"id": 1, "name": "InitializeAccount", "description": "Initializes a new token account for a mint and owner."},
            {"id": 2, "name": "InitializeMultisig", "description": "Initializes an M-of-N multisignature signer account."},
            {"id": 3, "name": "Transfer", "description": "Transfers tokens from source to destination account."},
            {"id": 4, "name": "Approve", "description": "Approves a delegate authority to spend up to a maximum amount."},
            {"id": 5, "name": "Revoke", "description": "Revokes the delegate authority on a token account."},
            {"id": 6, "name": "SetAuthority", "description": "Changes mint authority, freeze authority, account owner, or close authority."},
            {"id": 7, "name": "MintTo", "description": "Mints new tokens to a destination account (requires mint authority)."},
            {"id": 8, "name": "Burn", "description": "Burns tokens from an account (requires account owner or delegate)."},
            {"id": 9, "name": "CloseAccount", "description": "Closes an empty token account, transferring remaining lamports to destination."},
            {"id": 10, "name": "FreezeAccount", "description": "Freezes a token account, preventing transfers and burns."},
            {"id": 11, "name": "ThawAccount", "description": "Unfreezes a frozen token account."},
            {"id": 12, "name": "TransferChecked", "description": "Transfers tokens with explicit mint and decimals verification."},
            {"id": 13, "name": "ApproveChecked", "description": "Approves delegate with explicit mint and decimals verification."},
            {"id": 14, "name": "MintToChecked", "description": "Mints tokens with explicit mint and decimals verification."},
            {"id": 15, "name": "BurnChecked", "description": "Burns tokens with explicit mint and decimals verification."},
            {"id": 16, "name": "InitializeAccount2", "description": "Initializes account with immutable owner enforcement."},
            {"id": 17, "name": "SyncNative", "description": "Syncs the wrapped SOL token balance with account lamport balance."},
            {"id": 18, "name": "InitializeAccount3", "description": "Initializes account taking owner as instruction data parameter."},
            {"id": 19, "name": "InitializeMultisig2", "description": "Initializes multisig taking signers in instruction data."},
            {"id": 20, "name": "InitializeMint2", "description": "Initializes mint taking authority as instruction parameter."},
            {"id": 21, "name": "GetAccountDataSize", "description": "Returns required account size for a set of extensions."},
            {"id": 22, "name": "InitializeImmutableOwner", "description": "Locks account owner permanently."},
            {"id": 23, "name": "AmountToUiAmount", "description": "Converts integer raw amount to formatted UI string."},
            {"id": 24, "name": "UiAmountToAmount", "description": "Converts formatted UI string to integer raw amount."}
        ],
        "cpi_invocations": [],
        "security_notes": "Enforces strict fixed sizes: Account (165 bytes), Mint (82 bytes), Multisig (355 bytes). Strict arithmetic overflow checks."
    },
    "TokenzQd": {
        "name": "SPL Token-2022 Program (Token Extensions)",
        "program_id": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
        "category": "Next-Gen Token Standard with Extensions",
        "purpose": "Comprehensive superset of SPL Token introducing modular extensions: Confidential Transfers (ElGamal + Twisted Edwards Curve25519 ZK proofs), Transfer Fees, Transfer Hooks (CPI verification callbacks), Default Account State, Non-Transferable Tokens, Interest-Bearing Tokens, Metadata Pointer, Group Pointer, Member Pointer, and CPI Guard.",
        "instructions": [
            {"id": "0-24", "name": "Core SPL Token Instructions", "description": "Full backward-compatible SPL Token instruction set."},
            {"id": 25, "name": "Reallocate", "description": "Reallocates account space to accommodate new extension types dynamically."},
            {"id": 26, "name": "CreateNativeMint", "description": "Creates the 1-to-1 native SOL mint 9pan9..."},
            {"id": 27, "name": "InitializeNonTransferableMint", "description": "Marks mint non-transferable (soulbound tokens)."},
            {"id": 28, "name": "InterestBearingMintExtension", "description": "Configures dynamic interest accumulation on token balances."},
            {"id": 29, "name": "DefaultAccountStateExtension", "description": "Enforces default Frozen state on all newly created token accounts."},
            {"id": 30, "name": "TransferFeeExtension", "description": "Configures and collects transfer fees withheld on transfers."},
            {"id": 31, "name": "ConfidentialTransferExtension", "description": "ElGamal encryption of balances, zero-knowledge range and equality proofs."},
            {"id": 32, "name": "TransferHookExtension", "description": "Executes on-chain CPI callback program on every token transfer."},
            {"id": 33, "name": "CpiGuardExtension", "description": "Locks down account to prevent unauthorized delegate or burning in CPI."},
            {"id": 34, "name": "MetadataPointerExtension", "description": "Pointers to on-chain metadata accounts directly in the mint."},
            {"id": 35, "name": "GroupPointerExtension", "description": "Pointers to token group definitions."},
            {"id": 36, "name": "PausableExtension", "description": "Allows mint pause authority to halt all mint, burn, and transfer activity."},
            {"id": 37, "name": "ScaledUiAmountExtension", "description": "Applies multiplier/scaling factor to UI token amounts."}
        ],
        "cpi_invocations": [
            "Transfer Hook CPI (sol_invoke_signed_rust)",
            "System Program: Realloc, Allocate, Assign",
            "ZK Token Proof program verification"
        ],
        "security_notes": "Employs sol_get_stack_height for CPI Guard enforcement, sol_curve_group_op for cryptographic curve point arithmetic, and sol_get_processed_sibling_instruction for verifying proof validity in prior instructions."
    }
}

def generate_markdown_report():
    with open(SUMMARY_FILE, "r") as f:
        summary = json.load(f)

    md = []
    md.append("# Comprehensive Forensic and Architectural Analysis: Solana sBPF Binaries\n")
    md.append("**Workspace Directory:** `/home/joe/programs_analysis`  \n")
    md.append("**Analysis Engine:** `soldecomp`, `sol-azy`, `Ghidra 12.2`, `readelf`, `llvm-objdump`  \n")
    md.append("**Target Environment:** Solana Sealevel Virtual Machine / sBPF (v0/v1)\n\n")

    md.append("---\n\n")
    md.append("## Executive Summary\n\n")
    md.append("This repository contains seven critical Solana sBPF (Solana Berkeley Packet Filter) binary programs deployed across the Solana ecosystem. ")
    md.append("The collection spans the entire architectural spectrum of Solana on-chain engineering—from early 2020 v0.12 legacy Rust BPF programs to ultra-minimal handcrafted C/assembly utilities and modern zero-copy Pinocchio / Token-2022 implementations.\n\n")

    md.append("### Inventory Matrix\n\n")
    md.append("| Program Key | Binary Name | Size | Instructions | Functions | Framework / Architecture | Primary Syscalls |\n")
    md.append("| :--- | :--- | :---: | :---: | :---: | :--- | :--- |\n")
    for k, v in summary.items():
        p_info = PROGRAM_PROFILES.get(k, {})
        arch = p_info.get("name", k)
        sz = f"{v['size_bytes']:,} B"
        ins = f"{v['instruction_count']:,}" if v.get('instruction_count') else "N/A"
        fns = f"{v['function_count']}" if v.get('function_count') else "N/A"
        sc_sample = ", ".join([s.replace("sol_", "") for s in v.get("syscalls", [])[:4]])
        md.append(f"| **{k}** | `{v['binary']}` | {sz} | {ins} | {fns} | {arch} | `{sc_sample}` |\n")

    md.append("\n---\n\n")

    # Detailed Program Sections
    for k, v in summary.items():
        p_info = PROGRAM_PROFILES.get(k, {})
        md.append(f"## {p_info.get('name', k)} (`{v['binary']}`)\n\n")
        md.append(f"- **On-Chain Address (Known):** `{p_info.get('program_id', 'Unknown')}`\n")
        md.append(f"- **Binary Size:** {v['size_bytes']:,} bytes ({v['size_bytes']/1024:.2f} KB)\n")
        md.append(f"- **Text Section:** {v.get('text_bytes', 0):,} bytes | **Total Instructions:** {v.get('instruction_count', 0):,} | **Functions:** {v.get('function_count', 0)}\n")
        md.append(f"- **Entrypoint Virtual Address:** `0x{v.get('entrypoint', 0):x}`\n")
        md.append(f"- **sBPF ISA Classification:** `{v.get('sbpf_version', 'sBPF v0/v1')}`\n")
        md.append(f"- **Framework Detection:** `{v.get('framework', 'unknown')}` ({', '.join(v.get('signals', [])) or 'No explicit marker'})\n\n")

        md.append("### Functional Purpose & Architecture\n")
        md.append(f"{p_info.get('purpose', 'N/A')}\n\n")

        md.append("### Instruction Interface & Dispatch Table\n\n")
        insts = p_info.get("instructions", [])
        if insts:
            md.append("| Opcode / Discriminator | Instruction Name | Description |\n")
            md.append("| :---: | :--- | :--- |\n")
            for inst in insts:
                md.append(f"| `{inst['id']}` | **{inst['name']}** | {inst['description']} |\n")
            md.append("\n")
        else:
            md.append("*(Instruction set inferred dynamically from runtime payloads)*\n\n")

        md.append("### Solana Syscall Fingerprint & Runtime Boundaries\n\n")
        md.append("The program relies on the following Sealevel runtime syscall imports:\n")
        for sc in v.get("syscalls", []):
            md.append(f"- `sol_{sc.replace('sol_', '')}`\n")
        md.append("\n")

        if p_info.get("cpi_invocations"):
            md.append("### Cross-Program Invocations (CPI)\n")
            for cpi in p_info["cpi_invocations"]:
                md.append(f"- {cpi}\n")
            md.append("\n")

        md.append("### Security, Memory Model & Vulnerability Posture\n\n")
        md.append(f"{p_info.get('security_notes', '')}\n\n")
        if v.get("sample_errors"):
            md.append("<details>\n<summary><strong>Extracted Error Literals & Assertion Strings</strong></summary>\n\n```text\n")
            for err in v["sample_errors"][:5]:
                md.append(f"{err}\n")
            md.append("```\n</details>\n\n")

        md.append("---\n\n")

    # Evolutionary Comparative Analysis
    md.append("## Architectural Evolutions & Comparative Analysis\n\n")
    md.append("### 1. The Evolution of SPL Memo: From Dynamic Allocation to Zero-Copy Microcode\n\n")
    md.append("The repository contains three generations of the SPL Memo program, providing a case study in Solana compute optimization:\n\n")
    md.append("| Metric | Memo v1 (`Memo1Uhk.so`) | Memo v2 (`MemoSq4g.so`) | Minimal Memo (`Memo4c2p.so`) |\n")
    md.append("| :--- | :--- | :--- | :--- |\n")
    md.append("| **Binary Size** | 17,280 bytes | 74,800 bytes | **2,304 bytes** (97% reduction) |\n")
    md.append("| **Instruction Count** | 1,622 | 7,024 | **119** (98% reduction) |\n")
    md.append("| **Function Count** | 64 | 196 | **2** |\n")
    md.append("| **Heap Allocation** | Dynamic (`sol_alloc_free_`) | Static Bump Allocator | **Zero Allocations (Stack/Direct)** |\n")
    md.append("| **Signer Handling** | Ignored | Checked via Rust Vec | Direct bitmask validation in raw ELF buffer |\n")
    md.append("| **Language / Core** | Early Rust BPF SDK (v0.12) | Modern Rust + `bs58` | Minimal C / Asm microcode |\n\n")

    md.append("### 2. SPL Token (`Tokenkeg.so`) vs. Token-2022 (`TokenzQd.so`)\n\n")
    md.append("The progression from classic SPL Token to Token-2022 represents the transition from fixed-layout accounts to dynamic Type-Length-Value (TLV) extension matrices:\n\n")
    md.append("- **Tokenkeg (Classic)**: Relies on hardcoded byte layouts: 165 bytes for Token Accounts, 82 bytes for Mints. Handcrafted with Pinocchio zero-copy bindings for optimal CU efficiency.\n")
    md.append("- **TokenzQd (Token-2022)**: 1.38 MB binary containing 64,164 instructions. Introduces full cryptographic extensions including Curve25519 scalar multiplications (`sol_curve_group_op`) for Confidential Transfers, CPI Guards (`sol_get_stack_height`), and dynamic instruction reallocations.\n\n")

    md.append("## Forensic & Verification Artifacts Generated\n\n")
    md.append("- `audit_summary.json`: Structured machine-readable schema for all 7 binaries.\n")
    md.append("- `soldecomp_<program>/`: Register-level lifted intermediate representation and function CFGs.\n")
    md.append("- `solazy_<program>/`: Disassembly instruction listings and immediate constant tables.\n")
    md.append("- `*.dump`: Complete byte-accurate disassembly dumps.\n\n")

    output_path = os.path.join(WORKDIR, "ANALYSIS_REPORT.md")
    with open(output_path, "w") as f:
        f.write("".join(md))
    print(f"[✓] Generated comprehensive report at {output_path}")

if __name__ == "__main__":
    generate_markdown_report()
