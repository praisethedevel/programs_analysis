// SPDX-License-Identifier: PACARL-1.0
// Copyright (c) 2026 praisethedevel <xsolutionx@gmail.com>. All Rights Reserved.
// Governed by the Praisethedevel Universal Architecture, Compulsory Attribution & Commercial Royalty License.
// Non-commercial / evaluation use only. Commercial exploitation strictly requires prior written agreement.

use std::time::Instant;

// Fast xorshift64 PRNG for reproducible, high-throughput fuzzing
struct XorShift64(u64);
impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 { 0xdeadbeefcafebabe } else { seed })
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn fill_bytes(&mut self, buf: &mut [u8]) {
        for chunk in buf.chunks_mut(8) {
            let val = self.next_u64().to_le_bytes();
            let len = chunk.len();
            chunk.copy_from_slice(&val[..len]);
        }
    }
    fn next_range(&mut self, min: usize, max: usize) -> usize {
        if min >= max { return min; }
        min + (self.next_u64() as usize % (max - min + 1))
    }
}

fn main() {
    println!("============================================================");
    println!("⚡ STARTING ZERO-COPY SBPF PROGRAM SUITE FUZZER");
    println!("   Target: 300,000 fuzz iterations with pseudorandom mutations");
    println!("============================================================\n");

    let total_start = Instant::now();
    let mut rng = XorShift64::new(0x1337c0de);

    // =========================================================================
    // 1. FUZZ CUSTOM TOKEN PROGRAM
    // =========================================================================
    println!(">>> [1/3] Fuzzing CustomToken Program (100,000 iterations)...");
    let token_start = Instant::now();
    let mut token_stats = [0u64; 10]; // Counts for error codes 0..=9
    let mut token_crashes = 0usize;

    for iter in 0..100_000 {
        // Generate random instruction data (0 to 128 bytes)
        let ix_len = match iter % 6 {
            0 => 0,
            1 => 1,
            2 => 4,
            3 => 9,
            4 => rng.next_range(1, 16),
            _ => rng.next_range(16, 128),
        };
        let mut ix_data = vec![0u8; ix_len];
        rng.fill_bytes(&mut ix_data);
        if !ix_data.is_empty() && iter % 2 == 0 {
            // Bias first byte to valid instruction tags (0..=6)
            ix_data[0] = (rng.next_u64() % 7) as u8;
        }

        // Varied account data allocations: under-sized, exact, over-sized
        let mint_buf_len = match (iter / 7) % 5 {
            0 => 0,
            1 => 40,
            2 => custom_token::MINT_SIZE,
            3 => custom_token::MINT_SIZE - 1,
            _ => custom_token::MINT_SIZE + rng.next_range(1, 64),
        };
        let mut mint_buf = vec![0u8; mint_buf_len];
        rng.fill_bytes(&mut mint_buf);

        let acct_buf_len = match (iter / 11) % 5 {
            0 => 0,
            1 => 32,
            2 => custom_token::ACCOUNT_SIZE,
            3 => custom_token::ACCOUNT_SIZE - 1,
            _ => custom_token::ACCOUNT_SIZE + rng.next_range(1, 64),
        };
        let mut acct1_buf = vec![0u8; acct_buf_len];
        let mut acct2_buf = vec![0u8; acct_buf_len];
        rng.fill_bytes(&mut acct1_buf);
        rng.fill_bytes(&mut acct2_buf);

        let key0 = [1u8; 32];
        let key1 = [2u8; 32];
        let key2 = [3u8; 32];
        let key3 = [4u8; 32];
        let mut lamports = 1_000_000_000u64;

        let accounts = [
            custom_token::AccountView {
                is_signer: (iter & 1) != 0,
                is_writable: true,
                pubkey: &key0,
                owner: &key0,
                lamports: &mut lamports,
                data_len: mint_buf_len,
                data: mint_buf.as_mut_ptr(),
            },
            custom_token::AccountView {
                is_signer: (iter & 2) != 0,
                is_writable: true,
                pubkey: &key1,
                owner: &key0,
                lamports: &mut lamports,
                data_len: acct_buf_len,
                data: acct1_buf.as_mut_ptr(),
            },
            custom_token::AccountView {
                is_signer: (iter & 4) != 0,
                is_writable: true,
                pubkey: &key2,
                owner: &key0,
                lamports: &mut lamports,
                data_len: acct_buf_len,
                data: acct2_buf.as_mut_ptr(),
            },
            custom_token::AccountView {
                is_signer: (iter & 8) != 0,
                is_writable: false,
                pubkey: &key3,
                owner: &key0,
                lamports: &mut lamports,
                data_len: 0,
                data: std::ptr::null_mut(),
            },
        ];

        let num_accs = rng.next_range(0, 4);
        let ret = unsafe {
            custom_token::process_instruction(&accounts[..num_accs], &ix_data)
        };

        if ret > 9 {
            token_crashes += 1;
            eprintln!("Token fuzzer anomaly at iteration {iter}: unexpected code {ret}");
        } else {
            token_stats[ret as usize] += 1;
        }
    }
    let token_dur = token_start.elapsed();
    println!("    Finished 100,000 iterations in {:.2?}", token_dur);
    println!("    Rate: {:.0} exec/sec", 100_000.0 / token_dur.as_secs_f64());
    println!("    Crashes / Invariants violated: {token_crashes}");
    println!("    Return distribution: SUCCESS={}, NOT_ENOUGH_ACCTS={}, MISSING_SIGNER={}, UNINITIALIZED={}, OVERFLOW={}",
        token_stats[0], token_stats[2], token_stats[3], token_stats[5], token_stats[9]);
    assert_eq!(token_crashes, 0);

    // =========================================================================
    // 2. FUZZ CUSTOM SYSCLOCK PROGRAM
    // =========================================================================
    println!("\n>>> [2/3] Fuzzing CustomSysclock Program (100,000 iterations)...");
    let clock_start = Instant::now();
    let mut clock_stats = [0u64; 10];
    let mut clock_crashes = 0usize;

    for iter in 0..100_000 {
        let ix_len = match iter % 5 {
            0 => 0,
            1 => 1,
            2 => 18,
            3 => rng.next_range(1, 17),
            _ => rng.next_range(19, 64),
        };
        let mut ix_data = vec![0u8; ix_len];
        rng.fill_bytes(&mut ix_data);
        if !ix_data.is_empty() && iter % 2 == 0 {
            ix_data[0] = (rng.next_u64() % 3) as u8;
        }

        let clock_buf_len = match (iter / 5) % 4 {
            0 => 0,
            1 => custom_sysclock::CLOCK_STATE_SIZE - 1,
            2 => custom_sysclock::CLOCK_STATE_SIZE,
            _ => custom_sysclock::CLOCK_STATE_SIZE + rng.next_range(1, 32),
        };
        let mut clock_buf = vec![0u8; clock_buf_len];
        rng.fill_bytes(&mut clock_buf);

        let key0 = [10u8; 32];
        let key1 = [11u8; 32];

        let accounts = [
            custom_sysclock::AccountView {
                is_signer: (iter & 1) != 0,
                pubkey: &key0,
                data_len: clock_buf_len,
                data: clock_buf.as_mut_ptr(),
            },
            custom_sysclock::AccountView {
                is_signer: (iter & 2) != 0,
                pubkey: &key1,
                data_len: 0,
                data: std::ptr::null_mut(),
            },
        ];

        let num_accs = rng.next_range(0, 2);
        let ret = unsafe {
            custom_sysclock::process_instruction(&accounts[..num_accs], &ix_data)
        };

        if ret > 6 {
            clock_crashes += 1;
            eprintln!("Sysclock fuzzer anomaly at iteration {iter}: unexpected code {ret}");
        } else {
            clock_stats[ret as usize] += 1;
        }
    }
    let clock_dur = clock_start.elapsed();
    println!("    Finished 100,000 iterations in {:.2?}", clock_dur);
    println!("    Rate: {:.0} exec/sec", 100_000.0 / clock_dur.as_secs_f64());
    println!("    Crashes / Invariants violated: {clock_crashes}");
    println!("    Return distribution: SUCCESS={}, INVALID_IX={}, NOT_ENOUGH_ACCTS={}, MISSING_SIGNER={}, UNINITIALIZED={}, UNAUTHORIZED={}",
        clock_stats[0], clock_stats[1], clock_stats[2], clock_stats[3], clock_stats[5], clock_stats[6]);
    assert_eq!(clock_crashes, 0);

    // =========================================================================
    // 3. FUZZ CUSTOM RAM PROGRAM
    // =========================================================================
    println!("\n>>> [3/3] Fuzzing CustomRAM Program (100,000 iterations)...");
    let ram_start = Instant::now();
    let mut ram_stats = [0u64; 10];
    let mut ram_crashes = 0usize;

    for iter in 0..100_000 {
        let ix_len = match iter % 6 {
            0 => 0,
            1 => 1,
            2 => 5,
            3 => 9,
            4 => rng.next_range(1, 16),
            _ => rng.next_range(16, 256),
        };
        let mut ix_data = vec![0u8; ix_len];
        rng.fill_bytes(&mut ix_data);
        if !ix_data.is_empty() && iter % 2 == 0 {
            ix_data[0] = (rng.next_u64() % 3) as u8;
        }

        let ram_buf_len = match (iter / 7) % 5 {
            0 => 0,
            1 => custom_ram::RAM_HEADER_SIZE - 1,
            2 => custom_ram::RAM_HEADER_SIZE,
            3 => custom_ram::RAM_HEADER_SIZE + 64,
            _ => custom_ram::RAM_HEADER_SIZE + rng.next_range(65, 512),
        };
        let mut ram_buf = vec![0u8; ram_buf_len];
        rng.fill_bytes(&mut ram_buf);

        let key0 = [20u8; 32];
        let key1 = [21u8; 32];

        let accounts = [
            custom_ram::AccountView {
                is_signer: (iter & 1) != 0,
                pubkey: &key0,
                data_len: ram_buf_len,
                data: ram_buf.as_mut_ptr(),
            },
            custom_ram::AccountView {
                is_signer: (iter & 2) != 0,
                pubkey: &key1,
                data_len: 0,
                data: std::ptr::null_mut(),
            },
        ];

        let num_accs = rng.next_range(0, 2);
        let ret = unsafe {
            custom_ram::process_instruction(&accounts[..num_accs], &ix_data)
        };

        if ret > 7 {
            ram_crashes += 1;
            eprintln!("RAM fuzzer anomaly at iteration {iter}: unexpected code {ret}");
        } else {
            ram_stats[ret as usize] += 1;
        }
    }
    let ram_dur = ram_start.elapsed();
    println!("    Finished 100,000 iterations in {:.2?}", ram_dur);
    println!("    Rate: {:.0} exec/sec", 100_000.0 / ram_dur.as_secs_f64());
    println!("    Crashes / Invariants violated: {ram_crashes}");
    println!("    Return distribution: SUCCESS={}, INVALID_IX={}, NOT_ENOUGH_ACCTS={}, MISSING_SIGNER={}, UNINITIALIZED={}, OUT_OF_BOUNDS={}",
        ram_stats[0], ram_stats[1], ram_stats[2], ram_stats[3], ram_stats[5], ram_stats[7]);
    assert_eq!(ram_crashes, 0);

    let total_dur = total_start.elapsed();
    println!("\n============================================================");
    println!("✅ ALL 300,000 FUZZ ITERATIONS PASSED WITH ZERO CRASHES!");
    println!("   Total duration: {:.2?} (overall rate: {:.0} exec/sec)",
        total_dur, 300_000.0 / total_dur.as_secs_f64());
    println!("============================================================");
}
