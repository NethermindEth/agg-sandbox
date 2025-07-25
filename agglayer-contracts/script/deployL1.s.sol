// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import {FflonkVerifier} from "../src/FflonkVerifier.sol";
import {PolygonZkEVM} from "../src/PolygonZkEVM.sol";
import {PolygonZkEVMBridgeV2} from "../src/PolygonZkEVMBridgeV2.sol";
import {PolygonZkEVMTimelock} from "../src/PolygonZkEVMTimelock.sol";
import {PolygonZkEVMGlobalExitRootV2} from "../src/PolygonZkEVMGlobalExitRootV2.sol";
import {PolygonRollupManager} from "../src/PolygonRollupManager.sol";
import {AggERC20} from "../src/mocks/AggERC20.sol";
import {Script, console2} from "forge-std/Script.sol";
import {IVerifierRollup} from "../src/interfaces/IVerifierRollup.sol";
import {IPolygonZkEVMBridge} from "../src/interfaces/IPolygonZkEVMBridge.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IPolygonZkEVMGlobalExitRootV2} from "../src/interfaces/IPolygonZkEVMGlobalExitRootV2.sol";
import {IBasePolygonZkEVMGlobalExitRoot} from "../src/interfaces/IBasePolygonZkEVMGlobalExitRoot.sol";
import {IPolygonRollupBase} from "../src/interfaces/IPolygonRollupBase.sol";
import {IPolygonRollupManager} from "../src/interfaces/IPolygonRollupManager.sol";
import {BridgeExtension} from "../src/BridgeExtension.sol";

contract DeployContractsL1 is Script {
    function run() external {
        // load your deployer private key from env
        uint256 deployerKey = vm.envUint("PRIVATE_KEY_1");
        address deployer = vm.addr(deployerKey);

        // start broadcasting transactions
        vm.startBroadcast(deployerKey);

        AggERC20 aggERC20 = new AggERC20(deployer, deployer, 1000000);

        // actual on-chain deploys
        FflonkVerifier fflonkVerifier = new FflonkVerifier();

        PolygonZkEVMBridgeV2 polygonZkEVMBridgeV2 = new PolygonZkEVMBridgeV2();

        PolygonZkEVMGlobalExitRootV2 polygonZkEVMGlobalExitRootV2 =
            new PolygonZkEVMGlobalExitRootV2(deployer, address(polygonZkEVMBridgeV2));

        PolygonZkEVM polygonZkEVM = new PolygonZkEVM(
            IPolygonZkEVMGlobalExitRootV2(address(polygonZkEVMGlobalExitRootV2)),
            IERC20(address(aggERC20)),
            IVerifierRollup(address(fflonkVerifier)),
            IPolygonZkEVMBridge(address(polygonZkEVMBridgeV2)),
            1,
            1
        );

        uint256 minDelay = 3600;
        address[] memory proposers = new address[](1);
        proposers[0] = deployer;
        address[] memory executors = new address[](1);
        executors[0] = deployer;
        PolygonZkEVMTimelock polygonZkEVMTimelock =
            new PolygonZkEVMTimelock(minDelay, proposers, executors, deployer, polygonZkEVM);
        PolygonRollupManager polygonRollupManager = new PolygonRollupManager(
            IPolygonZkEVMGlobalExitRootV2(address(polygonZkEVMGlobalExitRootV2)),
            IERC20(address(aggERC20)),
            IPolygonZkEVMBridge(address(polygonZkEVMBridgeV2))
        );

        BridgeExtension bridgeExtension = new BridgeExtension(address(polygonZkEVMBridgeV2));

        // Initialize the bridge
        polygonZkEVMBridgeV2.initialize(
            1, // _networkID - 1 for Ethereum
            address(0), // _gasTokenAddress - address(0) for ETH
            0, // _gasTokenNetwork
            IBasePolygonZkEVMGlobalExitRoot(address(polygonZkEVMGlobalExitRootV2)), // _globalExitRootManager
            address(polygonRollupManager), // _polygonRollupManager
            "" // _gasTokenMetadata - empty for ETH
        );

        // Initialize the RollupManager (MOCK VERSION: automatically grants roles to deployer)
        polygonRollupManager.initialize();

        // Register the PolygonZkEVM rollup in the RollupManager
        polygonRollupManager.addExistingRollup(
            IPolygonRollupBase(address(polygonZkEVM)), // rollupAddress
            address(fflonkVerifier), // verifier
            1, // forkID
            1101, // chainID (your L2 chain ID)
            0x0000000000000000000000000000000000000000000000000000000000000000, // initRoot (genesis state root)
            IPolygonRollupManager.VerifierType.StateTransition, // rollupVerifierType
            0x0000000000000000000000000000000000000000000000000000000000000000 // programVKey (empty for StateTransition)
        );

        // stop broadcasting so logs don't count as on-chain txs
        vm.stopBroadcast();

        // print out the addresses
        console2.log("FflonkVerifier:         ", address(fflonkVerifier));
        console2.log("PolygonZkEVM:           ", address(polygonZkEVM));
        console2.log("PolygonZkEVMBridgeV2:   ", address(polygonZkEVMBridgeV2));
        console2.log("PolygonZkEVMTimelock:   ", address(polygonZkEVMTimelock));
        console2.log("PolygonZkEVMGlobalExitRootV2: ", address(polygonZkEVMGlobalExitRootV2));
        console2.log("PolygonRollupManager:   ", address(polygonRollupManager));
        console2.log("AggERC20:              ", address(aggERC20));
        console2.log("BridgeExtension:       ", address(bridgeExtension));
        console2.log("Bridge initialized successfully!");
        console2.log("RollupManager initialized and rollup registered!");
        console2.log("Rollup registered with ID: 1");
    }
}
