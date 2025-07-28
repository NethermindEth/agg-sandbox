"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateBridgeMessageCommand = exports.validateClaimAssetCommand = exports.validateBridgeAssetCommand = exports.isValidNetworkId = exports.isValidAmount = exports.isValidTxHash = exports.isValidAddress = void 0;
function isValidAddress(address) {
    return /^0x[a-fA-F0-9]{40}$/.test(address);
}
exports.isValidAddress = isValidAddress;
function isValidTxHash(hash) {
    return /^0x[a-fA-F0-9]{64}$/.test(hash);
}
exports.isValidTxHash = isValidTxHash;
function isValidAmount(amount) {
    try {
        const num = parseFloat(amount);
        return num > 0 && !isNaN(num);
    }
    catch {
        return false;
    }
}
exports.isValidAmount = isValidAmount;
function isValidNetworkId(networkId, supportedNetworks) {
    return supportedNetworks.includes(networkId.toString());
}
exports.isValidNetworkId = isValidNetworkId;
function validateBridgeAssetCommand(command, supportedNetworks) {
    const errors = [];
    if (!isValidNetworkId(command.network, supportedNetworks)) {
        errors.push(`Unsupported source network: ${command.network}`);
    }
    if (!isValidNetworkId(command.destinationNetwork, supportedNetworks)) {
        errors.push(`Unsupported destination network: ${command.destinationNetwork}`);
    }
    if (command.network === command.destinationNetwork) {
        errors.push('Source and destination networks cannot be the same');
    }
    if (!isValidAmount(command.amount)) {
        errors.push('Invalid amount: must be a positive number');
    }
    if (!isValidAddress(command.tokenAddress)) {
        errors.push('Invalid token address format');
    }
    if (command.toAddress && !isValidAddress(command.toAddress)) {
        errors.push('Invalid recipient address format');
    }
    return {
        isValid: errors.length === 0,
        errors,
    };
}
exports.validateBridgeAssetCommand = validateBridgeAssetCommand;
function validateClaimAssetCommand(command, supportedNetworks) {
    const errors = [];
    if (!isValidNetworkId(command.network, supportedNetworks)) {
        errors.push(`Unsupported network: ${command.network}`);
    }
    if (!isValidNetworkId(command.sourceNetwork, supportedNetworks)) {
        errors.push(`Unsupported source network: ${command.sourceNetwork}`);
    }
    if (!isValidTxHash(command.txHash)) {
        errors.push('Invalid transaction hash format');
    }
    if (command.network === command.sourceNetwork) {
        errors.push('Network and source network cannot be the same');
    }
    return {
        isValid: errors.length === 0,
        errors,
    };
}
exports.validateClaimAssetCommand = validateClaimAssetCommand;
function validateBridgeMessageCommand(command, supportedNetworks) {
    const errors = [];
    if (!isValidNetworkId(command.network, supportedNetworks)) {
        errors.push(`Unsupported source network: ${command.network}`);
    }
    if (!isValidNetworkId(command.destinationNetwork, supportedNetworks)) {
        errors.push(`Unsupported destination network: ${command.destinationNetwork}`);
    }
    if (command.network === command.destinationNetwork) {
        errors.push('Source and destination networks cannot be the same');
    }
    if (!isValidAddress(command.target)) {
        errors.push('Invalid target contract address format');
    }
    if (!command.data.startsWith('0x')) {
        errors.push('Invalid call data format: must start with 0x');
    }
    if (command.amount && !isValidAmount(command.amount)) {
        errors.push('Invalid amount: must be a positive number');
    }
    if (command.fallbackAddress && !isValidAddress(command.fallbackAddress)) {
        errors.push('Invalid fallback address format');
    }
    return {
        isValid: errors.length === 0,
        errors,
    };
}
exports.validateBridgeMessageCommand = validateBridgeMessageCommand;
//# sourceMappingURL=validation.js.map