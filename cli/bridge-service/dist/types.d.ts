export interface NetworkConfig {
    rpcUrl: string;
    bridgeAddress: string;
    wrapperAddress?: string;
    bridgeExtensionAddress?: string;
    isEIP1559Supported: boolean;
    chainId: number;
}
export interface BridgeConfig {
    networks: Record<string, NetworkConfig>;
    defaultAccount: string;
    gasSettings: GasSettings;
}
export interface GasSettings {
    gasLimit: number;
    gasPrice?: string;
    maxFeePerGas?: string;
    maxPriorityFeePerGas?: string;
}
export interface BridgeAssetCommand {
    network: number;
    destinationNetwork: number;
    amount: string;
    tokenAddress: string;
    toAddress?: string;
    gasSettings?: GasSettings;
}
export interface ClaimAssetCommand {
    network: number;
    txHash: string;
    sourceNetwork: number;
    gasSettings?: GasSettings;
}
export interface BridgeMessageCommand {
    network: number;
    destinationNetwork: number;
    target: string;
    data: string;
    amount?: string;
    fallbackAddress?: string;
    gasSettings?: GasSettings;
}
export interface BridgeResponse {
    success: boolean;
    txHash?: string;
    error?: string;
    message?: string;
}
export interface ValidationResult {
    isValid: boolean;
    errors: string[];
}
//# sourceMappingURL=types.d.ts.map