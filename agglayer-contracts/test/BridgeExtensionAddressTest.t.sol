// SPDX-License-Identifier: AGPL-3.0
pragma solidity ^0.8.22;

import "forge-std/Test.sol";
import "../src/BridgeExtension.sol";
import "../src/PolygonZkEVMBridgeV2.sol";
import "../src/BridgeL2SovereignChain.sol";
import "../src/interfaces/IBasePolygonZkEVMGlobalExitRoot.sol";

contract MockGlobalExitRoot is IBasePolygonZkEVMGlobalExitRoot {
    mapping(bytes32 => uint256) public globalExitRootMap;

    function updateExitRoot(bytes32 newRoot) external {
        globalExitRootMap[newRoot] = block.timestamp;
    }

    function getLastGlobalExitRoot() external pure returns (bytes32) {
        return bytes32(uint256(1));
    }
}

contract BridgeExtensionAddressTest is Test {
    function testBridgeExtensionDifferentAddresses() public {
        MockGlobalExitRoot globalExitRoot = new MockGlobalExitRoot();

        // Deploy L1-style bridge (PolygonZkEVMBridgeV2)
        PolygonZkEVMBridgeV2 bridgeL1 = new PolygonZkEVMBridgeV2();
        bridgeL1.initialize(
            1, // networkID
            address(0), // gasTokenAddress
            0, // gasTokenNetwork
            globalExitRoot,
            address(0), // polygonRollupManager
            "" // gasTokenMetadata
        );

        // Deploy L2-style bridge (BridgeL2SovereignChain)
        BridgeL2SovereignChain bridgeL2 = new BridgeL2SovereignChain();
        bridgeL2.initialize(
            1101, // networkID
            address(0), // gasTokenAddress
            0, // gasTokenNetwork
            globalExitRoot,
            address(0), // polygonRollupManager
            "", // gasTokenMetadata
            address(this), // bridgeManager
            address(0), // sovereignWETHAddress
            false // sovereignWETHAddressIsNotMintable
        );

        // Deploy BridgeExtensions using CREATE2 with same salt
        bytes32 salt = keccak256("BRIDGE_EXTENSION_SALT_V1");

        BridgeExtension bridgeExtensionL1 = new BridgeExtension{salt: salt}(address(bridgeL1));
        BridgeExtension bridgeExtensionL2 = new BridgeExtension{salt: salt}(address(bridgeL2));

        console2.log("BridgeExtension L1 address:", address(bridgeExtensionL1));
        console2.log("BridgeExtension L2 address:", address(bridgeExtensionL2));
        console2.log("Bridge L1 address:", address(bridgeL1));
        console2.log("Bridge L2 address:", address(bridgeL2));

        // They will have different addresses because constructor params differ
        bool sameAddress = address(bridgeExtensionL1) == address(bridgeExtensionL2);
        console2.log("Same address:", sameAddress);

        // This confirms that CREATE2 with different constructor params produces different addresses
        assertFalse(
            sameAddress, "BridgeExtension addresses should be different due to different bridge constructor params"
        );

        // This is the fundamental issue - L1 uses PolygonZkEVMBridgeV2, L2/L3 use BridgeL2SovereignChain
        // Both inherit from PolygonZkEVMBridgeV2 but have different addresses
        console2.log("The issue: L1 and L2 bridge types are compatible but have different addresses");
        console2.log("L1 bridge type: PolygonZkEVMBridgeV2");
        console2.log("L2 bridge type: BridgeL2SovereignChain (inherits from PolygonZkEVMBridgeV2)");
    }

    function testCalculateCreate2Address() public {
        // Test CREATE2 address calculation manually
        bytes32 salt = keccak256("BRIDGE_EXTENSION_SALT_V1");

        address mockBridge1 = address(0x1111111111111111111111111111111111111111);
        address mockBridge2 = address(0x2222222222222222222222222222222222222222);

        // Calculate CREATE2 addresses
        bytes memory bytecode1 = abi.encodePacked(type(BridgeExtension).creationCode, abi.encode(mockBridge1));

        bytes memory bytecode2 = abi.encodePacked(type(BridgeExtension).creationCode, abi.encode(mockBridge2));

        bytes32 hash1 = keccak256(abi.encodePacked(bytes1(0xff), address(this), salt, keccak256(bytecode1)));

        bytes32 hash2 = keccak256(abi.encodePacked(bytes1(0xff), address(this), salt, keccak256(bytecode2)));

        address addr1 = address(uint160(uint256(hash1)));
        address addr2 = address(uint160(uint256(hash2)));

        console2.log("Calculated address with bridge1:", addr1);
        console2.log("Calculated address with bridge2:", addr2);
        console2.log("Same calculated address:", addr1 == addr2);

        // The addresses will be different because constructor params are different
        assertFalse(addr1 == addr2, "CREATE2 addresses should be different with different constructor params");
    }
}
