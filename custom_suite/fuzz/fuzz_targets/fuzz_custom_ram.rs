#![no_main]
// SPDX-License-Identifier: PACARL-1.0
// Copyright (c) 2026 praisethedevel <xsolutionx@gmail.com>. All Rights Reserved.
// Governed by the Praisethedevel Universal Architecture, Compulsory Attribution & Commercial Royalty License.
// Non-commercial / evaluation use only. Commercial exploitation strictly requires prior written agreement.
use libfuzzer_sys::fuzz_target;

const VALID_CODES: &[u64] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    use custom_ram::*;

    // Max capacity is u16::MAX = 65535 bytes; we cap our fake buffer at 256 bytes
    const BUF_SIZE: usize = 256;
    let mut header_data = [0u8; 40 + BUF_SIZE];
    // Optionally mark initialized and set capacity
    if data.len() > 1 && data[1] & 1 == 1 {
        header_data[0] = 1; // is_initialized
        // capacity = 256
        header_data[2..4].copy_from_slice(&(BUF_SIZE as u16).to_le_bytes());
    }

    let owner_key = [0u8; 32];
    let ram_key   = [1u8; 32];

    let ram_view = AccountView {
        key: ram_key.as_ptr(),
        lamports: core::ptr::addr_of!(u64::MAX) as *mut u64,
        data: header_data.as_mut_ptr(),
        data_len: 40 + BUF_SIZE,
        owner: owner_key.as_ptr(),
        is_signer: true,
        is_writable: true,
    };
    let auth_view = AccountView {
        key: owner_key.as_ptr(),
        lamports: core::ptr::addr_of!(u64::MAX) as *mut u64,
        data: core::ptr::NonNull::dangling().as_ptr(),
        data_len: 0,
        owner: owner_key.as_ptr(),
        is_signer: true,
        is_writable: false,
    };
    let accounts = [ram_view, auth_view];
    let result = process_instruction(&accounts, data);
    assert!(
        VALID_CODES.contains(&result),
        "process_instruction returned unexpected code {result}"
    );
});
