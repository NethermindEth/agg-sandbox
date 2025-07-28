import { ITransactionWriteResult } from "@maticnetwork/maticjs";
export declare class TransactionWriteResult implements ITransactionWriteResult {
    private promise;
    onTransactionHash: Function;
    onTransactionError: Function;
    onTransactionReceiptError: Function;
    onTransactionReceipt: Function;
    getReceipt: any;
    getTransactionHash: any;
    constructor(promise: any);
}
