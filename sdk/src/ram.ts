import {
  PublicKey,
  TransactionInstruction,
  Connection
} from '@solana/web3.js';

export const RAM_PROGRAM_ID = new PublicKey('5oFB4c7Answx5c2reA9w5u3DGcE1HHw2PfhkXTRFvc3m');
export const RAM_HEADER_SIZE = 40;

export interface RamHeaderState {
  isInitialized: boolean;
  isLocked: boolean;
  allocatedSize: number;
  owner: PublicKey;
}

export class CustomRAMClient {
  constructor(
    public readonly connection: Connection,
    public readonly programId: PublicKey = RAM_PROGRAM_ID
  ) {}

  public createInitRamInstruction(
    ramPubkey: PublicKey,
    ownerPubkey: PublicKey
  ): TransactionInstruction {
    return new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: ramPubkey, isSigner: false, isWritable: true },
        { pubkey: ownerPubkey, isSigner: true, isWritable: false }
      ],
      data: Buffer.from([0]) // Tag 0 = InitRam
    });
  }

  public createWriteRamInstruction(
    ramPubkey: PublicKey,
    ownerPubkey: PublicKey,
    offset: number,
    payload: Uint8Array | Buffer
  ): TransactionInstruction {
    const data = Buffer.alloc(5 + payload.length);
    data.writeUInt8(1, 0); // Tag 1 = WriteRam
    data.writeUInt32LE(offset, 1);
    Buffer.from(payload).copy(data, 5);

    return new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: ramPubkey, isSigner: false, isWritable: true },
        { pubkey: ownerPubkey, isSigner: true, isWritable: false }
      ],
      data
    });
  }

  public createLockRamInstruction(
    ramPubkey: PublicKey,
    ownerPubkey: PublicKey
  ): TransactionInstruction {
    return new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: ramPubkey, isSigner: false, isWritable: true },
        { pubkey: ownerPubkey, isSigner: true, isWritable: false }
      ],
      data: Buffer.from([2]) // Tag 2 = LockRam
    });
  }

  public async fetchRamHeader(ramPubkey: PublicKey): Promise<RamHeaderState | null> {
    const info = await this.connection.getAccountInfo(ramPubkey);
    if (!info || info.data.length < RAM_HEADER_SIZE) return null;
    const buf = Buffer.from(info.data);

    return {
      isInitialized: buf.readUInt8(0) === 1,
      isLocked: buf.readUInt8(1) === 1,
      allocatedSize: buf.readUInt32LE(4),
      owner: new PublicKey(buf.subarray(8, 40))
    };
  }

  public async readMemoryRange(
    ramPubkey: PublicKey,
    offset: number,
    length: number
  ): Promise<Buffer | null> {
    const info = await this.connection.getAccountInfo(ramPubkey);
    if (!info) return null;
    const start = RAM_HEADER_SIZE + offset;
    const end = start + length;
    if (end > info.data.length) return null;
    return Buffer.from(info.data.subarray(start, end));
  }
}
