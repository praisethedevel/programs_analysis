import {
  PublicKey,
  TransactionInstruction,
  Connection
} from '@solana/web3.js';

export const SYSCLOCK_PROGRAM_ID = new PublicKey('8VB675MfjjiaizM2VDBRQfk8GMKCiZVtMELqGdstgMTK');
export const CLOCK_STATE_SIZE = 72;

export interface SysclockState {
  isInitialized: boolean;
  isFrozen: boolean;
  slot: bigint;
  epoch: bigint;
  unixTimestampMs: bigint;
  tickSequence: bigint;
  authority: PublicKey;
}

export class CustomSysclockClient {
  constructor(
    public readonly connection: Connection,
    public readonly programId: PublicKey = SYSCLOCK_PROGRAM_ID
  ) {}

  public createInitClockInstruction(
    clockPubkey: PublicKey,
    authorityPubkey: PublicKey
  ): TransactionInstruction {
    return new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: clockPubkey, isSigner: false, isWritable: true },
        { pubkey: authorityPubkey, isSigner: true, isWritable: false }
      ],
      data: Buffer.from([0]) // Tag 0 = InitializeClock
    });
  }

  public createTickInstruction(
    clockPubkey: PublicKey,
    authorityPubkey: PublicKey,
    slot: bigint,
    unixTimestampMs: bigint,
    advanceEpoch: boolean
  ): TransactionInstruction {
    const data = Buffer.alloc(18);
    data.writeUInt8(1, 0); // Tag 1 = Tick
    data.writeBigUInt64LE(slot, 1);
    data.writeBigInt64LE(unixTimestampMs, 9);
    data.writeUInt8(advanceEpoch ? 1 : 0, 17);

    return new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: clockPubkey, isSigner: false, isWritable: true },
        { pubkey: authorityPubkey, isSigner: true, isWritable: false }
      ],
      data
    });
  }

  public createReadClockInstruction(clockPubkey: PublicKey): TransactionInstruction {
    return new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: clockPubkey, isSigner: false, isWritable: false }
      ],
      data: Buffer.from([2]) // Tag 2 = ReadClock
    });
  }

  public async fetchClock(clockPubkey: PublicKey): Promise<SysclockState | null> {
    const info = await this.connection.getAccountInfo(clockPubkey);
    if (!info || info.data.length < CLOCK_STATE_SIZE) return null;
    const buf = Buffer.from(info.data);

    return {
      isInitialized: buf.readUInt8(0) === 1,
      isFrozen: buf.readUInt8(1) === 1,
      slot: buf.readBigUInt64LE(8),
      epoch: buf.readBigUInt64LE(16),
      unixTimestampMs: buf.readBigInt64LE(24),
      tickSequence: buf.readBigUInt64LE(32),
      authority: new PublicKey(buf.subarray(40, 72))
    };
  }
}
