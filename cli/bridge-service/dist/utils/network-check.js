"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.filterAvailableNetworks = exports.isNetworkAvailable = void 0;
const web3_1 = __importDefault(require("web3"));
async function isNetworkAvailable(rpcUrl) {
    try {
        const web3 = new web3_1.default(rpcUrl);
        await web3.eth.getBlockNumber();
        return true;
    }
    catch (error) {
        return false;
    }
}
exports.isNetworkAvailable = isNetworkAvailable;
async function filterAvailableNetworks(networks) {
    const availableNetworks = {};
    for (const [networkId, config] of Object.entries(networks)) {
        const isAvailable = await isNetworkAvailable(config.rpcUrl);
        if (isAvailable) {
            availableNetworks[networkId] = config;
            console.log(`✅ Network ${networkId} is available at ${config.rpcUrl}`);
        }
        else {
            console.log(`❌ Network ${networkId} is not available at ${config.rpcUrl} - skipping`);
        }
    }
    return availableNetworks;
}
exports.filterAvailableNetworks = filterAvailableNetworks;
//# sourceMappingURL=network-check.js.map