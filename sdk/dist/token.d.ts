import { PublicKey, TransactionInstruction, Connection } from '@solana/web3.js';
export declare const TOKEN_PROGRAM_ID: PublicKey;
export declare const MINT_SIZE = 80;
export declare const ACCOUNT_SIZE = 80;
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
    state: number;
}
export declare class CustomTokenClient {
    readonly connection: Connection;
    readonly programId: PublicKey;
    constructor(connection: Connection, programId?: PublicKey);
    createInitMintInstruction(mintPubkey: PublicKey, payerPubkey: PublicKey, decimals: number, feeBasisPoints: number): TransactionInstruction;
    createInitAccountInstruction(accountPubkey: PublicKey, mintPubkey: PublicKey, ownerPubkey: PublicKey): TransactionInstruction;
    createMintToInstruction(mintPubkey: PublicKey, destinationPubkey: PublicKey, authorityPubkey: PublicKey, amount: bigint): TransactionInstruction;
    createTransferInstruction(sourcePubkey: PublicKey, destinationPubkey: PublicKey, mintPubkey: PublicKey, authorityPubkey: PublicKey, amount: bigint): TransactionInstruction;
    createBurnInstruction(accountPubkey: PublicKey, mintPubkey: PublicKey, authorityPubkey: PublicKey, amount: bigint): TransactionInstruction;
    createFreezeInstruction(accountPubkey: PublicKey, mintPubkey: PublicKey, authorityPubkey: PublicKey): TransactionInstruction;
    createThawInstruction(accountPubkey: PublicKey, mintPubkey: PublicKey, authorityPubkey: PublicKey): TransactionInstruction;
    fetchMint(mintPubkey: PublicKey): Promise<MintState | null>;
    fetchAccount(accountPubkey: PublicKey): Promise<TokenAccountState | null>;
}
