#![cfg_attr(target_os = "solana", no_std)]
// SPDX-License-Identifier: PACARL-1.0
// Copyright (c) 2026 praisethedevel <xsolutionx@gmail.com>. All Rights Reserved.
// Governed by the Praisethedevel Universal Architecture, Compulsory Attribution & Commercial Royalty License.
// Non-commercial / evaluation use only. Commercial exploitation strictly requires prior written agreement.

#[cfg(target_os = "solana")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(target_os = "solana")]
extern "C" {
    fn sol_log_(message: *const u8, length: u64);
    fn sol_set_return_data(data: *const u8, length: u64);
}

#[cfg(not(target_os = "solana"))]
unsafe fn sol_log_(_message: *const u8, _length: u64) {}

#[cfg(not(target_os = "solana"))]
static mut RETURN_DATA_BUF: [u8; 1024] = [0; 1024];
#[cfg(not(target_os = "solana"))]
static mut RETURN_DATA_LEN: usize = 0;

#[cfg(not(target_os = "solana"))]
unsafe fn sol_set_return_data(data: *const u8, length: u64) {
    let len = length as usize;
    core::ptr::copy_nonoverlapping(data, RETURN_DATA_BUF.as_mut_ptr(), len);
    RETURN_DATA_LEN = len;
}

fn log(msg: &str) {
    unsafe { sol_log_(msg.as_ptr(), msg.len() as u64) };
}

// -----------------------------------------------------------------------------
// Security Metadata (Solana Neodyme SECURITY.TXT Standard v1)
// -----------------------------------------------------------------------------
#[cfg_attr(target_os = "solana", no_mangle)]
pub static SECURITY_TXT: &str = "=======BEGIN SECURITY.TXT V1=======\0\
name\0CustomSysclock (Zero-Copy Oracle Rebuild)\0\
project_url\0https://github.com/praisethedevel/programs_analysis\0\
contacts\0email:xsolutionx@gmail.com,link:https://github.com/praisethedevel/programs_analysis/security\0\
policy\0https://github.com/praisethedevel/programs_analysis/blob/main/SECURITY.md\0\
preferred_languages\0en\0\
source_code\0https://github.com/praisethedevel/programs_analysis/tree/main/custom_suite/custom_sysclock\0\
source_release\0custom-sysclock@v0.1.0\0\
auditors\0Autonomous Forensic SBPF Engine\0\
expiry\02028-01-01\0\
=======END SECURITY.TXT V1=======\0";

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CustomClockState {
    pub is_initialized: u8,
    pub is_frozen: u8,
    pub _padding: [u8; 6],
    pub slot: u64,
    pub epoch: u64,
    pub unix_timestamp_ms: i64,
    pub tick_sequence: u64,
    pub authority: [u8; 32],
}

impl CustomClockState {
    pub const LEN: usize = 72;
}
pub const CLOCK_STATE_SIZE: usize = CustomClockState::LEN;

#[derive(Clone, Copy)]
pub struct AccountView {
    pub is_signer: bool,
    pub pubkey: *const [u8; 32],
    pub data_len: usize,
    pub data: *mut u8,
}

pub const SUCCESS: u64 = 0;
pub const ERR_INVALID_INSTRUCTION: u64 = 1;
pub const ERR_NOT_ENOUGH_ACCOUNTS: u64 = 2;
pub const ERR_MISSING_SIGNER: u64 = 3;
pub const ERR_ALREADY_INITIALIZED: u64 = 4;
pub const ERR_UNINITIALIZED: u64 = 5;
pub const ERR_UNAUTHORIZED: u64 = 6;

#[cfg_attr(target_os = "solana", no_mangle)]
pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
    let num_accounts = *(input as *const u64) as usize;
    let mut offset = 8;
    
    let mut accounts: [AccountView; 4] = [AccountView {
        is_signer: false,
        pubkey: core::ptr::null(),
        data_len: 0,
        data: core::ptr::null_mut(),
    }; 4];

    let count = if num_accounts > 4 { 4 } else { num_accounts };
    for i in 0..count {
        let dup = *input.add(offset);
        if dup == 0xff {
            accounts[i] = AccountView {
                is_signer: *input.add(offset + 1) != 0,
                pubkey: input.add(offset + 8) as *const [u8; 32],
                data_len: *(input.add(offset + 80) as *const u64) as usize,
                data: input.add(offset + 88),
            };
            offset += 88 + accounts[i].data_len + 10240 + 8;
            offset = (offset + 7) & !7;
        } else {
            accounts[i] = accounts[dup as usize];
            offset += 8;
        }
    }

    let instruction_data_len = *(input.add(offset) as *const u64) as usize;
    offset += 8;
    let instruction_data = core::slice::from_raw_parts(input.add(offset), instruction_data_len);

    let res = process_instruction(&accounts[..count], instruction_data);
    if res == SUCCESS {
        SUCCESS
    } else {
        (1u64 << 32) | res
    }
}

