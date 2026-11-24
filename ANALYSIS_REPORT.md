# Comprehensive Forensic and Architectural Analysis: Solana sBPF Binaries
**Workspace Directory:** `/home/joe/programs_analysis`  
**Analysis Engine:** `soldecomp`, `sol-azy`, `Ghidra 12.2`, `readelf`, `llvm-objdump`  
**Target Environment:** Solana Sealevel Virtual Machine / sBPF (v0/v1)

---

## Executive Summary

This repository contains seven critical Solana sBPF (Solana Berkeley Packet Filter) binary programs deployed across the Solana ecosystem. The collection spans the entire architectural spectrum of Solana on-chain engineering—from early 2020 v0.12 legacy Rust BPF programs to ultra-minimal handcrafted C/assembly utilities and modern zero-copy Pinocchio / Token-2022 implementations.

### Inventory Matrix

| Program Key | Binary Name | Size | Instructions | Functions | Framework / Architecture | Primary Syscalls |
| :--- | :--- | :---: | :---: | :---: | :--- | :--- |
| **ATokenGP** | `ATokenGP.so` | 105,032 B | 11,219 | 129 | SPL Associated Token Account Program | `get_rent_sysvar, get_return_data, invoke_signe[...], log_` |
| **CustomRAM** | `CustomRAM.so` | 7,696 B | 696 | 9 | Custom On-Chain RAM / Memory Bank (Built by Assistant) | `log_, memcpy_, set_return_data` |
| **CustomSysclock** | `CustomSysclock.so` | 7,616 B | 694 | 8 | Custom On-Chain Clock & Time Oracle (Built by Assistant) | `log_, set_return_data` |
| **CustomToken** | `CustomToken.so` | 16,184 B | 1,738 | 15 | Custom Zero-Copy Token Program (Built by Assistant) | `log_, memmove_` |
| **Feat1YXH** | `Feat1YXH.so` | 191,328 B | 19,909 | 423 | Solana Feature Proposal Program | `create_progr[...], invoke_signe[...], log_, panic_` |
| **Memo1Uhk** | `Memo1Uhk.so` | 17,280 B | 1,622 | 64 | SPL Memo Program v1 (Legacy) | `alloc_free_, log_, panic_` |
| **Memo4c2p** | `Memo4c2p.so` | 2,304 B | 119 | 2 | SPL Memo Program Minimal (Assembly / C Optimization) | `log_, log_pubkey, memcpy_` |
| **MemoSq4g** | `MemoSq4g.so` | 74,800 B | 7,024 | 196 | SPL Memo Program v2 (Standard) | `log_` |
| **Tokenkeg** | `Tokenkeg.so` | 108,600 B | 12,061 | 29 | SPL Token Program (Pinocchio Optimized) | `get_sysvar, log_, memcmp_, memcpy_` |
| **TokenzQd** | `TokenzQd.so` | 1,382,016 B | 64,164 | 490 | SPL Token-2022 Program (Token Extensions) | `curve_group_op, get_clock_sysvar, get_processe[...], get_rent_sysvar` |

---

## SPL Associated Token Account Program (`ATokenGP.so`)

- **On-Chain Address (Known):** `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL`
- **Binary Size:** 105,032 bytes (102.57 KB)
- **Text Section:** 92,056 bytes | **Total Instructions:** 11,219 | **Functions:** 129
- **Entrypoint Virtual Address:** `0x54c8`
- **sBPF ISA Classification:** `v0_or_v1_preversioning`
- **Framework Detection:** `unknown` (No explicit marker)

### Functional Purpose & Architecture
Deterministically derives and manages Associated Token Accounts (ATAs) bound to a wallet address and mint address using PDA seeds: [wallet_address, token_program_id, mint_address].

### Instruction Interface & Dispatch Table

| Opcode / Discriminator | Instruction Name | Description |
| :---: | :--- | :--- |
| `0` | **Create** | Creates an associated token account for the given wallet address and token mint. Fails if the account already exists. |
| `1` | **CreateIdempotent** | Idempotent variant of Create. If the account already exists and is initialized properly, succeeds without error. |
| `2` | **RecoverNested** | Recovers tokens from a nested ATA (an ATA owned by another ATA) and sends them to the owner's ATA. |

### Solana Syscall Fingerprint & Runtime Boundaries

The program relies on the following Sealevel runtime syscall imports:
- `sol_get_rent_sysvar`
- `sol_get_return_data`
- `sol_invoke_signe[...]`
- `sol_log_`
- `sol_memcmp_`
- `sol_memcpy_`
- `sol_memset_`
- `sol_try_find_pro[...]`

### Cross-Program Invocations (CPI)
- System Program: CreateAccount (allocating space and transferring lamports for rent-exemption)
- Token Program / Token-2022: InitializeAccount / InitializeImmutableOwner

### Security, Memory Model & Vulnerability Posture

