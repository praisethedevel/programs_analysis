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

    use custom_sysclock::*;

    let mut clock_data = [0u8; 72];
    // Optionally mark initialized
    if data.len() > 1 && data[1] & 1 == 1 {
        clock_data[0] = 1;
    }
    let auth_key  = [0u8; 32];
    let clock_key = [1u8; 32];

    let clock_view = AccountView {
        key: clock_key.as_ptr(),
        lamports: core::ptr::addr_of!(u64::MAX) as *mut u64,
        data: clock_data.as_mut_ptr(),
        data_len: 72,
        owner: auth_key.as_ptr(),
        is_signer: true,
        is_writable: true,
    };
    let auth_view = AccountView {
        key: auth_key.as_ptr(),
        lamports: core::ptr::addr_of!(u64::MAX) as *mut u64,
        data: core::ptr::NonNull::dangling().as_ptr(),
        data_len: 0,
        owner: auth_key.as_ptr(),
        is_signer: true,
        is_writable: false,
    };
    let accounts = [clock_view, auth_view];
    let result = process_instruction(&accounts, data);
    assert!(
        VALID_CODES.contains(&result),
        "process_instruction returned unexpected code {result}"
    );
});
