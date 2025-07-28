import { config } from 'dotenv';
import { BridgeConfig, NetworkConfig } from '../types';

// Load environment variables
config();

export function loadBridgeConfig(): BridgeConfig {
  // Network configurations for sandbox environment
  // Network ID 0 = Mainnet (Chain ID 1), Network ID 1 = L2 AggLayer 1 (Chain ID 1101), Network ID 2 = L2 AggLayer 2 (Chain ID 137)
  const networks: Record<string, NetworkConfig> = {
    '0': {
      rpcUrl: process.env.L1_RPC_URL || 'http://localhost:8545',
      bridgeAddress: process.env.L1_BRIDGE_ADDRESS || '',
      wrapperAddress: process.env.L1_WRAPPER_ADDRESS,
      bridgeExtensionAddress: process.env.L1_BRIDGE_EXTENSION_ADDRESS,
      isEIP1559Supported: true,
      chainId: 1, // CHAIN_ID_MAINNET
    },
    '1': {
      rpcUrl: process.env.L2_RPC_URL || 'http://localhost:8546',
      bridgeAddress: process.env.L2_BRIDGE_ADDRESS || '',
      wrapperAddress: process.env.L2_WRAPPER_ADDRESS,
      bridgeExtensionAddress: process.env.L2_BRIDGE_EXTENSION_ADDRESS,
      isEIP1559Supported: false,
      chainId: 1101, // CHAIN_ID_AGGLAYER_1
    },
  };

  // Only add L3 if it's explicitly enabled and addresses are provided
  // This is only available when sandbox is started with --multi-l2 flag
  if (process.env.L3_RPC_URL && process.env.L3_BRIDGE_ADDRESS) {
    networks['2'] = {
      rpcUrl: process.env.L3_RPC_URL || 'http://localhost:8547',
      bridgeAddress: process.env.L3_BRIDGE_ADDRESS || '',
      wrapperAddress: process.env.L3_WRAPPER_ADDRESS,
      bridgeExtensionAddress: process.env.L3_BRIDGE_EXTENSION_ADDRESS,
      isEIP1559Supported: false,
      chainId: 137, // CHAIN_ID_AGGLAYER_2
    };
  }

  return {
    networks,
    defaultAccount: process.env.DEFAULT_ACCOUNT || '',
    gasSettings: {
      gasLimit: parseInt(process.env.GAS_LIMIT || '500000'),
      gasPrice: process.env.GAS_PRICE,
      maxFeePerGas: process.env.MAX_FEE_PER_GAS,
      maxPriorityFeePerGas: process.env.MAX_PRIORITY_FEE_PER_GAS,
    },
  };
}

export function validateConfig(config: BridgeConfig): string[] {
  const errors: string[] = [];

  if (!config.defaultAccount) {
    errors.push('DEFAULT_ACCOUNT environment variable is required');
  }

  for (const [networkId, networkConfig] of Object.entries(config.networks)) {
    if (!networkConfig.rpcUrl) {
      errors.push(`RPC URL is required for network ${networkId}`);
    }
    if (!networkConfig.bridgeAddress) {
      errors.push(`Bridge address is required for network ${networkId}`);
    }
    if (!networkConfig.chainId) {
      errors.push(`Chain ID is required for network ${networkId}`);
    }
  }

  return errors;
}

export function getChainIdForNetwork(config: BridgeConfig, networkId: number): number | undefined {
  const networkConfig = config.networks[networkId.toString()];
  return networkConfig?.chainId;
}