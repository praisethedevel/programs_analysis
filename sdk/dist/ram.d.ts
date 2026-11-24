import { PublicKey, TransactionInstruction, Connection } from '@solana/web3.js';
export declare const RAM_PROGRAM_ID: PublicKey;
export declare const RAM_HEADER_SIZE = 40;
export interface RamHeaderState {
    isInitialized: boolean;
    isLocked: boolean;
    allocatedSize: number;
    owner: PublicKey;
}
export declare class CustomRAMClient {
    readonly connection: Connection;
    readonly programId: PublicKey;
    constructor(connection: Connection, programId?: PublicKey);
    createInitRamInstruction(ramPubkey: PublicKey, ownerPubkey: PublicKey): TransactionInstruction;
    createWriteRamInstruction(ramPubkey: PublicKey, ownerPubkey: PublicKey, offset: number, payload: Uint8Array | Buffer): TransactionInstruction;
    createLockRamInstruction(ramPubkey: PublicKey, ownerPubkey: PublicKey): TransactionInstruction;
    fetchRamHeader(ramPubkey: PublicKey): Promise<RamHeaderState | null>;
    readMemoryRange(ramPubkey: PublicKey, offset: number, length: number): Promise<Buffer | null>;
}
