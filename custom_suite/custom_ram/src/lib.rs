#![cfg_attr(target_os = "solana", no_std)]

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
name\0CustomRAM (On-Chain Memory Bank Rebuild)\0\
project_url\0https://github.com/praisethedevel/programs_analysis\0\
contacts\0email:xsolutionx@gmail.com,link:https://github.com/praisethedevel/programs_analysis/security\0\
policy\0https://github.com/praisethedevel/programs_analysis/blob/main/SECURITY.md\0\
preferred_languages\0en\0\
source_code\0https://github.com/praisethedevel/programs_analysis/tree/main/custom_suite/custom_ram\0\
source_release\0custom-ram@v0.1.0\0\
auditors\0Autonomous Forensic SBPF Engine\0\
expiry\02028-01-01\0\
=======END SECURITY.TXT V1=======\0";

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RamHeader {
    pub is_initialized: u8,
    pub is_locked: u8,
    pub _padding: [u8; 2],
    pub allocated_size: u32,
    pub owner: [u8; 32],
}

impl RamHeader {
    pub const LEN: usize = 40;
}
pub const RAM_HEADER_SIZE: usize = RamHeader::LEN;
pub const MIN_RAM_ACCOUNT_SIZE: usize = RamHeader::LEN;
pub const MAX_RETURN_DATA_SIZE: usize = 1024;

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
pub const ERR_OUT_OF_BOUNDS: u64 = 7;

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
        0 => init_ram(accounts),
        1 => write_ram(accounts, &instruction_data[1..]),
        2 => read_ram(accounts, &instruction_data[1..]),
        _ => ERR_INVALID_INSTRUCTION,
    }
}

// 0: InitRam
pub unsafe fn init_ram(accounts: &[AccountView]) -> u64 {
    if accounts.len() < 2 {
        return ERR_NOT_ENOUGH_ACCOUNTS;
    }
    let ram_acc = &accounts[0];
    let owner = &accounts[1];

    if !owner.is_signer {
        return ERR_MISSING_SIGNER;
    }
    if ram_acc.data_len < RAM_HEADER_SIZE {
        return ERR_UNINITIALIZED;
    }

    let header = &mut *(ram_acc.data as *mut RamHeader);
    if header.is_initialized != 0 {
        return ERR_ALREADY_INITIALIZED;
    }

    header.is_initialized = 1;
    header.is_locked = 0;
    header._padding = [0; 2];
    header.allocated_size = (ram_acc.data_len - RAM_HEADER_SIZE) as u32;
    header.owner = *owner.pubkey;

    log("CustomRAM: Initialized Memory Bank");
    SUCCESS
}

// 1: WriteRam { offset: u32, data: &[u8] }
pub unsafe fn write_ram(accounts: &[AccountView], data: &[u8]) -> u64 {
    if accounts.len() < 2 || data.len() < 4 {
        return ERR_NOT_ENOUGH_ACCOUNTS;
    }
    let ram_acc = &accounts[0];
    let owner = &accounts[1];

    if !owner.is_signer {
        return ERR_MISSING_SIGNER;
    }
    if ram_acc.data_len < RAM_HEADER_SIZE {
        return ERR_UNINITIALIZED;
    }

    let header = &*(ram_acc.data as *const RamHeader);
    if header.is_initialized == 0 {
        return ERR_UNINITIALIZED;
    }
    if header.owner != *owner.pubkey {
        return ERR_UNAUTHORIZED;
    }

    let write_offset = u32::from_le_bytes(*(data[0..4].as_ptr() as *const [u8; 4])) as usize;
    let payload = &data[4..];
    let header_size = core::mem::size_of::<RamHeader>();

    if header_size + write_offset + payload.len() > ram_acc.data_len {
        return ERR_OUT_OF_BOUNDS;
    }

    let dst = ram_acc.data.add(header_size + write_offset);
    core::ptr::copy_nonoverlapping(payload.as_ptr(), dst, payload.len());

    log("CustomRAM: Direct Memory Write Completed");
    SUCCESS
}

// 2: ReadRam { offset: u32, len: u32 }
pub unsafe fn read_ram(accounts: &[AccountView], data: &[u8]) -> u64 {
    if accounts.is_empty() || data.len() < 8 {
        return ERR_NOT_ENOUGH_ACCOUNTS;
    }
    let ram_acc = &accounts[0];
    if ram_acc.data_len < RAM_HEADER_SIZE {
        return ERR_UNINITIALIZED;
    }

    let header = &*(ram_acc.data as *const RamHeader);
    if header.is_initialized == 0 {
        return ERR_UNINITIALIZED;
    }

    let read_offset = u32::from_le_bytes(*(data[0..4].as_ptr() as *const [u8; 4])) as usize;
    let read_len = u32::from_le_bytes(*(data[4..8].as_ptr() as *const [u8; 4])) as usize;
    let header_size = core::mem::size_of::<RamHeader>();

    if header_size + read_offset + read_len > ram_acc.data_len || read_len > 1024 {
        return ERR_OUT_OF_BOUNDS;
    }

    let src = ram_acc.data.add(header_size + read_offset);
    sol_set_return_data(src, read_len as u64);

    log("CustomRAM: Data Returned to Caller");
    SUCCESS
}

#[cfg(test)]
#[allow(static_mut_refs)]
mod tests {
    use super::*;

    #[test]
    fn test_ram_lifecycle() {
        let owner_pubkey = [9u8; 32];
        let ram_pubkey = [10u8; 32];
        let mut ram_memory = [0u8; 256];

        unsafe {
            let ram_view = AccountView {
                is_signer: false,
                pubkey: &ram_pubkey,
                data_len: 256,
                data: ram_memory.as_mut_ptr(),
            };
            let owner_view = AccountView {
                is_signer: true,
                pubkey: &owner_pubkey,
                data_len: 0,
                data: core::ptr::null_mut(),
            };

            // Init RAM bank
            assert_eq!(init_ram(&[ram_view, owner_view]), SUCCESS);

            // Write 16 bytes at offset 0
            let payload = b"HELLO SOLANA RAM";
            let mut write_data = [0u8; 20];
            write_data[0..4].copy_from_slice(&0u32.to_le_bytes()); // offset 0
            write_data[4..20].copy_from_slice(payload);
            assert_eq!(write_ram(&[ram_view, owner_view], &write_data), SUCCESS);

            // Read back 16 bytes
            let mut read_data = [0u8; 8];
            read_data[0..4].copy_from_slice(&0u32.to_le_bytes()); // offset 0
            read_data[4..8].copy_from_slice(&16u32.to_le_bytes()); // len 16
            assert_eq!(read_ram(&[ram_view], &read_data), SUCCESS);

            assert_eq!(RETURN_DATA_LEN, 16);
            assert_eq!(&RETURN_DATA_BUF[..16], payload);

            // Test Out of Bounds Write
            let mut oob_write = [0u8; 8];
            oob_write[0..4].copy_from_slice(&250u32.to_le_bytes()); // offset 250 (out of bounds for 256 - 40 bytes)
            oob_write[4..8].copy_from_slice(b"FAIL");
            assert_eq!(write_ram(&[ram_view, owner_view], &oob_write), ERR_OUT_OF_BOUNDS);
        }
    }
}
