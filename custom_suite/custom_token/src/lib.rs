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
}

#[cfg(not(target_os = "solana"))]
unsafe fn sol_log_(_message: *const u8, _length: u64) {}

fn log(msg: &str) {
    unsafe { sol_log_(msg.as_ptr(), msg.len() as u64) };
}

// -----------------------------------------------------------------------------
// Security Metadata (Solana Neodyme SECURITY.TXT Standard v1)
// -----------------------------------------------------------------------------
#[cfg_attr(target_os = "solana", no_mangle)]
pub static SECURITY_TXT: &str = "=======BEGIN SECURITY.TXT V1=======\0\
name\0CustomToken (Zero-Copy SPL Token Rebuild)\0\
project_url\0https://github.com/praisethedevel/programs_analysis\0\
contacts\0email:xsolutionx@gmail.com,link:https://github.com/praisethedevel/programs_analysis/security\0\
policy\0https://github.com/praisethedevel/programs_analysis/blob/main/SECURITY.md\0\
preferred_languages\0en\0\
source_code\0https://github.com/praisethedevel/programs_analysis/tree/main/custom_suite/custom_token\0\
source_release\0custom-token@v0.1.0\0\
auditors\0Autonomous Forensic SBPF Engine\0\
expiry\02028-01-01\0\
=======END SECURITY.TXT V1=======\0";

// -----------------------------------------------------------------------------
// State Layouts (Fixed Size, Zero Deserialization Overhead)
// -----------------------------------------------------------------------------
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CustomMint {
    pub is_initialized: u8,
    pub decimals: u8,
    pub fee_bps: u16,           // Transfer fee in basis points (e.g. 50 = 0.5%)
    pub _padding: [u8; 4],
    pub supply: u64,
    pub mint_authority: [u8; 32],
    pub freeze_authority: [u8; 32],
}

impl CustomMint {
    pub const LEN: usize = 80;
}
pub const MINT_SIZE: usize = CustomMint::LEN;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CustomAccount {
    pub is_initialized: u8,
    pub is_frozen: u8,
    pub _padding: [u8; 6],
    pub amount: u64,
    pub mint: [u8; 32],
    pub owner: [u8; 32],
}

impl CustomAccount {
    pub const LEN: usize = 80;
}
pub const ACCOUNT_SIZE: usize = CustomAccount::LEN;

// -----------------------------------------------------------------------------
// Sealevel Direct Buffer Parser
// -----------------------------------------------------------------------------
#[derive(Clone, Copy)]
pub struct AccountView {
    pub is_signer: bool,
    pub is_writable: bool,
    pub pubkey: *const [u8; 32],
    pub owner: *const [u8; 32],
    pub lamports: *mut u64,
    pub data_len: usize,
    pub data: *mut u8,
}

pub const SUCCESS: u64 = 0;
pub const ERR_INVALID_INSTRUCTION: u64 = 1;
pub const ERR_NOT_ENOUGH_ACCOUNTS: u64 = 2;
pub const ERR_MISSING_SIGNER: u64 = 3;
pub const ERR_ALREADY_INITIALIZED: u64 = 4;
pub const ERR_UNINITIALIZED: u64 = 5;
pub const ERR_INSUFFICIENT_FUNDS: u64 = 6;
pub const ERR_ACCOUNT_FROZEN: u64 = 7;
pub const ERR_UNAUTHORIZED: u64 = 8;
pub const ERR_OVERFLOW: u64 = 9;

