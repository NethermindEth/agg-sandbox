/// <reference types="node" />
import { BaseBigNumber } from "@maticnetwork/maticjs";
export declare class MaticBigNumber extends BaseBigNumber {
    private bn_;
    constructor(value: any);
    static isBN(value: any): boolean;
    toString(base?: any): string;
    toNumber(): number;
    toBuffer(base?: any): Buffer;
    add(value: BaseBigNumber): MaticBigNumber;
    sub(value: BaseBigNumber): MaticBigNumber;
    mul(value: BaseBigNumber): MaticBigNumber;
    div(value: BaseBigNumber): MaticBigNumber;
    lte(value: BaseBigNumber): boolean;
    lt(value: BaseBigNumber): boolean;
    gte(value: BaseBigNumber): boolean;
    gt(value: BaseBigNumber): boolean;
    eq(value: BaseBigNumber): boolean;
}
