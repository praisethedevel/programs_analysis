# SBPF Zero-Copy On-Chain Compute Unit Benchmarks

Verified on Solana Devnet (Cluster `525743223`+). Real transaction execution data retrieved directly from validator transaction metadata.

---

## ⚡ Compute Unit (CU) Consumption Comparison

| Operation | Custom Zero-Copy Suite | Standard SPL Token / Anchor | CU Savings | Performance Multiplier |
|---|---|---|---|---|
| **Token Transfer (w/ Fee)** | **589 CU** | ~4,800 CU | **-87.7%** | **8.1x faster** |
| **Token MintTo** | **340 CU** | ~3,200 CU | **-89.4%** | **9.4x faster** |
| **Token Initialize Mint** | **432 CU** | ~2,900 CU | **-85.1%** | **6.7x faster** |
| **Token Initialize Account** | **460 CU** | ~3,400 CU | **-86.5%** | **7.4x faster** |
| **Oracle Tick (Sysclock)** | **232 CU** | ~2,500 CU (Anchor) | **-90.7%** | **10.8x faster** |
| **RAM Direct Write (42B)** | **246 CU** | ~3,100 CU (Borsh) | **-92.1%** | **12.6x faster** |

---

## 🔬 Architectural Reasons for ~90% Compute Unit Reduction

1. **Pure `#![no_std]` Stack Allocation**:
   - Zero dynamic heap allocation (`alloc` or `Vec`).
   - No dynamic memory allocator syscalls (`sol_alloc_free_`).

2. **Pointer Casting Over Deserialization**:
   - Bypasses Borsh/Anchor 8-byte discriminator hashing and iterative byte-by-byte deserialization.
   - Casts raw Sealevel input memory slices directly into `#[repr(C)]` memory layouts via pointer offsets.

3. **In-Place Account Mutation**:
   - State mutations occur directly in the account's existing input buffer slice.
   - Eliminates serialization re-encoding before runtime finalization.

---

## 📜 Devnet Transaction Telemetry Receipts

| Operation | Signature | Slot | Status |
|---|---|---|---|
| `InitMint` | `zNVdPTKEsM9ofyWU6e7akN7B7naySPz2688wo8YweRZJ2CGxvuFibEyGfkHQdnqpBr9F7WiCW4EuVf1dvK9qjjN` | 525743450 | Confirmed (432 CU) |
| `InitAlice` | `4hxQ4hqtcgJcYQFPLZEkwE83nVmZ1FFqUQ1XUZbDJE4LeW4nqAm8G8umwhTruFnpxKsHeVUsKrs2Me1wLo21uTxe` | 525743455 | Confirmed (460 CU) |
| `MintTo` | `3MjXMxktxpirh9UVnHPP5uLHHEbchxZnPZh2AkxhqHtjC4ETHSCYUqnHQtd1nkkVzgLJQgzZ2eSB6EjZ6Dvkyjqw` | 525743460 | Confirmed (340 CU) |
| `Transfer` | `4cNmJpXJPdLdu92v7VtBdBjqXQTuczHkcJGPaEaJU4Tbma5e6YyjXWFkMXwrzcAeHR72MDVTQotBgENkW3h7FBmT` | 525743465 | Confirmed (589 CU) |
| `ClockInit` | `2UEcRXGQ8Uy9gmshPDj1ZcCFBEPT6PkAUzUAaJWmvVQMefg8rM61LgyQ18Px13xWoAg9rHYwZjVDgwCbX7ET913P` | 525743470 | Confirmed (368 CU) |
| `ClockTick` | `5T6qBqLhYxH7GL4i9zJC67MLzKjZEpvwVvEH1z3qZjc85tDmsfqcAahqH7QHtn9mpLtWoidi9zKhtoyTuzxX2ziS` | 525743475 | Confirmed (232 CU) |
| `RamInit` | `4gjXeX3t7hzg8ruziNHmcqoMEFPLEjB9uX4bFVxLPjjAVZxXW3BwuE2bPHKDG4cdaAVR7ttQ6C1W6B2JRiohcjKG` | 525743480 | Confirmed (364 CU) |
| `RamWrite` | `644TcGoD2pHZsCnVFuhYxe1ZXK9faca617dhcpmNs6HUEijMeMhZnZMGXeg1RkwxV3WcTpFCmFnPyw25XeYjTGCX` | 525743485 | Confirmed (246 CU) |

---

*Governed by the Praisethedevel Universal Architecture, Compulsory Attribution & Commercial Royalty License (PACARL-1.0).*
