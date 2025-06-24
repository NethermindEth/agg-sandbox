// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import {PolygonZkEVMBridgeV2} from "../src/PolygonZkEVMBridgeV2.sol";
import {PolygonZkEVMTimelock} from "../src/PolygonZkEVMTimelock.sol";
import {PolygonZkEVM} from "../src/PolygonZkEVM.sol";
import {Script, console2} from "forge-std/Script.sol";

contract DeployContractsL1 is Script {
    function run() external {
        // load your deployer private key from env
        uint256 deployerKey = vm.envUint("PRIVATE_KEY_1");
        address deployer = vm.addr(deployerKey);

        // start broadcasting transactions
        vm.startBroadcast(deployerKey);

        PolygonZkEVMBridgeV2 polygonZkEVMBridgeV2 = new PolygonZkEVMBridgeV2();

        uint256 minDelay = 3600;
        address[] memory proposers = new address[](1);
        proposers[0] = deployer;
        address[] memory executors = new address[](1);
        executors[0] = deployer;
        PolygonZkEVMTimelock polygonZkEVMTimelock =
            new PolygonZkEVMTimelock(minDelay, proposers, executors, deployer, PolygonZkEVM(address(0)));

        // stop broadcasting so logs don't count as on-chain txs
        vm.stopBroadcast();

        console2.log("PolygonZkEVMBridgeV2:   ", address(polygonZkEVMBridgeV2));
        console2.log("PolygonZkEVMTimelock:   ", address(polygonZkEVMTimelock));
    }
}
