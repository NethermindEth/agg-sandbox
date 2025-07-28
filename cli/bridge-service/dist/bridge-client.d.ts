import { BridgeConfig, BridgeResponse, BridgeAssetCommand, ClaimAssetCommand, BridgeMessageCommand } from './types';
export declare class BridgeClientWrapper {
    private client;
    private config;
    private providers;
    constructor(config: BridgeConfig);
    initialize(): Promise<void>;
    bridgeAsset(command: BridgeAssetCommand): Promise<BridgeResponse>;
    claimAsset(command: ClaimAssetCommand): Promise<BridgeResponse>;
    bridgeMessage(command: BridgeMessageCommand): Promise<BridgeResponse>;
    getNetworkStatus(networkId: number): Promise<{
        connected: boolean;
        blockNumber?: number;
    }>;
}
//# sourceMappingURL=bridge-client.d.ts.map