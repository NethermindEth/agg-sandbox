import { BridgeAssetCommand, ClaimAssetCommand, BridgeMessageCommand, ValidationResult } from '../types';
export declare function isValidAddress(address: string): boolean;
export declare function isValidTxHash(hash: string): boolean;
export declare function isValidAmount(amount: string): boolean;
export declare function isValidNetworkId(networkId: number, supportedNetworks: string[]): boolean;
export declare function validateBridgeAssetCommand(command: BridgeAssetCommand, supportedNetworks: string[]): ValidationResult;
export declare function validateClaimAssetCommand(command: ClaimAssetCommand, supportedNetworks: string[]): ValidationResult;
export declare function validateBridgeMessageCommand(command: BridgeMessageCommand, supportedNetworks: string[]): ValidationResult;
//# sourceMappingURL=validation.d.ts.map