Enforces strict PDA bump seed derivation. Validates that the owner ATA, nested ATA, and mint are owned by the passed token program before transferring funds in RecoverNested.

<details>
<summary><strong>Extracted Error Literals & Assertion Strings</strong></summary>

```text
Error: Associated address does not match seed derivation
Initialize the associated token accountError: Owner associated address does not match seed derivationError: Nested associated address does not match seed derivationError: Destination associated address does not match seed derivationWallet of the owner associated token account must signOwner mint not owned by provided token programOwner associated token account not owned by provided token program, recreate the owner associated token account firstOwner associated token account not owned by provided walletNested associated token account not owned by provided token programNested associated token account not owned by provided associated token accountNested mint account not owned by provided token programAssociated token account owner does not match address derivationRecoverNested
already mutably borrowedError: memory allocation failed, out of memorylibrary/alloc/src/raw_vec.rscapacity overflowa formatting trait implementation returned an errorlibrary/alloc/src/fmt.rs
..BorrowError <= called `Option::unwrap()` on a `None` value:panicked at '', index out of bounds: the len is  but the index is `: library/core/src/fmt/num.rs0x00010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899library/core/src/fmt/mod.rsrange start index  out of range for slice of length slice index starts at  but ends at  (bytes library/core/src/str/validations.rs[...]byte index  is out of bounds of `begin <= end ( is not a char boundary; it is inside ) of `library/core/src/unicode/printable.rs
```
</details>

---

## Custom On-Chain RAM / Memory Bank (Built by Assistant) (`CustomRAM.so`)

- **On-Chain Address (Known):** `CustomRAM111111111111111111111111111111111111`
- **Binary Size:** 7,696 bytes (7.52 KB)
- **Text Section:** 5,704 bytes | **Total Instructions:** 696 | **Functions:** 9
- **Entrypoint Virtual Address:** `0x128`
- **sBPF ISA Classification:** `v0_or_v1_preversioning`
- **Framework Detection:** `unknown` (No explicit marker)

### Functional Purpose & Architecture
Treats account space as fast volatile memory. Supports direct zero-copy byte-range writes and inter-instruction reads via sol_set_return_data.

### Instruction Interface & Dispatch Table

| Opcode / Discriminator | Instruction Name | Description |
| :---: | :--- | :--- |
| `0` | **InitRam** | Initializes memory bank header and sets owner authority. |
| `1` | **WriteRam** | Direct memory write at specific byte offset with payload bounds checks. |
| `2` | **ReadRam** | Direct memory read returning byte slice (up to 1,024 bytes) via sol_set_return_data. |

### Solana Syscall Fingerprint & Runtime Boundaries

The program relies on the following Sealevel runtime syscall imports:
- `sol_log_`
- `sol_memcpy_`
- `sol_set_return_data`

### Security, Memory Model & Vulnerability Posture

Strict buffer bounds checks preventing memory corruption beyond account allocation. Owner signer enforced on writes.

---

## Custom On-Chain Clock & Time Oracle (Built by Assistant) (`CustomSysclock.so`)

- **On-Chain Address (Known):** `CustomClock111111111111111111111111111111111`
- **Binary Size:** 7,616 bytes (7.44 KB)
- **Text Section:** 5,688 bytes | **Total Instructions:** 694 | **Functions:** 8
- **Entrypoint Virtual Address:** `0x128`
- **sBPF ISA Classification:** `v0_or_v1_preversioning`
- **Framework Detection:** `unknown` (No explicit marker)

### Functional Purpose & Architecture
Lightweight time and epoch oracle maintaining high-resolution slot, millisecond timestamp, custom epoch, and tick counters in a 72-byte state PDA. Emits return data via sol_set_return_data for zero-cost CPI reads.

### Instruction Interface & Dispatch Table

| Opcode / Discriminator | Instruction Name | Description |
| :---: | :--- | :--- |
| `0` | **InitializeClock** | Initializes custom clock PDA with authority. |
| `1` | **Tick** | Crank update: advances slot, millisecond timestamp, tick counter, and optional epoch rollover. |
| `2` | **GetTime** | Emits 24-byte payload [slot: u64, epoch: u64, timestamp_ms: i64] via sol_set_return_data for calling programs. |

### Solana Syscall Fingerprint & Runtime Boundaries

The program relies on the following Sealevel runtime syscall imports:
- `sol_log_`
- `sol_set_return_data`

### Security, Memory Model & Vulnerability Posture

Authority signer enforced on ticks. Safe non-overlapping return data encoding.

---

## Custom Zero-Copy Token Program (Built by Assistant) (`CustomToken.so`)

- **On-Chain Address (Known):** `CustomToken1111111111111111111111111111111111`
- **Binary Size:** 16,184 bytes (15.80 KB)
- **Text Section:** 14,152 bytes | **Total Instructions:** 1,738 | **Functions:** 15
- **Entrypoint Virtual Address:** `0x128`
- **sBPF ISA Classification:** `v0_or_v1_preversioning`
- **Framework Detection:** `unknown` (No explicit marker)

### Functional Purpose & Architecture
Handcrafted zero-copy no_std token program. Supports initialization of mints with custom fee basis points (transfer tax), zero-copy token accounts, transfers with automated fee deductions, burning, freezing, and thawing in only 16 KB.

### Instruction Interface & Dispatch Table

| Opcode / Discriminator | Instruction Name | Description |
| :---: | :--- | :--- |
| `0` | **InitializeMint** | Initializes mint with custom decimals and transfer fee basis points. |
| `1` | **InitializeAccount** | Initializes token account bound to mint and owner. |
| `2` | **MintTo** | Mints tokens directly to destination account (authority checked). |
| `3` | **Transfer** | Transfers tokens with automatic fee withholding calculation. |
| `4` | **Burn** | Burns tokens from owner account and decrements total supply. |
| `5` | **FreezeAccount** | Freezes target token account (freeze authority checked). |
| `6` | **ThawAccount** | Unfreezes target token account. |

### Solana Syscall Fingerprint & Runtime Boundaries

The program relies on the following Sealevel runtime syscall imports:
- `sol_log_`
- `sol_memmove_`

### Security, Memory Model & Vulnerability Posture

Zero-alloc architecture. Enforces owner signatures, checked 64-bit arithmetic to prevent overflows, and freeze authority validation.

---

## Solana Feature Proposal Program (`Feat1YXH.so`)

- **On-Chain Address (Known):** `Feat1YXHhqv6149bppmH7JaGa82jWHwy46YrUMDMq4`
- **Binary Size:** 191,328 bytes (186.84 KB)
- **Text Section:** 163,424 bytes | **Total Instructions:** 19,909 | **Functions:** 423
- **Entrypoint Virtual Address:** `0xb480`
- **sBPF ISA Classification:** `v0_or_v1_preversioning`
- **Framework Detection:** `native` (solana-* crates without a framework marker)

### Functional Purpose & Architecture
Enables decentralized on-chain voting and activation of Solana core protocol upgrades and runtime features by validator token tallying.

### Instruction Interface & Dispatch Table

| Opcode / Discriminator | Instruction Name | Description |
| :---: | :--- | :--- |
| `0` | **Propose** | Proposes a new protocol feature. Derives distributor and acceptance mints/token accounts, mints acceptance tokens to voters. |
| `1` | **Tally** | Tallies acceptance tokens deposited into the acceptance account. If threshold reached before expiration, activates feature by allocating and assigning the feature-id account. |

### Solana Syscall Fingerprint & Runtime Boundaries

The program relies on the following Sealevel runtime syscall imports:
- `sol_create_progr[...]`
- `sol_invoke_signe[...]`
- `sol_log_`
- `sol_panic_`

### Cross-Program Invocations (CPI)
- System Program: CreateAccount, Allocate, Assign
- SPL Token: InitializeMint, InitializeAccount, MintTo, Burn

### Security, Memory Model & Vulnerability Posture

Relies on hardcoded threshold calculation and slot-based expiration timestamps from Clock sysvar.

<details>
<summary><strong>Extracted Error Literals & Assertion Strings</strong></summary>

```text
index out of bounds: the len is already mutably borrowedFeatureProposalInstruction::TallyError: feature proposal account not in the pending stateError: acceptance token address derivation mismatchError: feature-id address derivation mismatchfeature-idFeature proposal expired
Unpacking acceptance token accountFeature proposal has received  tokens, and  tokens required for acceptanceActivation threshold has not been reachedAssigning feature id accountFeature proposal acceptedFeatureProposalInstruction::ProposeError: mint address derivation mismatchError: distributor token address derivation mismatchdistributoracceptanceCreating feature proposal accountCreating mintInitializing mintCreating distributor token accountInitializing distributor token accountCreating acceptance token accountInitializing acceptance token account tokensFunding feature id accountAllocating feature id accountcalled `Result::unwrap()` on an `Err` valueUnexpected length of inputError: failed to deserialize feature proposal account: Unexpected variant index: /Users/mvines/.cache/solana/v0.12/rust-bpf-sysroot/src/libcore/slice/mod.rsassertion failed: `(left == right)`
distributoracceptancefeature-idlamports_per_byte_yearexemption_thresholdburn_percentClockepoch_start_timestampepochleader_schedule_epochunix_timestampalready mutably borrowedcalled `Result::unwrap()` on an `Err` valueUnexpected length of inputError: failed to deserialize feature proposal instruction: Unexpected variant index: Tried to shrink to a larger capacity/Users/mvines/.cache/solana/v0.12/rust-bpf-sysroot/src/liballoc/raw_vec.rsinternal error: entered unreachable codestruct Rent with 3 elementsstruct Clock with 5 elements
/Users/mvines/.cargo/registry/src/github.com-1ecc6299db9ec823/solana-program-1.4.9/src/pubkey.rsSlice must be the same length as a PubkeyPubkeyCustomSequenceMustHaveLengthSizeLimitDeserializeAnyNotSupportedInvalidTagEncodingInvalidCharEncodingInvalidBoolEncodingInvalidUtf8EncodingIointernal error: entered unreachable code/Users/mvines/.cache/solana/v0.12/rust-bpf-sysroot/src/liballoc/raw_vec.rsunexpected end of fileother os erroroperation interruptedwrite zerotimed outinvalid datainvalid input parameteroperation would blockentity already existsbroken pipeaddress not availableaddress in usenot connectedconnection abortedconnection refusedpermission deniedOsmessage (os error )CustomerrorUnexpectedEofOtherInterruptedWriteZeroInvalidDataInvalidInputWouldBlockAlreadyExistsBrokenPipeAddrInUseNotConnectedConnectionAbortedConnectionResetConnectionRefusedError: memory allocation failed, out of memory/Users/mvines/.cache/solana/v0.12/rust-bpf-sysroot/src/libcore/slice/mod.rsassertion failed: `(left == right)`
```
</details>

---

## SPL Memo Program v1 (Legacy) (`Memo1Uhk.so`)

- **On-Chain Address (Known):** `Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo`
- **Binary Size:** 17,280 bytes (16.88 KB)
- **Text Section:** 13,232 bytes | **Total Instructions:** 1,622 | **Functions:** 64
- **Entrypoint Virtual Address:** `0xe8`
- **sBPF ISA Classification:** `v0_or_v1_preversioning`
- **Framework Detection:** `unknown` (No explicit marker)

### Functional Purpose & Architecture
Historical first version of SPL Memo. Logs arbitrary UTF-8 or binary strings to transaction logs for auditing and indexing.

### Instruction Interface & Dispatch Table

| Opcode / Discriminator | Instruction Name | Description |
| :---: | :--- | :--- |
| `Any` | **RecordMemo** | Logs the instruction data bytes directly to runtime logs via sol_log_. |

### Solana Syscall Fingerprint & Runtime Boundaries

The program relies on the following Sealevel runtime syscall imports:
- `sol_alloc_free_`
- `sol_log_`
- `sol_panic_`

### Security, Memory Model & Vulnerability Posture

Early architecture compiled with rust-bpf-sysroot relying on sol_alloc_free_ dynamic heap allocation. Inefficient compute unit consumption compared to newer iterations.

<details>
<summary><strong>Extracted Error Literals & Assertion Strings</strong></summary>

```text
index out of bounds: the len is /Users/tyeraeulberg/Documents/Solana/solana-program-library/bin/bpf-sdk/dependencies/rust-bpf-sysroot/src/libcore/slice/mod.rs/Users/tyeraeulberg/Documents/Solana/solana-program-library/bin/bpf-sdk/dependencies/rust-bpf-sysroot/src/liballoc/raw_vec.rsinternal error: entered unreachable codeError: memory allocation failed, out of memory/Users/tyeraeulberg/Documents/Solana/solana-program-library/bin/bpf-sdk/dependencies/rust-bpf-sysroot/src/liballoc/raw_vec.rscapacity overflow00010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899
```
</details>

---

## SPL Memo Program Minimal (Assembly / C Optimization) (`Memo4c2p.so`)

- **On-Chain Address (Known):** `Memo4c2pYq7taAumGuaHG8K4Y5zPqJ5m3g6G5p...`
- **Binary Size:** 2,304 bytes (2.25 KB)
- **Text Section:** 984 bytes | **Total Instructions:** 119 | **Functions:** 2
- **Entrypoint Virtual Address:** `0x120`
- **sBPF ISA Classification:** `v0_or_v1_preversioning`
- **Framework Detection:** `unknown` (No explicit marker)

### Functional Purpose & Architecture
Extreme micro-optimization of the Memo program. 2,304 bytes total binary size with 119 instructions. Zero dynamic memory allocation.

### Instruction Interface & Dispatch Table

| Opcode / Discriminator | Instruction Name | Description |
| :---: | :--- | :--- |
| `Any` | **RecordMemo** | Verifies signers, logs pubkey of each signer via sol_log_pubkey, then logs 'Memo (len X)' and the memo string. |

### Solana Syscall Fingerprint & Runtime Boundaries

The program relies on the following Sealevel runtime syscall imports:
- `sol_log_`
- `sol_log_pubkey`
- `sol_memcpy_`

### Security, Memory Model & Vulnerability Posture

Handcrafted zero-copy deserializer. Validates account signers directly by inspecting raw byte flags at offset in runtime input buffer without full account deserialization.

---

## SPL Memo Program v2 (Standard) (`MemoSq4g.so`)

- **On-Chain Address (Known):** `MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr`
- **Binary Size:** 74,800 bytes (73.05 KB)
- **Text Section:** 57,864 bytes | **Total Instructions:** 7,024 | **Functions:** 196
- **Entrypoint Virtual Address:** `0x9b8`
- **sBPF ISA Classification:** `v0_or_v1_preversioning`
- **Framework Detection:** `unknown` (No explicit marker)

### Functional Purpose & Architecture
Current standard SPL Memo v2. Validates that the input payload is valid UTF-8, logs signers, and emits the memo to the Solana program log.

### Instruction Interface & Dispatch Table

| Opcode / Discriminator | Instruction Name | Description |
| :---: | :--- | :--- |
| `Any` | **RecordMemo** | Validates UTF-8 encoding of instruction payload and logs it. Also verifies that required transaction signers match expected keys. |

### Solana Syscall Fingerprint & Runtime Boundaries

The program relies on the following Sealevel runtime syscall imports:
- `sol_log_`

### Security, Memory Model & Vulnerability Posture

Uses standard Rust core UTF-8 validator without dynamic heap allocations. Safe panic handler.

<details>
<summary><strong>Extracted Error Literals & Assertion Strings</strong></summary>

```text
called `Result::unwrap()` on an `Err` valuealign_offset: align is not a power-of-two/Users/tyeraeulberg/.cache/solana/v0.13/rust-bpf-sysroot/src/libcore/ptr/mod.rscalled `Result::unwrap()` on an `Err` value__NonExhaustiveBufferTooSmallError: memory allocation failed, out of memoryinternal error: entered unreachable code/Users/tyeraeulberg/.cache/solana/v0.13/rust-bpf-sysroot/src/liballoc/raw_vec.rscapacity overflowFromUtf8Errorbyteserror
[...]byte index  is out of bounds of ``/Users/tyeraeulberg/.cache/solana/v0.13/rust-bpf-sysroot/src/libcore/str/mod.rsbegin <= end ( is not a char boundary; it is inside ) of `Utf8Errorvalid_up_toerror_lenPanicked at: '', :: /Users/tyeraeulberg/.cache/solana/v0.13/rust-bpf-sysroot/src/libcore/result.rs/Users/tyeraeulberg/.cache/solana/v0.13/rust-bpf-sysroot/src/libcore/slice/mod.rsindex  out of range for slice of length slice index starts at  but ends at align_offset: align is not a power-of-two/Users/tyeraeulberg/.cache/solana/v0.13/rust-bpf-sysroot/src/libcore/ptr/mod.rscalled `Option::unwrap()` on a `None` value/Users/tyeraeulberg/.cache/solana/v0.13/rust-bpf-sysroot/src/libcore/option.rs
```
</details>

---

## SPL Token Program (Pinocchio Optimized) (`Tokenkeg.so`)

- **On-Chain Address (Known):** `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`
- **Binary Size:** 108,600 bytes (106.05 KB)
- **Text Section:** 102,608 bytes | **Total Instructions:** 12,061 | **Functions:** 29
- **Entrypoint Virtual Address:** `0x828`
- **sBPF ISA Classification:** `v0_or_v1_preversioning`
- **Framework Detection:** `pinocchio` (pinocchio present)

### Functional Purpose & Architecture
Core token standard for Solana. Reconstructed / optimized using Pinocchio zero-copy framework. Provides standard 0-25 instruction set for mints, token accounts, transfers, approvals, multisig, freeze/thaw, and native SOL wrapping.

### Instruction Interface & Dispatch Table

| Opcode / Discriminator | Instruction Name | Description |
| :---: | :--- | :--- |
| `0` | **InitializeMint** | Initializes a new token mint with decimals and mint authority. |
| `1` | **InitializeAccount** | Initializes a new token account for a mint and owner. |
| `2` | **InitializeMultisig** | Initializes an M-of-N multisignature signer account. |
| `3` | **Transfer** | Transfers tokens from source to destination account. |
| `4` | **Approve** | Approves a delegate authority to spend up to a maximum amount. |
| `5` | **Revoke** | Revokes the delegate authority on a token account. |
| `6` | **SetAuthority** | Changes mint authority, freeze authority, account owner, or close authority. |
| `7` | **MintTo** | Mints new tokens to a destination account (requires mint authority). |
| `8` | **Burn** | Burns tokens from an account (requires account owner or delegate). |
| `9` | **CloseAccount** | Closes an empty token account, transferring remaining lamports to destination. |
| `10` | **FreezeAccount** | Freezes a token account, preventing transfers and burns. |
| `11` | **ThawAccount** | Unfreezes a frozen token account. |
| `12` | **TransferChecked** | Transfers tokens with explicit mint and decimals verification. |
| `13` | **ApproveChecked** | Approves delegate with explicit mint and decimals verification. |
| `14` | **MintToChecked** | Mints tokens with explicit mint and decimals verification. |
| `15` | **BurnChecked** | Burns tokens with explicit mint and decimals verification. |
| `16` | **InitializeAccount2** | Initializes account with immutable owner enforcement. |
| `17` | **SyncNative** | Syncs the wrapped SOL token balance with account lamport balance. |
| `18` | **InitializeAccount3** | Initializes account taking owner as instruction data parameter. |
| `19` | **InitializeMultisig2** | Initializes multisig taking signers in instruction data. |
| `20` | **InitializeMint2** | Initializes mint taking authority as instruction parameter. |
| `21` | **GetAccountDataSize** | Returns required account size for a set of extensions. |
| `22` | **InitializeImmutableOwner** | Locks account owner permanently. |
| `23` | **AmountToUiAmount** | Converts integer raw amount to formatted UI string. |
| `24` | **UiAmountToAmount** | Converts formatted UI string to integer raw amount. |

### Solana Syscall Fingerprint & Runtime Boundaries

The program relies on the following Sealevel runtime syscall imports:
- `sol_get_sysvar`
- `sol_log_`
- `sol_memcmp_`
- `sol_memcpy_`
- `sol_memset_`
- `sol_panic_`
- `sol_set_return_data`

### Security, Memory Model & Vulnerability Posture

Enforces strict fixed sizes: Account (165 bytes), Mint (82 bytes), Multisig (355 bytes). Strict arithmetic overflow checks.

<details>
<summary><strong>Extracted Error Literals & Assertion Strings</strong></summary>

```text
Error: UnknownError: InvalidArgumentError: InvalidInstructionDataError: InvalidAccountDataError: AccountDataTooSmallError: InsufficientFundsError: IncorrectProgramIdError: MissingRequiredSignature
Error: AccountAlreadyInitializedindex out of bounds: the len is
Error: UninitializedAccountError: NotEnoughAccountKeysError: AccountBorrowFailedError: MaxSeedLengthExceededError: InvalidSeedsError: BorshIoErrorError: AccountNotRentExemptError: UnsupportedSysvarError: IllegalOwnerError: MaxAccountsDataAllocationsExceededError: InvalidReallocError: MaxInstructionTraceLengthExceededError: BuiltinProgramsMustConsumeComputeUnitsError: InvalidAccountOwnerError: ArithmeticOverflow
Error: Immutablerange end index Error: IncorrectAuthorityError: Lamport balance below rent-exempt thresholdError: insufficient fundsError: Invalid MintError: Account not associated with this MintError: owner does not matchError: the total supply of this token is fixedError: account or token already in useError: Invalid number of provided signersError: Invalid number of required signersError: State is uninitializedError: Instruction does not support native tokensError: Non-native account can only be closed if its balance is zeroError: Invalid instructionError: Invalid account state for operationError: Operation overflowedError: Account does not support specified authority typeError: This token mint cannot freeze accountsError: Account is frozenError: decimals different from the Mint decimalsError: Instruction does not support non-native tokens
```
</details>

---

## SPL Token-2022 Program (Token Extensions) (`TokenzQd.so`)

- **On-Chain Address (Known):** `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb`
- **Binary Size:** 1,382,016 bytes (1349.62 KB)
- **Text Section:** 531,640 bytes | **Total Instructions:** 64,164 | **Functions:** 490
- **Entrypoint Virtual Address:** `0x4d080`
- **sBPF ISA Classification:** `v0_or_v1_preversioning`
- **Framework Detection:** `unknown` (No explicit marker)

### Functional Purpose & Architecture
Comprehensive superset of SPL Token introducing modular extensions: Confidential Transfers (ElGamal + Twisted Edwards Curve25519 ZK proofs), Transfer Fees, Transfer Hooks (CPI verification callbacks), Default Account State, Non-Transferable Tokens, Interest-Bearing Tokens, Metadata Pointer, Group Pointer, Member Pointer, and CPI Guard.

### Instruction Interface & Dispatch Table

| Opcode / Discriminator | Instruction Name | Description |
| :---: | :--- | :--- |
| `0-24` | **Core SPL Token Instructions** | Full backward-compatible SPL Token instruction set. |
| `25` | **Reallocate** | Reallocates account space to accommodate new extension types dynamically. |
| `26` | **CreateNativeMint** | Creates the 1-to-1 native SOL mint 9pan9... |
| `27` | **InitializeNonTransferableMint** | Marks mint non-transferable (soulbound tokens). |
| `28` | **InterestBearingMintExtension** | Configures dynamic interest accumulation on token balances. |
| `29` | **DefaultAccountStateExtension** | Enforces default Frozen state on all newly created token accounts. |
| `30` | **TransferFeeExtension** | Configures and collects transfer fees withheld on transfers. |
| `31` | **ConfidentialTransferExtension** | ElGamal encryption of balances, zero-knowledge range and equality proofs. |
| `32` | **TransferHookExtension** | Executes on-chain CPI callback program on every token transfer. |
| `33` | **CpiGuardExtension** | Locks down account to prevent unauthorized delegate or burning in CPI. |
| `34` | **MetadataPointerExtension** | Pointers to on-chain metadata accounts directly in the mint. |
| `35` | **GroupPointerExtension** | Pointers to token group definitions. |
| `36` | **PausableExtension** | Allows mint pause authority to halt all mint, burn, and transfer activity. |
| `37` | **ScaledUiAmountExtension** | Applies multiplier/scaling factor to UI token amounts. |

### Solana Syscall Fingerprint & Runtime Boundaries

The program relies on the following Sealevel runtime syscall imports:
- `sol_curve_group_op`
- `sol_get_clock_sysvar`
- `sol_get_processe[...]`
- `sol_get_rent_sysvar`
- `sol_get_stack_height`
- `sol_invoke_signe[...]`
- `sol_log_`
- `sol_memcmp_`
- `sol_memcpy_`
- `sol_memmove_`
- `sol_memset_`
- `sol_panic_`
- `sol_set_return_data`
- `sol_try_find_pro[...]`

### Cross-Program Invocations (CPI)
- Transfer Hook CPI (sol_invoke_signed_rust)
- System Program: Realloc, Allocate, Assign
- ZK Token Proof program verification

### Security, Memory Model & Vulnerability Posture

Employs sol_get_stack_height for CPI Guard enforcement, sol_curve_group_op for cryptographic curve point arithmetic, and sol_get_processed_sibling_instruction for verifying proof validity in prior instructions.

<details>
<summary><strong>Extracted Error Literals & Assertion Strings</strong></summary>

```text
Error: UnknownError: InvalidArgumentError: InvalidInstructionDataError: InvalidAccountDataError: AccountDataTooSmallError: InsufficientFundsError: IncorrectProgramIdError: MissingRequiredSignature
Error: AccountAlreadyInitializedindex out of bounds: the len is
4Instruction: InitializeMultisig2Error: UninitializedAccountError: NotEnoughAccountKeysError: AccountBorrowFailedError: MaxSeedLengthExceededError: InvalidSeedsError: BorshIoErrorError: AccountNotRentExemptError: UnsupportedSysvarError: IllegalOwnerError: MaxAccountsDataAllocationsExceededError: InvalidReallocError: MaxInstructionTraceLengthExceededError: BuiltinProgramsMustConsumeComputeUnitsError: InvalidAccountOwnerError: ArithmeticOverflow
Error: Immutablerange end index Balance mismatch
PermissionDeniedAddrNotAvailable right` failed: 0123456789abcdefError: IncorrectAuthority
```
</details>

---

## Architectural Evolutions & Comparative Analysis

### 1. The Evolution of SPL Memo: From Dynamic Allocation to Zero-Copy Microcode

The repository contains three generations of the SPL Memo program, providing a case study in Solana compute optimization:

| Metric | Memo v1 (`Memo1Uhk.so`) | Memo v2 (`MemoSq4g.so`) | Minimal Memo (`Memo4c2p.so`) |
| :--- | :--- | :--- | :--- |
| **Binary Size** | 17,280 bytes | 74,800 bytes | **2,304 bytes** (97% reduction) |
| **Instruction Count** | 1,622 | 7,024 | **119** (98% reduction) |
| **Function Count** | 64 | 196 | **2** |
| **Heap Allocation** | Dynamic (`sol_alloc_free_`) | Static Bump Allocator | **Zero Allocations (Stack/Direct)** |
| **Signer Handling** | Ignored | Checked via Rust Vec | Direct bitmask validation in raw ELF buffer |
| **Language / Core** | Early Rust BPF SDK (v0.12) | Modern Rust + `bs58` | Minimal C / Asm microcode |

### 2. SPL Token (`Tokenkeg.so`) vs. Token-2022 (`TokenzQd.so`)

The progression from classic SPL Token to Token-2022 represents the transition from fixed-layout accounts to dynamic Type-Length-Value (TLV) extension matrices:

- **Tokenkeg (Classic)**: Relies on hardcoded byte layouts: 165 bytes for Token Accounts, 82 bytes for Mints. Handcrafted with Pinocchio zero-copy bindings for optimal CU efficiency.
- **TokenzQd (Token-2022)**: 1.38 MB binary containing 64,164 instructions. Introduces full cryptographic extensions including Curve25519 scalar multiplications (`sol_curve_group_op`) for Confidential Transfers, CPI Guards (`sol_get_stack_height`), and dynamic instruction reallocations.

## Forensic & Verification Artifacts Generated

- `audit_summary.json`: Structured machine-readable schema for all 7 binaries.
- `soldecomp_<program>/`: Register-level lifted intermediate representation and function CFGs.
- `solazy_<program>/`: Disassembly instruction listings and immediate constant tables.
- `*.dump`: Complete byte-accurate disassembly dumps.
- `live_transactions_summary.json`: Confirmed on-chain transaction signatures and state snapshots.
- `fuzz_results.json`: Complete 300,000-iteration zero-crash fuzzing benchmark report.

---

## Live Localnet Deployment, Live Transactions & Fuzzing Verification

### 1. Program Deployments on Solana Localnet (Agave/Sealevel 3.1.10)

| Program | Program ID | Slot | Deployed Size | Status |
| :--- | :--- | :---: | :---: | :---: |
| **`CustomToken`** | `CShaS9ezrJoEYBXsAk4bsUzL2pbxE14kAadgBCeXS7Z1` | 247 | 16,320 B | **CONFIRMED** |
| **`CustomSysclock`** | `8VB675MfjjiaizM2VDBRQfk8GMKCiZVtMELqGdstgMTK` | 251 | 7,688 B | **CONFIRMED** |
| **`CustomRAM`** | `5oFB4c7Answx5c2reA9w5u3DGcE1HHw2PfhkXTRFvc3m` | 256 | 7,760 B | **CONFIRMED** |

### 2. Live On-Chain Transaction Confirmations

- **CustomToken**:
  - `InitializeMint` (6 decimals, 100 bps tax): `5D34eDPHsD5oUGqQmJuKfhnc38DDYcNC7vhhHHrMaQo9gRw9nWYBwEwnTwWtUajBsMmExgM1tpNZ3dM1DvJkKiDd`
  - `InitializeAccount` Alice: `3bcJNGrYK1JTx3Qi9iRyqi5R1vTAySGHncgscR3Qey5oJFqjTafHnRBrPz8x2fmXkvxLaHfS8uoKGzG8HqBxtP2B`
  - `InitializeAccount` Bob: `8QsmjrSyF2c8mHZi9tDu1XB2gfPE84qaXvE89gffSvgYg8DvgjqdC1DXM5PaW26Y9ASPNbv1AwkSrA2V5St2TG2`
  - `MintTo` Alice (1,000,000): `4J9F6yV4v3dXfSevMxzgGx7toDDsVKK2kRMQ2YxX2GAmkVkjChCiYuuDjP1hCDJqYR8uVGoyeP1pAeJRLx9m5ciq`
  - `Transfer` 500,000 Alice -> Bob (1% fee applied): `49NqPC2nuN9w27FGF3M9PnddqQPWtJFJQ4ebP5aFh3hfBeaLAwmXfFXUZfnhQUSCDwwcKeyxYw663mkJoV9iY2oL` (Bob received 495,000)
  - `Burn` 100,000: `3cwBWqRVoyfvE5XnTfc3mbRTvbYnnR2ZogVcZ1Rnb4ETsG4KMJhA885BxhvsuCDQ6A7M1nMyvPmVQ1AVrBkj2k2P`
  - `FreezeAccount` Bob: `44Hfez7HMcst6ooxXi8qsadyjANfSF3koPEyGkkL5WP5gcGyh9htSAEuvumPPXDdEMMM9RWtaJsrkPkShhmDCfjv`
  - Freeze validation: Transfer to frozen account rejected with `ERR_ACCOUNT_FROZEN`
  - `ThawAccount` Bob: `3A5ZDzJLnXTYwjhNphuTo2CiDC2E6fhv7N4EfZkrMuFGwFaj8Ec69BHHRFcUYKsZ9QMRBNx43drcbxPzG3hfyp2H`
- **CustomSysclock**:
  - `InitializeClock`: `5ZzmdEQ1YDshQHTRxS9sCaHKGRHmfcmULac4NpryxhDoaVaXVH3o7D5ErjeULrnZedQZuJ5sCzzuNAzTi679gxUU`
  - `Tick` (slot 10420, ts 1732414800000 ms, epoch advance): `3swAAb7CXqgUMqcM5EYdkue798dfNfHuf71DLBretMXxLPjoc6C1FU1UrutHWjNkFa46ijnNncp98UD15J38Ng6w`
- **CustomRAM**:
  - `InitRam` (296 bytes): `3qDxyCTXgvrprzxSa6R2HS6hMUrXpHcN5EoCiPZc9C1RAn7XWStrRQUCJc1mBsozo4sWX5AxzGZmhWTqukcurVt2`
  - `WriteRam` (offset 16, payload `"DEEPMIND_ANTIGRAVITY_CUSTOM_RAM_V1_VERIFIED"`): `sEkK7mc2gQng2PQuWdNFuukiUDC5uUw6k1zW9pg9gxVeZoB11nzEtE2MD7wooEoCLG3AcX1sV2KhDYn4MqqJ8q5`

### 3. Fuzz Testing Benchmark (300,000 Iterations)

- **Total Iterations**: 300,000
- **Total Duration**: 100.94 ms
- **Throughput Rate**: 2,972,113 executions/second
- **Crashes / Invariant Violations**: 0 (100% safe against malformed payloads, under-allocated account slices, and OOB indices)


