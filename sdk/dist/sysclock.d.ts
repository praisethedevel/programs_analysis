import { PublicKey, TransactionInstruction, Connection } from '@solana/web3.js';
export declare const SYSCLOCK_PROGRAM_ID: PublicKey;
export declare const CLOCK_STATE_SIZE = 72;
export interface SysclockState {
    isInitialized: boolean;
    isFrozen: boolean;
    slot: bigint;
    epoch: bigint;
    unixTimestampMs: bigint;
    tickSequence: bigint;
    authority: PublicKey;
}
export declare class CustomSysclockClient {
    readonly connection: Connection;
    readonly programId: PublicKey;
    constructor(connection: Connection, programId?: PublicKey);
    createInitClockInstruction(clockPubkey: PublicKey, authorityPubkey: PublicKey): TransactionInstruction;
    createTickInstruction(clockPubkey: PublicKey, authorityPubkey: PublicKey, slot: bigint, unixTimestampMs: bigint, advanceEpoch: boolean): TransactionInstruction;
    createReadClockInstruction(clockPubkey: PublicKey): TransactionInstruction;
    fetchClock(clockPubkey: PublicKey): Promise<SysclockState | null>;
}
