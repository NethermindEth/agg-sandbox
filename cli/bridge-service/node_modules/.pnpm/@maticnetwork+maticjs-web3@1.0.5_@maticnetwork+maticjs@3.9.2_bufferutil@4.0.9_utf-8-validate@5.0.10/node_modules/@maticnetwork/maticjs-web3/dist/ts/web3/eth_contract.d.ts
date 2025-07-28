import { BaseContract } from "@maticnetwork/maticjs";
import Contract from "web3/eth/contract";
import { EthMethod } from "./eth_method";
export declare class Web3Contract extends BaseContract {
    contract: Contract;
    constructor(address: string, contract: Contract, logger: any);
    method(methodName: string, ...args: any[]): EthMethod;
}
