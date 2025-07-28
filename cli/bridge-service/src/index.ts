#!/usr/bin/env node

import { Command } from 'commander';
import { BridgeClientWrapper } from './bridge-client';
import { loadBridgeConfig, validateConfig } from './utils/config';
import { 
  validateBridgeAssetCommand, 
  validateClaimAssetCommand, 
  validateBridgeMessageCommand 
} from './utils/validation';
import { BridgeAssetCommand, ClaimAssetCommand, BridgeMessageCommand } from './types';

// Handle unhandled promise rejections
process.on('unhandledRejection', (reason, promise) => {
  console.error('❌ Unhandled Promise Rejection:', reason);
  process.exit(1);
});

process.on('uncaughtException', (error) => {
  console.error('❌ Uncaught Exception:', error.message);
  process.exit(1);
});

// Kill process after 2 minutes to allow for bridge operations
setTimeout(() => {
  console.error('❌ Process timeout - killing after 2 minutes');
  process.exit(1);
}, 120000);

const program = new Command();

program
  .name('aggsandbox-bridge')
  .description('Bridge service for aggsandbox CLI using lxly.js')
  .version('1.0.0');

async function initializeBridgeClient(): Promise<BridgeClientWrapper> {
  try {
    const config = loadBridgeConfig();
    console.log('Loaded config networks:', Object.keys(config.networks));
    
    const configErrors = validateConfig(config);
    
    if (configErrors.length > 0) {
      console.error('Configuration errors:');
      configErrors.forEach(error => console.error(`  - ${error}`));
      process.exit(1);
    }

    const client = new BridgeClientWrapper(config);
    console.log('Initializing bridge client...');
    await client.initialize();
    console.log('Bridge client initialized successfully');
    return client;
  } catch (error: any) {
    console.error('Failed to initialize bridge client:', error.message);
    throw error;
  }
}

program
  .command('bridge-asset')
  .description('Bridge assets between networks')
  .requiredOption('--network <number>', 'Source network ID', parseInt)
  .requiredOption('--destination-network <number>', 'Destination network ID', parseInt)
  .requiredOption('--amount <string>', 'Amount to bridge')
  .requiredOption('--token-address <string>', 'Token contract address')
  .option('--to-address <string>', 'Recipient address (defaults to sender)')
  .option('--gas-limit <number>', 'Gas limit override', parseInt)
  .option('--gas-price <string>', 'Gas price override')
  .action(async (options) => {
    try {
      const command: BridgeAssetCommand = {
        network: options.network,
        destinationNetwork: options.destinationNetwork,
        amount: options.amount,
        tokenAddress: options.tokenAddress,
        toAddress: options.toAddress,
        gasSettings: {
          gasLimit: options.gasLimit,
          gasPrice: options.gasPrice,
        },
      };

      const config = loadBridgeConfig();
      const supportedNetworks = Object.keys(config.networks);
      const validation = validateBridgeAssetCommand(command, supportedNetworks);

      if (!validation.isValid) {
        console.error('Validation errors:');
        validation.errors.forEach(error => console.error(`  - ${error}`));
        process.exit(1);
      }

      const client = await initializeBridgeClient();
      
      // Add timeout to prevent hanging
      const result = await Promise.race([
        client.bridgeAsset(command),
        new Promise((_, reject) => 
          setTimeout(() => reject(new Error('Bridge operation timeout after 90 seconds')), 90000)
        )
      ]) as any;

      if (result.success) {
        console.log(`✅ ${result.message}`);
        console.log(`Transaction hash: ${result.txHash}`);
        process.exit(0); // Exit cleanly on success
      } else {
        console.error(`❌ Bridge failed: ${result.error}`);
        process.exit(1);
      }
    } catch (error: any) {
      console.error(`❌ Unexpected error: ${error.message}`);
      process.exit(1);
    }
  });

