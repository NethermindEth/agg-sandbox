import Web3 from 'web3';

export async function isNetworkAvailable(rpcUrl: string): Promise<boolean> {
  try {
    const web3 = new Web3(rpcUrl);
    await web3.eth.getBlockNumber();
    return true;
  } catch (error) {
    return false;
  }
}

export async function filterAvailableNetworks(networks: Record<string, any>): Promise<Record<string, any>> {
  const availableNetworks: Record<string, any> = {};
  
  for (const [networkId, config] of Object.entries(networks)) {
    const isAvailable = await isNetworkAvailable(config.rpcUrl);
    if (isAvailable) {
      availableNetworks[networkId] = config;
      console.log(`✅ Network ${networkId} is available at ${config.rpcUrl}`);
    } else {
      console.log(`❌ Network ${networkId} is not available at ${config.rpcUrl} - skipping`);
    }
  }
  
  return availableNetworks;
}