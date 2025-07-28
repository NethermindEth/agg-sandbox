"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.BridgeClientWrapper = void 0;
const lxlyjs_1 = require("@maticnetwork/lxlyjs");
const maticjs_web3_1 = require("@maticnetwork/maticjs-web3");
const web3_1 = __importDefault(require("web3"));
const hdwallet_provider_1 = __importDefault(require("@truffle/hdwallet-provider"));
const network_check_1 = require("./utils/network-check");
const config_1 = require("./utils/config");
// Initialize Web3 plugin for lxly.js
(0, lxlyjs_1.use)(maticjs_web3_1.Web3ClientPlugin);
class BridgeClientWrapper {
    constructor(config) {
        this.providers = {};
        this.config = config;
        this.client = new lxlyjs_1.LxLyClient();
    }
    async initialize() {
        try {
            // Filter networks to only include those that are actually available
            console.log('Checking network availability...');
            const availableNetworks = await (0, network_check_1.filterAvailableNetworks)(this.config.networks);
            if (Object.keys(availableNetworks).length === 0) {
                throw new Error('No networks are available. Make sure the sandbox is running.');
            }
            // Update config to only include available networks
            this.config.networks = availableNetworks;
            // Create HDWalletProvider for each available network
            // For testing, we'll use a mock private key - in production this should come from secure storage
            const privateKey = process.env.PRIVATE_KEY || '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80';
            console.log('Using account:', this.config.defaultAccount);
            for (const [networkId, networkConfig] of Object.entries(availableNetworks)) {
                console.log(`Creating provider for network ${networkId}: ${networkConfig.rpcUrl}`);
                this.providers[networkId] = new hdwallet_provider_1.default([privateKey], networkConfig.rpcUrl);
            }
            // Initialize lxly.js client
            console.log('Setting up lxly.js provider configuration...');
            const providersConfig = {};
            for (const [networkId, networkConfig] of Object.entries(availableNetworks)) {
                console.log(`Network ${networkId} config:`, {
                    bridgeAddress: networkConfig.bridgeAddress,
                    bridgeExtensionAddress: networkConfig.bridgeExtensionAddress,
                    isEIP1559Supported: networkConfig.isEIP1559Supported,
                });
                providersConfig[networkId] = {
                    provider: this.providers[networkId],
                    configuration: {
                        bridgeAddress: networkConfig.bridgeAddress,
                        wrapperAddress: networkConfig.wrapperAddress,
                        bridgeExtensionAddress: networkConfig.bridgeExtensionAddress,
                        isEIP1559Supported: networkConfig.isEIP1559Supported,
                    },
                    defaultConfig: {
                        from: this.config.defaultAccount,
                    },
                };
            }
            console.log('Initializing lxly.js client...');
            await this.client.init({
                log: true,
                network: 'testnet',
                providers: providersConfig,
            });
            console.log('lxly.js client initialized successfully');
        }
        catch (error) {
            console.error('Error during client initialization:', error);
            throw error;
        }
    }
    async bridgeAsset(command) {
        try {
            console.log('Bridging asset:', command);
            // Get chain IDs for source and destination networks
            const destinationChainId = (0, config_1.getChainIdForNetwork)(this.config, command.destinationNetwork);
            if (!destinationChainId) {
                throw new Error(`Invalid destination network: ${command.destinationNetwork}`);
            }
            console.log(`Using network ${command.network} for source, destination chain ID: ${destinationChainId}`);
            const token = this.client.erc20(command.tokenAddress, command.network);
            const destinationAddress = command.toAddress || this.config.defaultAccount;
            // Check if approval is needed for ERC20 tokens
            if (command.tokenAddress !== '0x0000000000000000000000000000000000000000') {
                console.log('Checking token balance and approval...');
                try {
                    // Check user's token balance first
                    console.log('Getting balance...');
                    const balance = await token.getBalance(this.config.defaultAccount);
                    console.log(`User balance: ${balance}`);
                    if (balance === '0' || balance === 0) {
                        throw new Error(`No tokens found for account ${this.config.defaultAccount}. Balance: ${balance}`);
                    }
                    console.log('Checking approval...');
                    const isApprovalNeeded = await token.isApprovalNeeded();
                    console.log(`Is approval needed: ${isApprovalNeeded}`);
                    if (isApprovalNeeded) {
                        console.log('Approval needed, approving token...');
                        // Approve the bridge contract to spend tokens
                        const approveResult = await token.approve(command.amount);
                        const approveTxHash = await approveResult.getTransactionHash();
                        console.log('Approval transaction:', approveTxHash);
                        console.log('Waiting for approval confirmation...');
                        await approveResult.getReceipt();
                        console.log('Approval confirmed');
                    }
                }
                catch (error) {
                    console.error('Error during balance/approval check:', error);
                    throw error;
                }
            }
            // Perform the bridge operation using chain ID for destination
            console.log('Calling bridgeAsset with:', { amount: command.amount, destinationAddress, destinationChainId });
            const result = await token.bridgeAsset(command.amount, destinationAddress, destinationChainId);
            console.log('Bridge operation initiated, getting transaction hash...');
            const txHash = await result.getTransactionHash();
            console.log('Bridge transaction:', txHash);
            // Wait for transaction receipt with timeout
            console.log('Waiting for transaction confirmation...');
            const receipt = await Promise.race([
                result.getReceipt(),
                new Promise((_, reject) => setTimeout(() => reject(new Error('Transaction confirmation timeout')), 30000))
            ]);
            console.log('Transaction confirmed in block:', receipt.blockNumber);
            return {
                success: true,
                txHash,
                message: `Asset bridged successfully from network ${command.network} to chain ${destinationChainId}`,
            };
        }
        catch (error) {
            console.error('Bridge asset error:', error);
            return {
                success: false,
                error: error.message || 'Unknown error occurred during bridge operation',
            };
        }
    }
    async claimAsset(command) {
        try {
            console.log('Claiming asset:', command);
            // For now, we'll use ETH token address as default
            // TODO: Extract token address from original transaction
            const tokenAddress = '0x0000000000000000000000000000000000000000';
            const token = this.client.erc20(tokenAddress, command.network);
            const result = await token.claimAsset(command.txHash, command.sourceNetwork);
            const txHash = await result.getTransactionHash();
            console.log('Claim transaction:', txHash);
            const receipt = await result.getReceipt();
            console.log('Claim confirmed in block:', receipt.blockNumber);
            return {
                success: true,
                txHash,
                message: `Asset claimed successfully on network ${command.network}`,
            };
        }
        catch (error) {
            console.error('Claim asset error:', error);
            return {
                success: false,
                error: error.message || 'Unknown error occurred during claim operation',
            };
        }
    }
    async bridgeMessage(command) {
        try {
            console.log('Bridging message:', command);
            // Get chain ID for destination network
            const destinationChainId = (0, config_1.getChainIdForNetwork)(this.config, command.destinationNetwork);
            if (!destinationChainId) {
                throw new Error(`Invalid destination network: ${command.destinationNetwork}`);
            }
            console.log(`Using network ${command.network} for source, destination chain ID: ${destinationChainId}`);
            const bridgeExtension = this.client.bridgeExtensions[command.network];
            if (!bridgeExtension) {
                throw new Error(`Bridge extension not available for network ${command.network}`);
            }
            const token = command.amount ? '0x0000000000000000000000000000000000000000' : '0x0000000000000000000000000000000000000000';
            const amount = command.amount || '0';
            const fallbackAddress = command.fallbackAddress || this.config.defaultAccount;
            const result = await bridgeExtension.bridgeAndCall(token, amount, destinationChainId, command.target, fallbackAddress, command.data, true // forceUpdateGlobalExitRoot
            );
            const txHash = await result.getTransactionHash();
            console.log('Bridge and call transaction:', txHash);
            const receipt = await result.getReceipt();
            console.log('Bridge and call confirmed in block:', receipt.blockNumber);
            return {
                success: true,
                txHash,
                message: `Message bridged successfully from network ${command.network} to chain ${destinationChainId}`,
            };
        }
        catch (error) {
            console.error('Bridge message error:', error);
            return {
                success: false,
                error: error.message || 'Unknown error occurred during bridge message operation',
            };
        }
    }
    async getNetworkStatus(networkId) {
        try {
            const networkConfig = this.config.networks[networkId.toString()];
            if (!networkConfig) {
                return { connected: false };
            }
            // Create a direct Web3 instance to check connection
            const web3 = new web3_1.default(networkConfig.rpcUrl);
            const blockNumber = await web3.eth.getBlockNumber();
            return { connected: true, blockNumber: Number(blockNumber) };
        }
        catch (error) {
            return { connected: false };
        }
    }
}
exports.BridgeClientWrapper = BridgeClientWrapper;
//# sourceMappingURL=bridge-client.js.map