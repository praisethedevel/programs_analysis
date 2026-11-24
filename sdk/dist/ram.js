"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.CustomRAMClient = exports.RAM_HEADER_SIZE = exports.RAM_PROGRAM_ID = void 0;
const web3_js_1 = require("@solana/web3.js");
exports.RAM_PROGRAM_ID = new web3_js_1.PublicKey('5oFB4c7Answx5c2reA9w5u3DGcE1HHw2PfhkXTRFvc3m');
exports.RAM_HEADER_SIZE = 40;
class CustomRAMClient {
    connection;
    programId;
    constructor(connection, programId = exports.RAM_PROGRAM_ID) {
        this.connection = connection;
        this.programId = programId;
    }
    createInitRamInstruction(ramPubkey, ownerPubkey) {
        return new web3_js_1.TransactionInstruction({
            programId: this.programId,
            keys: [
                { pubkey: ramPubkey, isSigner: false, isWritable: true },
                { pubkey: ownerPubkey, isSigner: true, isWritable: false }
            ],
            data: Buffer.from([0]) // Tag 0 = InitRam
        });
    }
    createWriteRamInstruction(ramPubkey, ownerPubkey, offset, payload) {
        const data = Buffer.alloc(5 + payload.length);
        data.writeUInt8(1, 0); // Tag 1 = WriteRam
        data.writeUInt32LE(offset, 1);
        Buffer.from(payload).copy(data, 5);
        return new web3_js_1.TransactionInstruction({
            programId: this.programId,
            keys: [
                { pubkey: ramPubkey, isSigner: false, isWritable: true },
                { pubkey: ownerPubkey, isSigner: true, isWritable: false }
            ],
            data
        });
    }
    createLockRamInstruction(ramPubkey, ownerPubkey) {
        return new web3_js_1.TransactionInstruction({
            programId: this.programId,
            keys: [
                { pubkey: ramPubkey, isSigner: false, isWritable: true },
                { pubkey: ownerPubkey, isSigner: true, isWritable: false }
            ],
            data: Buffer.from([2]) // Tag 2 = LockRam
        });
    }
    async fetchRamHeader(ramPubkey) {
        const info = await this.connection.getAccountInfo(ramPubkey);
        if (!info || info.data.length < exports.RAM_HEADER_SIZE)
            return null;
        const buf = Buffer.from(info.data);
        return {
            isInitialized: buf.readUInt8(0) === 1,
            isLocked: buf.readUInt8(1) === 1,
            allocatedSize: buf.readUInt32LE(4),
            owner: new web3_js_1.PublicKey(buf.subarray(8, 40))
        };
    }
    async readMemoryRange(ramPubkey, offset, length) {
        const info = await this.connection.getAccountInfo(ramPubkey);
        if (!info)
            return null;
        const start = exports.RAM_HEADER_SIZE + offset;
        const end = start + length;
        if (end > info.data.length)
            return null;
        return Buffer.from(info.data.subarray(start, end));
    }
}
exports.CustomRAMClient = CustomRAMClient;
