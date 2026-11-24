#![no_main]
use libfuzzer_sys::fuzz_target;

// Valid error codes our token program may return
const VALID_CODES: &[u64] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    // Build a fake Sealevel account input mimicking what custom_token expects.
    // We pass fuzz-generated instruction data with a fixed set of small accounts
    // so the entrypoint can run its full parsing path.

    // Allocate fake mint account (80 bytes)
    let mut mint_data = [0u8; 80];
    // Mark mint initialized (byte 0 = 1) if data says so
    if data.len() > 1 && data[1] & 1 == 1 {
        mint_data[0] = 1; // is_initialized
        mint_data[1] = 6; // decimals
    }

    // Allocate fake token account (80 bytes)
    let mut acct_data = [0u8; 80];
    if data.len() > 2 && data[2] & 1 == 1 {
        acct_data[0] = 1; // is_initialized
    }

    // Build instruction data: first byte is the tag, rest is payload
    let ix_data: Vec<u8> = data.to_vec();

    // Call the parse-and-dispatch path via the public process functions
    // that the entrypoint delegates to.  We just check it doesn't panic.
    use custom_token::*;

    // Simulate the account views manually with stack buffers.
    let mint_key   = [0u8; 32];
    let owner_key  = [0u8; 32];
    let acct_key   = [1u8; 32];
    let dst_key    = [2u8; 32];
    let mut dst_data = [0u8; 80];

    // Build AccountView array for each instruction variant
    // so the dispatcher always has at least some accounts to index into.
    let mint_view = AccountView {
        key: mint_key.as_ptr(),
        lamports: core::ptr::addr_of!(u64::MAX) as *mut u64,
        data: mint_data.as_mut_ptr(),
        data_len: 80,
        owner: owner_key.as_ptr(),
        is_signer: true,
        is_writable: true,
    };
    let acct_view = AccountView {
        key: acct_key.as_ptr(),
        lamports: core::ptr::addr_of!(u64::MAX) as *mut u64,
        data: acct_data.as_mut_ptr(),
        data_len: 80,
        owner: owner_key.as_ptr(),
        is_signer: true,
        is_writable: true,
    };
    let dst_view = AccountView {
        key: dst_key.as_ptr(),
        lamports: core::ptr::addr_of!(u64::MAX) as *mut u64,
        data: dst_data.as_mut_ptr(),
        data_len: 80,
        owner: owner_key.as_ptr(),
        is_signer: false,
        is_writable: true,
    };
    let accounts = [mint_view, acct_view, dst_view];

    let result = process_instruction(&accounts, &ix_data);
    // Must always be a valid error code — never an arbitrary value
    assert!(
        VALID_CODES.contains(&result),
        "process_instruction returned unexpected code {result}"
    );
});
