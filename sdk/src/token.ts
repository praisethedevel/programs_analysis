// SPDX-License-Identifier: PACARL-1.0
// Copyright (c) 2026 praisethedevel <xsolutionx@gmail.com>. All Rights Reserved.
// Governed by the Praisethedevel Universal Architecture, Compulsory Attribution & Commercial Royalty License.
// Non-commercial / evaluation use only. Commercial exploitation strictly requires prior written agreement.

import {
  PublicKey,
  TransactionInstruction,
  Connection,
  AccountInfo
} from '@solana/web3.js';

export const TOKEN_PROGRAM_ID = new PublicKey('CShaS9ezrJoEYBXsAk4bsUzL2pbxE14kAadgBCeXS7Z1');
export const MINT_SIZE = 80;
export const ACCOUNT_SIZE = 80;

export interface MintState {
  mintAuthority: PublicKey;
  supply: bigint;
  decimals: number;
  isInitialized: boolean;
  feeBasisPoints: number;
  freezeAuthority: PublicKey;
}

export interface TokenAccountState {
  mint: PublicKey;
  owner: PublicKey;
  amount: bigint;
  state: number; // 0 = uninit, 1 = active, 2 = frozen
}

export class CustomTokenClient {
  constructor(
    public readonly connection: Connection,
    public readonly programId: PublicKey = TOKEN_PROGRAM_ID
  ) {}

  public createInitMintInstruction(
    mintPubkey: PublicKey,
    payerPubkey: PublicKey,
    decimals: number,
    feeBasisPoints: number
  ): TransactionInstruction {
    const data = Buffer.alloc(4);
    data.writeUInt8(0, 0); // Tag 0 = InitializeMint
    data.writeUInt8(decimals, 1);
    data.writeUInt16LE(feeBasisPoints, 2);

    return new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: mintPubkey, isSigner: false, isWritable: true },
        { pubkey: payerPubkey, isSigner: true, isWritable: false }
      ],
      data
    });
  }

  public createInitAccountInstruction(
    accountPubkey: PublicKey,
    mintPubkey: PublicKey,
    ownerPubkey: PublicKey
  ): TransactionInstruction {
    return new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: accountPubkey, isSigner: false, isWritable: true },
        { pubkey: mintPubkey, isSigner: false, isWritable: false },
        { pubkey: ownerPubkey, isSigner: false, isWritable: false }
      ],
      data: Buffer.from([1]) // Tag 1 = InitializeAccount
    });
  }

  public createMintToInstruction(
    mintPubkey: PublicKey,
    destinationPubkey: PublicKey,
    authorityPubkey: PublicKey,
    amount: bigint
  ): TransactionInstruction {
    const data = Buffer.alloc(9);
    data.writeUInt8(2, 0); // Tag 2 = MintTo
    data.writeBigUInt64LE(amount, 1);

    return new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: mintPubkey, isSigner: false, isWritable: true },
        { pubkey: destinationPubkey, isSigner: false, isWritable: true },
        { pubkey: authorityPubkey, isSigner: true, isWritable: false }
      ],
      data
    });
  }

  public createTransferInstruction(
    sourcePubkey: PublicKey,
    destinationPubkey: PublicKey,
    mintPubkey: PublicKey,
    authorityPubkey: PublicKey,
    amount: bigint
  ): TransactionInstruction {
    const data = Buffer.alloc(9);
    data.writeUInt8(3, 0); // Tag 3 = Transfer
    data.writeBigUInt64LE(amount, 1);

    return new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: sourcePubkey, isSigner: false, isWritable: true },
        { pubkey: destinationPubkey, isSigner: false, isWritable: true },
        { pubkey: mintPubkey, isSigner: false, isWritable: false },
        { pubkey: authorityPubkey, isSigner: true, isWritable: false }
      ],
      data
    });
  }

  public createBurnInstruction(
    accountPubkey: PublicKey,
    mintPubkey: PublicKey,
    authorityPubkey: PublicKey,
    amount: bigint
  ): TransactionInstruction {
    const data = Buffer.alloc(9);
    data.writeUInt8(4, 0); // Tag 4 = Burn
    data.writeBigUInt64LE(amount, 1);

    return new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: accountPubkey, isSigner: false, isWritable: true },
        { pubkey: mintPubkey, isSigner: false, isWritable: true },
        { pubkey: authorityPubkey, isSigner: true, isWritable: false }
      ],
      data
    });
  }

  public createFreezeInstruction(
    accountPubkey: PublicKey,
    mintPubkey: PublicKey,
    authorityPubkey: PublicKey
  ): TransactionInstruction {
    return new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: accountPubkey, isSigner: false, isWritable: true },
        { pubkey: mintPubkey, isSigner: false, isWritable: false },
        { pubkey: authorityPubkey, isSigner: true, isWritable: false }
      ],
      data: Buffer.from([5]) // Tag 5 = FreezeAccount
    });
  }

  public createThawInstruction(
    accountPubkey: PublicKey,
    mintPubkey: PublicKey,
    authorityPubkey: PublicKey
  ): TransactionInstruction {
    return new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: accountPubkey, isSigner: false, isWritable: true },
        { pubkey: mintPubkey, isSigner: false, isWritable: false },
        { pubkey: authorityPubkey, isSigner: true, isWritable: false }
      ],
      data: Buffer.from([6]) // Tag 6 = ThawAccount
    });
  }

  public async fetchMint(mintPubkey: PublicKey): Promise<MintState | null> {
    const info = await this.connection.getAccountInfo(mintPubkey);
    if (!info || info.data.length < MINT_SIZE) return null;
    const buf = Buffer.from(info.data);

    return {
      mintAuthority: new PublicKey(buf.subarray(0, 32)),
      supply: buf.readBigUInt64LE(32),
      decimals: buf.readUInt8(40),
      isInitialized: buf.readUInt8(41) === 1,
      feeBasisPoints: buf.readUInt16LE(42),
      freezeAuthority: new PublicKey(buf.subarray(44, 76))
    };
  }

  public async fetchAccount(accountPubkey: PublicKey): Promise<TokenAccountState | null> {
    const info = await this.connection.getAccountInfo(accountPubkey);
    if (!info || info.data.length < ACCOUNT_SIZE) return null;
    const buf = Buffer.from(info.data);

    return {
      mint: new PublicKey(buf.subarray(0, 32)),
      owner: new PublicKey(buf.subarray(32, 64)),
      amount: buf.readBigUInt64LE(64),
      state: buf.readUInt8(72)
    };
  }
}
