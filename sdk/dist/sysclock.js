"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.CustomSysclockClient = exports.CLOCK_STATE_SIZE = exports.SYSCLOCK_PROGRAM_ID = void 0;
const web3_js_1 = require("@solana/web3.js");
exports.SYSCLOCK_PROGRAM_ID = new web3_js_1.PublicKey('8VB675MfjjiaizM2VDBRQfk8GMKCiZVtMELqGdstgMTK');
exports.CLOCK_STATE_SIZE = 72;
class CustomSysclockClient {
    connection;
    programId;
    constructor(connection, programId = exports.SYSCLOCK_PROGRAM_ID) {
        this.connection = connection;
        this.programId = programId;
    }
    createInitClockInstruction(clockPubkey, authorityPubkey) {
        return new web3_js_1.TransactionInstruction({
            programId: this.programId,
            keys: [
                { pubkey: clockPubkey, isSigner: false, isWritable: true },
                { pubkey: authorityPubkey, isSigner: true, isWritable: false }
            ],
            data: Buffer.from([0]) // Tag 0 = InitializeClock
        });
    }
    createTickInstruction(clockPubkey, authorityPubkey, slot, unixTimestampMs, advanceEpoch) {
        const data = Buffer.alloc(18);
        data.writeUInt8(1, 0); // Tag 1 = Tick
        data.writeBigUInt64LE(slot, 1);
        data.writeBigInt64LE(unixTimestampMs, 9);
        data.writeUInt8(advanceEpoch ? 1 : 0, 17);
        return new web3_js_1.TransactionInstruction({
            programId: this.programId,
            keys: [
                { pubkey: clockPubkey, isSigner: false, isWritable: true },
                { pubkey: authorityPubkey, isSigner: true, isWritable: false }
            ],
            data
        });
    }
    createReadClockInstruction(clockPubkey) {
        return new web3_js_1.TransactionInstruction({
            programId: this.programId,
            keys: [
                { pubkey: clockPubkey, isSigner: false, isWritable: false }
            ],
            data: Buffer.from([2]) // Tag 2 = ReadClock
        });
    }
    async fetchClock(clockPubkey) {
        const info = await this.connection.getAccountInfo(clockPubkey);
        if (!info || info.data.length < exports.CLOCK_STATE_SIZE)
            return null;
        const buf = Buffer.from(info.data);
        return {
            isInitialized: buf.readUInt8(0) === 1,
            isFrozen: buf.readUInt8(1) === 1,
            slot: buf.readBigUInt64LE(8),
            epoch: buf.readBigUInt64LE(16),
            unixTimestampMs: buf.readBigInt64LE(24),
            tickSequence: buf.readBigUInt64LE(32),
            authority: new web3_js_1.PublicKey(buf.subarray(40, 72))
        };
    }
}
exports.CustomSysclockClient = CustomSysclockClient;
