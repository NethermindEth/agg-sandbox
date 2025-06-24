// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import {FflonkVerifier} from "../src/FflonkVerifier.sol";
import {PolygonZkEVM} from "../src/PolygonZkEVM.sol";
import {PolygonZkEVMBridgeV2} from "../src/PolygonZkEVMBridgeV2.sol";
import {PolygonZkEVMTimelock} from "../src/PolygonZkEVMTimelock.sol";
import {PolygonZkEVMGlobalExitRootV2} from "../src/PolygonZkEVMGlobalExitRootV2.sol";
import {PolygonRollupManager} from "../src/PolygonRollupManager.sol";
import {Matic} from "../src/mocks/Matic.sol";
import {Script, console2} from "forge-std/Script.sol";
import {IVerifierRollup} from "../src/interfaces/IVerifierRollup.sol";
import {IPolygonZkEVMBridge} from "../src/interfaces/IPolygonZkEVMBridge.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IPolygonZkEVMGlobalExitRootV2} from "../src/interfaces/IPolygonZkEVMGlobalExitRootV2.sol";

contract DeployContractsL1 is Script {
    function run() external {
        // load your deployer private key from env
        uint256 deployerKey = vm.envUint("PRIVATE_KEY_1");
        address deployer = vm.addr(deployerKey);

        // start broadcasting transactions
        vm.startBroadcast(deployerKey);

        Matic matic = new Matic(deployer, deployer);

        // actual on-chain deploys
        FflonkVerifier fflonkVerifier = new FflonkVerifier();

        PolygonZkEVMBridgeV2 polygonZkEVMBridgeV2 = new PolygonZkEVMBridgeV2();

        PolygonZkEVMGlobalExitRootV2 polygonZkEVMGlobalExitRootV2 =
            new PolygonZkEVMGlobalExitRootV2(deployer, address(polygonZkEVMBridgeV2));

        PolygonZkEVM polygonZkEVM = new PolygonZkEVM(
            IPolygonZkEVMGlobalExitRootV2(address(polygonZkEVMGlobalExitRootV2)),
            IERC20(address(matic)),
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
            IERC20(address(matic)),
            IPolygonZkEVMBridge(address(polygonZkEVMBridgeV2))
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
    }
}
