// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import {BridgeL2SovereignChain} from "../src/BridgeL2SovereignChain.sol";
import {PolygonZkEVMTimelock} from "../src/PolygonZkEVMTimelock.sol";
import {PolygonZkEVM} from "../src/PolygonZkEVM.sol";
import {IBasePolygonZkEVMGlobalExitRoot} from "../src/interfaces/IBasePolygonZkEVMGlobalExitRoot.sol";
import {Script, console2} from "forge-std/Script.sol";
import {GlobalExitRootManagerL2SovereignChain} from "../src/GlobalExitRootManagerL2SovereignChain.sol";
import {AggERC20} from "../src/mocks/AggERC20.sol";
import {BridgeExtension} from "../src/BridgeExtension.sol";

contract DeployContractsL2 is Script {
    function run() external {
        // load your deployer private key from env
        uint256 deployerKey = vm.envUint("PRIVATE_KEY_1");
        address deployer = vm.addr(deployerKey);

        // start broadcasting transactions
        vm.startBroadcast(deployerKey);

        BridgeL2SovereignChain polygonZkEVMBridgeV2 = new BridgeL2SovereignChain();
        GlobalExitRootManagerL2SovereignChain globalExitRootManagerL2SovereignChain =
            new GlobalExitRootManagerL2SovereignChain(address(polygonZkEVMBridgeV2));

        uint256 minDelay = 3600;
        address[] memory proposers = new address[](1);
        proposers[0] = deployer;
        address[] memory executors = new address[](1);
        executors[0] = deployer;
        PolygonZkEVMTimelock polygonZkEVMTimelock =
            new PolygonZkEVMTimelock(minDelay, proposers, executors, deployer, PolygonZkEVM(address(0)));

        // Initialize the bridge
        polygonZkEVMBridgeV2.initialize(
            1, // _networkID - 1 for first L2
            address(0), // _gasTokenAddress - address(0) for ETH
            0, // _gasTokenNetwork
            IBasePolygonZkEVMGlobalExitRoot(address(globalExitRootManagerL2SovereignChain)), // _globalExitRootManager
            address(0), // _polygonRollupManager
            "", // _gasTokenMetadata - empty for ETH
            deployer, // _bridgeManager
            address(0), // _sovereignWETHAddress
            false // _sovereignWETHAddressIsNotMintable
        );

        // Initialize the global exit root manager
        globalExitRootManagerL2SovereignChain.initialize(deployer, address(0));

        AggERC20 aggERC20 = new AggERC20(deployer, deployer);

        BridgeExtension bridgeExtension = new BridgeExtension(address(polygonZkEVMBridgeV2));

        // stop broadcasting so logs don't count as on-chain txs
        vm.stopBroadcast();

        console2.log("PolygonZkEVMBridgeV2:   ", address(polygonZkEVMBridgeV2));
        console2.log("PolygonZkEVMTimelock:   ", address(polygonZkEVMTimelock));
        console2.log("GlobalExitRootManagerL2SovereignChain:   ", address(globalExitRootManagerL2SovereignChain));
        console2.log("AggERC20:              ", address(aggERC20));
        console2.log("BridgeExtension:       ", address(bridgeExtension));
    }
}