pub unsafe fn process_instruction(accounts: &[AccountView], instruction_data: &[u8]) -> u64 {
    if instruction_data.is_empty() {
        return ERR_INVALID_INSTRUCTION;
    }

    match instruction_data[0] {
        0 => init_clock(accounts),
        1 => tick(accounts, &instruction_data[1..]),
        2 => get_time(accounts),
        _ => ERR_INVALID_INSTRUCTION,
    }
}

// 0: InitializeClock
pub unsafe fn init_clock(accounts: &[AccountView]) -> u64 {
    if accounts.len() < 2 {
        return ERR_NOT_ENOUGH_ACCOUNTS;
    }
    let clock_acc = &accounts[0];
    let authority = &accounts[1];

    if !authority.is_signer {
        return ERR_MISSING_SIGNER;
    }
    if clock_acc.data_len < CLOCK_STATE_SIZE {
        return ERR_UNINITIALIZED;
    }

    let clock = &mut *(clock_acc.data as *mut CustomClockState);
    if clock.is_initialized != 0 {
        return ERR_ALREADY_INITIALIZED;
    }

    clock.is_initialized = 1;
    clock.is_frozen = 0;
    clock._padding = [0; 6];
    clock.slot = 0;
    clock.epoch = 0;
    clock.unix_timestamp_ms = 0;
    clock.tick_sequence = 0;
    clock.authority = *authority.pubkey;

    log("CustomSysclock: Initialized");
    SUCCESS
}

// 1: Tick { slot: u64, unix_timestamp_ms: i64, advance_epoch: u8 }
pub unsafe fn tick(accounts: &[AccountView], data: &[u8]) -> u64 {
    if accounts.len() < 2 || data.len() < 17 {
        return ERR_NOT_ENOUGH_ACCOUNTS;
    }
    let clock_acc = &accounts[0];
    let authority = &accounts[1];

    if !authority.is_signer {
        return ERR_MISSING_SIGNER;
    }
    if clock_acc.data_len < CLOCK_STATE_SIZE {
        return ERR_UNINITIALIZED;
    }

    let clock = &mut *(clock_acc.data as *mut CustomClockState);
    if clock.is_initialized == 0 {
        return ERR_UNINITIALIZED;
    }
    if clock.authority != *authority.pubkey {
        return ERR_UNAUTHORIZED;
    }

    clock.slot = u64::from_le_bytes(*(data[0..8].as_ptr() as *const [u8; 8]));
    clock.unix_timestamp_ms = i64::from_le_bytes(*(data[8..16].as_ptr() as *const [u8; 8]));
    if data[16] != 0 {
        clock.epoch += 1;
    }
    clock.tick_sequence += 1;

    log("CustomSysclock: Tick Processed");
    SUCCESS
}

// 2: GetTime
pub unsafe fn get_time(accounts: &[AccountView]) -> u64 {
    if accounts.is_empty() {
        return ERR_NOT_ENOUGH_ACCOUNTS;
    }
    if accounts[0].data_len < CLOCK_STATE_SIZE {
        return ERR_UNINITIALIZED;
    }
    let clock = &*(accounts[0].data as *const CustomClockState);
    if clock.is_initialized == 0 {
        return ERR_UNINITIALIZED;
    }

    let mut buf = [0u8; 24];
    buf[0..8].copy_from_slice(&clock.slot.to_le_bytes());
    buf[8..16].copy_from_slice(&clock.epoch.to_le_bytes());
    buf[16..24].copy_from_slice(&clock.unix_timestamp_ms.to_le_bytes());

    sol_set_return_data(buf.as_ptr(), 24);
    log("CustomSysclock: Return Data Set");
    SUCCESS
}

#[cfg(test)]
#[allow(static_mut_refs)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_lifecycle() {
        let auth_pubkey = [7u8; 32];
        let clock_pubkey = [8u8; 32];
        let mut clock_data = [0u8; 72];

        unsafe {
            let clock_view = AccountView {
                is_signer: false,
                pubkey: &clock_pubkey,
                data_len: 72,
                data: clock_data.as_mut_ptr(),
            };
            let auth_view = AccountView {
                is_signer: true,
                pubkey: &auth_pubkey,
                data_len: 0,
                data: core::ptr::null_mut(),
            };

            // Init clock
            assert_eq!(init_clock(&[clock_view, auth_view]), SUCCESS);

            // Tick clock: slot 1000, timestamp 1700000000000 ms, advance_epoch = true
            let mut tick_data = [0u8; 17];
            tick_data[0..8].copy_from_slice(&1000u64.to_le_bytes());
            tick_data[8..16].copy_from_slice(&1700000000000i64.to_le_bytes());
            tick_data[16] = 1;

            assert_eq!(tick(&[clock_view, auth_view], &tick_data), SUCCESS);

            let clock = &*(clock_data.as_ptr() as *const CustomClockState);
            assert_eq!(clock.slot, 1000);
            assert_eq!(clock.unix_timestamp_ms, 1700000000000);
            assert_eq!(clock.epoch, 1);
            assert_eq!(clock.tick_sequence, 1);

            // Read time via return data
            assert_eq!(get_time(&[clock_view]), SUCCESS);
            assert_eq!(RETURN_DATA_LEN, 24);
        }
    }
}
