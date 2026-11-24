"use strict";
// SPDX-License-Identifier: PACARL-1.0
// Copyright (c) 2026 praisethedevel <xsolutionx@gmail.com>. All Rights Reserved.
// Governed by the Praisethedevel Universal Architecture, Compulsory Attribution & Commercial Royalty License.
// Non-commercial / evaluation use only. Commercial exploitation strictly requires prior written agreement.
Object.defineProperty(exports, "__esModule", { value: true });
exports.CustomTokenClient = exports.ACCOUNT_SIZE = exports.MINT_SIZE = exports.TOKEN_PROGRAM_ID = void 0;
const web3_js_1 = require("@solana/web3.js");
exports.TOKEN_PROGRAM_ID = new web3_js_1.PublicKey('CShaS9ezrJoEYBXsAk4bsUzL2pbxE14kAadgBCeXS7Z1');
exports.MINT_SIZE = 80;
exports.ACCOUNT_SIZE = 80;
class CustomTokenClient {
    connection;
    programId;
    constructor(connection, programId = exports.TOKEN_PROGRAM_ID) {
        this.connection = connection;
        this.programId = programId;
    }
    createInitMintInstruction(mintPubkey, payerPubkey, decimals, feeBasisPoints) {
        const data = Buffer.alloc(4);
        data.writeUInt8(0, 0); // Tag 0 = InitializeMint
        data.writeUInt8(decimals, 1);
        data.writeUInt16LE(feeBasisPoints, 2);
        return new web3_js_1.TransactionInstruction({
            programId: this.programId,
            keys: [
                { pubkey: mintPubkey, isSigner: false, isWritable: true },
                { pubkey: payerPubkey, isSigner: true, isWritable: false }
            ],
            data
        });
    }
    createInitAccountInstruction(accountPubkey, mintPubkey, ownerPubkey) {
        return new web3_js_1.TransactionInstruction({
            programId: this.programId,
            keys: [
                { pubkey: accountPubkey, isSigner: false, isWritable: true },
                { pubkey: mintPubkey, isSigner: false, isWritable: false },
                { pubkey: ownerPubkey, isSigner: false, isWritable: false }
            ],
            data: Buffer.from([1]) // Tag 1 = InitializeAccount
        });
    }
    createMintToInstruction(mintPubkey, destinationPubkey, authorityPubkey, amount) {
        const data = Buffer.alloc(9);
        data.writeUInt8(2, 0); // Tag 2 = MintTo
        data.writeBigUInt64LE(amount, 1);
        return new web3_js_1.TransactionInstruction({
            programId: this.programId,
            keys: [
                { pubkey: mintPubkey, isSigner: false, isWritable: true },
                { pubkey: destinationPubkey, isSigner: false, isWritable: true },
                { pubkey: authorityPubkey, isSigner: true, isWritable: false }
            ],
            data
        });
    }
    createTransferInstruction(sourcePubkey, destinationPubkey, mintPubkey, authorityPubkey, amount) {
        const data = Buffer.alloc(9);
        data.writeUInt8(3, 0); // Tag 3 = Transfer
        data.writeBigUInt64LE(amount, 1);
        return new web3_js_1.TransactionInstruction({
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
    createBurnInstruction(accountPubkey, mintPubkey, authorityPubkey, amount) {
        const data = Buffer.alloc(9);
        data.writeUInt8(4, 0); // Tag 4 = Burn
        data.writeBigUInt64LE(amount, 1);
        return new web3_js_1.TransactionInstruction({
            programId: this.programId,
            keys: [
                { pubkey: accountPubkey, isSigner: false, isWritable: true },
                { pubkey: mintPubkey, isSigner: false, isWritable: true },
                { pubkey: authorityPubkey, isSigner: true, isWritable: false }
            ],
            data
        });
    }
    createFreezeInstruction(accountPubkey, mintPubkey, authorityPubkey) {
        return new web3_js_1.TransactionInstruction({
            programId: this.programId,
            keys: [
                { pubkey: accountPubkey, isSigner: false, isWritable: true },
                { pubkey: mintPubkey, isSigner: false, isWritable: false },
                { pubkey: authorityPubkey, isSigner: true, isWritable: false }
            ],
            data: Buffer.from([5]) // Tag 5 = FreezeAccount
        });
    }
    createThawInstruction(accountPubkey, mintPubkey, authorityPubkey) {
        return new web3_js_1.TransactionInstruction({
            programId: this.programId,
            keys: [
                { pubkey: accountPubkey, isSigner: false, isWritable: true },
                { pubkey: mintPubkey, isSigner: false, isWritable: false },
                { pubkey: authorityPubkey, isSigner: true, isWritable: false }
            ],
            data: Buffer.from([6]) // Tag 6 = ThawAccount
        });
    }
    async fetchMint(mintPubkey) {
        const info = await this.connection.getAccountInfo(mintPubkey);
        if (!info || info.data.length < exports.MINT_SIZE)
            return null;
        const buf = Buffer.from(info.data);
        return {
            mintAuthority: new web3_js_1.PublicKey(buf.subarray(0, 32)),
            supply: buf.readBigUInt64LE(32),
            decimals: buf.readUInt8(40),
            isInitialized: buf.readUInt8(41) === 1,
            feeBasisPoints: buf.readUInt16LE(42),
            freezeAuthority: new web3_js_1.PublicKey(buf.subarray(44, 76))
        };
    }
    async fetchAccount(accountPubkey) {
        const info = await this.connection.getAccountInfo(accountPubkey);
        if (!info || info.data.length < exports.ACCOUNT_SIZE)
            return null;
        const buf = Buffer.from(info.data);
        return {
            mint: new web3_js_1.PublicKey(buf.subarray(0, 32)),
            owner: new web3_js_1.PublicKey(buf.subarray(32, 64)),
            amount: buf.readBigUInt64LE(64),
            state: buf.readUInt8(72)
        };
    }
}
exports.CustomTokenClient = CustomTokenClient;