#[cfg_attr(target_os = "solana", no_mangle)]
pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
    let num_accounts = *(input as *const u64) as usize;
    let mut offset = 8;
    
    let mut accounts: [AccountView; 8] = [AccountView {
        is_signer: false,
        is_writable: false,
        pubkey: core::ptr::null(),
        owner: core::ptr::null(),
        lamports: core::ptr::null_mut(),
        data_len: 0,
        data: core::ptr::null_mut(),
    }; 8];

    let count = if num_accounts > 8 { 8 } else { num_accounts };
    for i in 0..count {
        let dup = *input.add(offset);
        if dup == 0xff {
            accounts[i] = AccountView {
                is_signer: *input.add(offset + 1) != 0,
                is_writable: *input.add(offset + 2) != 0,
                pubkey: input.add(offset + 8) as *const [u8; 32],
                owner: input.add(offset + 40) as *const [u8; 32],
                lamports: input.add(offset + 72) as *mut u64,
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
        0 => init_mint(accounts, &instruction_data[1..]),
        1 => init_account(accounts),
        2 => mint_to(accounts, &instruction_data[1..]),
        3 => transfer(accounts, &instruction_data[1..]),
        4 => burn(accounts, &instruction_data[1..]),
        5 => freeze_thaw(accounts, true),
        6 => freeze_thaw(accounts, false),
        _ => ERR_INVALID_INSTRUCTION,
    }
}

// 0: InitializeMint { decimals: u8, fee_bps: u16 }
pub unsafe fn init_mint(accounts: &[AccountView], data: &[u8]) -> u64 {
    if accounts.len() < 2 || data.len() < 3 {
        return ERR_NOT_ENOUGH_ACCOUNTS;
    }
    let mint_acc = &accounts[0];
    let authority = &accounts[1];

    if !authority.is_signer {
        return ERR_MISSING_SIGNER;
    }
    if mint_acc.data_len < MINT_SIZE {
        return ERR_UNINITIALIZED;
    }

    let mint = &mut *(mint_acc.data as *mut CustomMint);
    if mint.is_initialized != 0 {
        return ERR_ALREADY_INITIALIZED;
    }

    mint.is_initialized = 1;
    mint.decimals = data[0];
    mint.fee_bps = u16::from_le_bytes([data[1], data[2]]);
    mint._padding = [0; 4];
    mint.supply = 0;
    mint.mint_authority = *authority.pubkey;
    mint.freeze_authority = *authority.pubkey;

    log("CustomToken: Mint Initialized");
    SUCCESS
}

// 1: InitializeAccount
pub unsafe fn init_account(accounts: &[AccountView]) -> u64 {
    if accounts.len() < 3 {
        return ERR_NOT_ENOUGH_ACCOUNTS;
    }
    let tok_acc = &accounts[0];
    let mint_acc = &accounts[1];
    let owner_acc = &accounts[2];

    if tok_acc.data_len < ACCOUNT_SIZE {
        return ERR_UNINITIALIZED;
    }

    let acc = &mut *(tok_acc.data as *mut CustomAccount);
    if acc.is_initialized != 0 {
        return ERR_ALREADY_INITIALIZED;
    }

    acc.is_initialized = 1;
    acc.is_frozen = 0;
    acc._padding = [0; 6];
    acc.amount = 0;
    acc.mint = *mint_acc.pubkey;
    acc.owner = *owner_acc.pubkey;

    log("CustomToken: Account Initialized");
    SUCCESS
}

// 2: MintTo { amount: u64 }
pub unsafe fn mint_to(accounts: &[AccountView], data: &[u8]) -> u64 {
    if accounts.len() < 3 || data.len() < 8 {
        return ERR_NOT_ENOUGH_ACCOUNTS;
    }
    let mint_acc = &accounts[0];
    let dst_acc = &accounts[1];
    let authority = &accounts[2];

    if !authority.is_signer {
        return ERR_MISSING_SIGNER;
    }
    if mint_acc.data_len < MINT_SIZE || dst_acc.data_len < ACCOUNT_SIZE {
        return ERR_UNINITIALIZED;
    }

    let mint = &mut *(mint_acc.data as *mut CustomMint);
    let dst = &mut *(dst_acc.data as *mut CustomAccount);

    if mint.is_initialized == 0 || dst.is_initialized == 0 {
        return ERR_UNINITIALIZED;
    }
    if dst.is_frozen != 0 {
        return ERR_ACCOUNT_FROZEN;
    }
    if mint.mint_authority != *authority.pubkey {
        return ERR_UNAUTHORIZED;
    }

    let amount = u64::from_le_bytes(*(data.as_ptr() as *const [u8; 8]));
    mint.supply = match mint.supply.checked_add(amount) {
        Some(v) => v,
        None => return ERR_OVERFLOW,
    };
    dst.amount = match dst.amount.checked_add(amount) {
        Some(v) => v,
        None => return ERR_OVERFLOW,
    };

    log("CustomToken: MintTo Completed");
    SUCCESS
}

// 3: Transfer { amount: u64 }
pub unsafe fn transfer(accounts: &[AccountView], data: &[u8]) -> u64 {
    if accounts.len() < 4 || data.len() < 8 {
        return ERR_NOT_ENOUGH_ACCOUNTS;
    }
    let src_acc = &accounts[0];
    let dst_acc = &accounts[1];
    let mint_acc = &accounts[2];
    let owner = &accounts[3];

    if !owner.is_signer {
        return ERR_MISSING_SIGNER;
    }
    if src_acc.data_len < ACCOUNT_SIZE || dst_acc.data_len < ACCOUNT_SIZE || mint_acc.data_len < MINT_SIZE {
        return ERR_UNINITIALIZED;
    }

    let src = &mut *(src_acc.data as *mut CustomAccount);
    let dst = &mut *(dst_acc.data as *mut CustomAccount);
    let mint = &*(mint_acc.data as *const CustomMint);

    if src.is_initialized == 0 || dst.is_initialized == 0 || mint.is_initialized == 0 {
        return ERR_UNINITIALIZED;
    }
    if src.is_frozen != 0 || dst.is_frozen != 0 {
        return ERR_ACCOUNT_FROZEN;
    }
    if src.owner != *owner.pubkey {
        return ERR_UNAUTHORIZED;
    }

    let amount = u64::from_le_bytes(*(data.as_ptr() as *const [u8; 8]));
    if src.amount < amount {
        return ERR_INSUFFICIENT_FUNDS;
    }

    let fee = if mint.fee_bps > 0 {
        (amount as u128 * mint.fee_bps as u128 / 10000) as u64
    } else {
        0
    };
    let received = amount - fee;

    src.amount -= amount;
    dst.amount = match dst.amount.checked_add(received) {
        Some(v) => v,
        None => return ERR_OVERFLOW,
    };

    log("CustomToken: Transfer with Fee Completed");
    SUCCESS
}

// 4: Burn { amount: u64 }
pub unsafe fn burn(accounts: &[AccountView], data: &[u8]) -> u64 {
    if accounts.len() < 3 || data.len() < 8 {
        return ERR_NOT_ENOUGH_ACCOUNTS;
    }
    let src_acc = &accounts[0];
    let mint_acc = &accounts[1];
    let owner = &accounts[2];

    if !owner.is_signer {
        return ERR_MISSING_SIGNER;
    }
    if src_acc.data_len < ACCOUNT_SIZE || mint_acc.data_len < MINT_SIZE {
        return ERR_UNINITIALIZED;
    }

    let src = &mut *(src_acc.data as *mut CustomAccount);
    let mint = &mut *(mint_acc.data as *mut CustomMint);

    if src.is_initialized == 0 || mint.is_initialized == 0 {
        return ERR_UNINITIALIZED;
    }
    if src.owner != *owner.pubkey {
        return ERR_UNAUTHORIZED;
    }

    let amount = u64::from_le_bytes(*(data.as_ptr() as *const [u8; 8]));
    if src.amount < amount {
        return ERR_INSUFFICIENT_FUNDS;
    }

    src.amount -= amount;
    mint.supply -= amount;

    log("CustomToken: Burn Completed");
    SUCCESS
}

// 5: Freeze / 6: Thaw
pub unsafe fn freeze_thaw(accounts: &[AccountView], freeze: bool) -> u64 {
    if accounts.len() < 3 {
        return ERR_NOT_ENOUGH_ACCOUNTS;
    }
    let target_acc = &accounts[0];
    let mint_acc = &accounts[1];
    let authority = &accounts[2];

    if !authority.is_signer {
        return ERR_MISSING_SIGNER;
    }
    if target_acc.data_len < ACCOUNT_SIZE || mint_acc.data_len < MINT_SIZE {
        return ERR_UNINITIALIZED;
    }

    let target = &mut *(target_acc.data as *mut CustomAccount);
    let mint = &*(mint_acc.data as *const CustomMint);

    if target.is_initialized == 0 || mint.is_initialized == 0 {
        return ERR_UNINITIALIZED;
    }
    if mint.freeze_authority != *authority.pubkey {
        return ERR_UNAUTHORIZED;
    }

    target.is_frozen = if freeze { 1 } else { 0 };
    SUCCESS
}

#[cfg(test)]
#[allow(static_mut_refs)]
mod tests {
    use super::*;

    #[test]
    fn test_token_lifecycle() {
        let auth_pubkey = [1u8; 32];
        let owner_a = [2u8; 32];
        let owner_b = [3u8; 32];
        let mint_pubkey = [4u8; 32];

        let mut mint_data = [0u8; 80];
        let mut acc_a_data = [0u8; 80];
        let mut acc_b_data = [0u8; 80];

        unsafe {
            // 1. Initialize Mint with 100 bps (1%) fee
            let mint_view = AccountView {
                is_signer: false,
                is_writable: true,
                pubkey: &mint_pubkey,
                owner: &mint_pubkey,
                lamports: core::ptr::null_mut(),
                data_len: 80,
                data: mint_data.as_mut_ptr(),
            };
            let auth_view = AccountView {
                is_signer: true,
                is_writable: false,
                pubkey: &auth_pubkey,
                owner: &auth_pubkey,
                lamports: core::ptr::null_mut(),
                data_len: 0,
                data: core::ptr::null_mut(),
            };
            let res = init_mint(&[mint_view, auth_view], &[6, 100, 0]); // 6 decimals, 100 bps
            assert_eq!(res, SUCCESS);

            // 2. Initialize Account A & Account B
            let acc_a_view = AccountView {
                is_signer: false,
                is_writable: true,
                pubkey: &[10u8; 32],
                owner: &owner_a,
                lamports: core::ptr::null_mut(),
                data_len: 80,
                data: acc_a_data.as_mut_ptr(),
            };
            let acc_b_view = AccountView {
                is_signer: false,
                is_writable: true,
                pubkey: &[11u8; 32],
                owner: &owner_b,
                lamports: core::ptr::null_mut(),
                data_len: 80,
                data: acc_b_data.as_mut_ptr(),
            };
            let owner_a_view = AccountView {
                is_signer: true,
                is_writable: false,
                pubkey: &owner_a,
                owner: &owner_a,
                lamports: core::ptr::null_mut(),
                data_len: 0,
                data: core::ptr::null_mut(),
            };
            let owner_b_view = AccountView {
                is_signer: true,
                is_writable: false,
                pubkey: &owner_b,
                owner: &owner_b,
                lamports: core::ptr::null_mut(),
                data_len: 0,
                data: core::ptr::null_mut(),
            };

            assert_eq!(init_account(&[acc_a_view, mint_view, owner_a_view]), SUCCESS);
            assert_eq!(init_account(&[acc_b_view, mint_view, owner_b_view]), SUCCESS);

            // 3. Mint 1,000 tokens to Account A
            let mint_amount: u64 = 1000;
            assert_eq!(mint_to(&[mint_view, acc_a_view, auth_view], &mint_amount.to_le_bytes()), SUCCESS);
            
            let acc_a = &*(acc_a_data.as_ptr() as *const CustomAccount);
            assert_eq!(acc_a.amount, 1000);

            // 4. Transfer 500 tokens from A to B (1% fee = 5 tokens deducted, B gets 495)
            let xfer_amount: u64 = 500;
            assert_eq!(transfer(&[acc_a_view, acc_b_view, mint_view, owner_a_view], &xfer_amount.to_le_bytes()), SUCCESS);

            let acc_a = &*(acc_a_data.as_ptr() as *const CustomAccount);
            let acc_b = &*(acc_b_data.as_ptr() as *const CustomAccount);
            assert_eq!(acc_a.amount, 500);
            assert_eq!(acc_b.amount, 495);

            // 5. Burn 200 tokens from A
            let burn_amount: u64 = 200;
            assert_eq!(burn(&[acc_a_view, mint_view, owner_a_view], &burn_amount.to_le_bytes()), SUCCESS);
            let acc_a = &*(acc_a_data.as_ptr() as *const CustomAccount);
            assert_eq!(acc_a.amount, 300);

            // 6. Freeze Account B and verify transfer fails
            assert_eq!(freeze_thaw(&[acc_b_view, mint_view, auth_view], true), SUCCESS);
            assert_eq!(transfer(&[acc_b_view, acc_a_view, mint_view, owner_b_view], &100u64.to_le_bytes()), ERR_ACCOUNT_FROZEN);

            // 7. Thaw Account B
            assert_eq!(freeze_thaw(&[acc_b_view, mint_view, auth_view], false), SUCCESS);
            assert_eq!(transfer(&[acc_b_view, acc_a_view, mint_view, owner_b_view], &100u64.to_le_bytes()), SUCCESS);
        }
    }
}