program
  .command('claim-asset')
  .description('Claim bridged assets')
  .requiredOption('--network <number>', 'Network to claim on', parseInt)
  .requiredOption('--tx-hash <string>', 'Original bridge transaction hash')
  .requiredOption('--source-network <number>', 'Source network ID', parseInt)
  .option('--gas-limit <number>', 'Gas limit override', parseInt)
  .option('--gas-price <string>', 'Gas price override')
  .action(async (options) => {
    try {
      const command: ClaimAssetCommand = {
        network: options.network,
        txHash: options.txHash,
        sourceNetwork: options.sourceNetwork,
        gasSettings: {
          gasLimit: options.gasLimit,
          gasPrice: options.gasPrice,
        },
      };

      const config = loadBridgeConfig();
      const supportedNetworks = Object.keys(config.networks);
      const validation = validateClaimAssetCommand(command, supportedNetworks);

      if (!validation.isValid) {
        console.error('Validation errors:');
        validation.errors.forEach(error => console.error(`  - ${error}`));
        process.exit(1);
      }

      const client = await initializeBridgeClient();
      const result = await client.claimAsset(command);

      if (result.success) {
        console.log(`✅ ${result.message}`);
        console.log(`Transaction hash: ${result.txHash}`);
      } else {
        console.error(`❌ Claim failed: ${result.error}`);
        process.exit(1);
      }
    } catch (error: any) {
      console.error(`❌ Unexpected error: ${error.message}`);
      process.exit(1);
    }
  });

program
  .command('bridge-message')
  .description('Bridge message with contract call')
  .requiredOption('--network <number>', 'Source network ID', parseInt)
  .requiredOption('--destination-network <number>', 'Destination network ID', parseInt)
  .requiredOption('--target <string>', 'Target contract address')
  .requiredOption('--data <string>', 'Call data (hex encoded)')
  .option('--amount <string>', 'Amount of ETH to send')
  .option('--fallback-address <string>', 'Fallback address if call fails')
  .option('--gas-limit <number>', 'Gas limit override', parseInt)
  .option('--gas-price <string>', 'Gas price override')
  .action(async (options) => {
    try {
      const command: BridgeMessageCommand = {
        network: options.network,
        destinationNetwork: options.destinationNetwork,
        target: options.target,
        data: options.data,
        amount: options.amount,
        fallbackAddress: options.fallbackAddress,
        gasSettings: {
          gasLimit: options.gasLimit,
          gasPrice: options.gasPrice,
        },
      };

      const config = loadBridgeConfig();
      const supportedNetworks = Object.keys(config.networks);
      const validation = validateBridgeMessageCommand(command, supportedNetworks);

      if (!validation.isValid) {
        console.error('Validation errors:');
        validation.errors.forEach(error => console.error(`  - ${error}`));
        process.exit(1);
      }

      const client = await initializeBridgeClient();
      const result = await client.bridgeMessage(command);

      if (result.success) {
        console.log(`✅ ${result.message}`);
        console.log(`Transaction hash: ${result.txHash}`);
      } else {
        console.error(`❌ Bridge message failed: ${result.error}`);
        process.exit(1);
      }
    } catch (error: any) {
      console.error(`❌ Unexpected error: ${error.message}`);
      process.exit(1);
    }
  });

program
  .command('status')
  .description('Check network connection status')
  .action(async () => {
    try {
      const client = await initializeBridgeClient();
      const config = loadBridgeConfig();

      console.log('Network Status:');
      for (const networkId of Object.keys(config.networks)) {
        try {
          const status = await client.getNetworkStatus(parseInt(networkId));
          const icon = status.connected ? '✅' : '❌';
          const blockInfo = status.blockNumber ? ` (block: ${status.blockNumber})` : '';
          console.log(`  Network ${networkId}: ${icon} ${status.connected ? 'Connected' : 'Disconnected'}${blockInfo}`);
        } catch (error: any) {
          console.log(`  Network ${networkId}: ❌ Error - ${error.message}`);
        }
      }
    } catch (error: any) {
      console.error(`❌ Status check failed: ${error.message}`);
      process.exit(1);
    }
  });

// Parse command line arguments
program.parse();