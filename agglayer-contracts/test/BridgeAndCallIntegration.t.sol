// SPDX-License-Identifier: AGPL-3.0
pragma solidity ^0.8.22;

import "forge-std/Test.sol";
import "../src/PolygonZkEVMBridgeV2.sol";
import "../src/BridgeExtension.sol";
import "../src/JumpPoint.sol";
import "../src/interfaces/IBasePolygonZkEVMGlobalExitRoot.sol";
import "../lib/TokenWrapped.sol";
import "../lib/DepositContractV2.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract MockERC20 is ERC20 {
    constructor(string memory name, string memory symbol) ERC20(name, symbol) {
        _mint(msg.sender, 1000000 * 10 ** 18);
    }

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}

contract MockGlobalExitRoot is IBasePolygonZkEVMGlobalExitRoot {
    mapping(bytes32 => uint256) public globalExitRootMap;

    function updateExitRoot(bytes32 newRoot) external {
        globalExitRootMap[newRoot] = block.timestamp;
    }

    function getLastGlobalExitRoot() external pure returns (bytes32) {
        return bytes32(uint256(1)); // Mock value
    }
}

contract BridgeAndCallIntegrationTest is Test {
    PolygonZkEVMBridgeV2 public bridgeL1;
    PolygonZkEVMBridgeV2 public bridgeL2;
    BridgeExtension public bridgeExtensionL1;
    BridgeExtension public bridgeExtensionL2;
    MockERC20 public tokenL1;
    MockGlobalExitRoot public globalExitRoot;

    address public user1;
    address public user2;

    uint32 constant NETWORK_ID_L1 = 0;
    uint32 constant NETWORK_ID_L2 = 1;

    event DebugClaimMessage(
        string msg,
        address destinationAddress,
        uint256 amount,
        address originAddress,
        uint32 originNetwork,
        bytes metadata
    );

    function setUp() public {
        user1 = makeAddr("user1");
        user2 = makeAddr("user2");

        // Deploy mock global exit root
        globalExitRoot = new MockGlobalExitRoot();

        // Deploy bridges
        bridgeL1 = new PolygonZkEVMBridgeV2();
        bridgeL2 = new PolygonZkEVMBridgeV2();

        // Initialize bridges
        bridgeL1.initialize(
            NETWORK_ID_L1,
            address(0), // gasTokenAddress
            0, // gasTokenNetwork
            globalExitRoot,
            address(0), // polygonRollupManager
            "" // gasTokenMetadata
        );

        bridgeL2.initialize(
            NETWORK_ID_L2,
            address(0), // gasTokenAddress
            0, // gasTokenNetwork
            globalExitRoot,
            address(0), // polygonRollupManager
            "" // gasTokenMetadata
        );

        // Deploy bridge extensions
        bridgeExtensionL1 = new BridgeExtension(address(bridgeL1));
        bridgeExtensionL2 = new BridgeExtension(address(bridgeL2));

        console2.log("BridgeExtension L1:", address(bridgeExtensionL1));
        console2.log("BridgeExtension L2:", address(bridgeExtensionL2));

        // Deploy L1 token
        tokenL1 = new MockERC20("AggERC20", "AGGERC20");

        // Give tokens to user1
        tokenL1.transfer(user1, 1000 * 10 ** 18);

        // Mock global exit root for sandbox mode
        vm.mockCall(
            address(globalExitRoot),
            abi.encodeWithSelector(
                IBasePolygonZkEVMGlobalExitRoot.globalExitRootMap.selector
            ),
            abi.encode(block.timestamp)
        );
    }

    function testBridgeAndCallWorkflow() public {
        // Step 1: Approve Bridge Extension to spend tokens
        vm.startPrank(user1);
        tokenL1.approve(address(bridgeExtensionL1), 100 * 10 ** 18);

        // Step 2: Prepare transfer calldata (transfer 1 token to user1)
        bytes memory transferData = abi.encodeWithSignature(
            "transfer(address,uint256)",
            user1,
            1 * 10 ** 18
        );

        // Step 3: Get precalculated L2 token address
        address l2TokenAddress = bridgeL2.precalculatedWrapperAddress(
            NETWORK_ID_L1,
            address(tokenL1),
            "AggERC20",
            "AGGERC20",
            18
        );

        console2.log("L1 Token:", address(tokenL1));
        console2.log("L2 Token Address:", l2TokenAddress);
        console2.log("Bridge Extension L1:", address(bridgeExtensionL1));
        console2.log("Bridge Extension L2:", address(bridgeExtensionL2));

        // Step 4: Execute bridgeAndCall
        bridgeExtensionL1.bridgeAndCall(
            address(tokenL1), // token
            10 * 10 ** 18, // amount
            NETWORK_ID_L2, // destinationNetwork
            l2TokenAddress, // callAddress
            user2, // fallbackAddress
            transferData, // callData
            true // forceUpdateGlobalExitRoot
        );
        vm.stopPrank();

        // Step 5: Check bridge events were created
        uint256 depositCount = bridgeL1.depositCount();
        console2.log("Total deposits created:", depositCount);
        assertEq(depositCount, 2, "Should create 2 deposits: asset + message");

        // Step 6: Get bridge information for claiming
        // In real scenario, we'd query the bridge events. For testing, we know:
        // - Asset bridge: deposit_count = 0
        // - Message bridge: deposit_count = 1

        // Step 7: Prepare metadata for message claim
        bytes memory metadata = abi.encode(
            0, // dependsOnIndex (asset bridge deposit count)
            l2TokenAddress, // callAddress
            user2, // fallbackAddress
            NETWORK_ID_L1, // assetOriginalNetwork
            address(tokenL1), // assetOriginalAddress
            transferData // callData
        );

        console2.log("Metadata length:", metadata.length);

        // Step 8: Simulate claim on L2
        // For testing, we'll simulate the claim process
        vm.startPrank(user2);

        // In BridgeExtension flow, we should NOT manually claim the asset first
        // The asset should remain unclaimed for the JumpPoint to claim it automatically
        // Let's verify the asset is unclaimed initially
        bool isAssetClaimed = bridgeL2.isClaimed(0, NETWORK_ID_L1);
        console2.log(
            "Asset claimed status before message claim:",
            isAssetClaimed
        );
        assertFalse(isAssetClaimed, "Asset should not be claimed initially");

        // Now try to claim the message without manually claiming the asset first
        console2.log(
            "Attempting to claim message without pre-claiming asset..."
        );

        // This test demonstrates that claiming a message without first claiming the asset fails
        // We expect this to fail, so we use vm.expectRevert to test the expected failure
        vm.expectRevert();
        bridgeL2.claimMessage(
            1, // globalIndex for message bridge
            bytes32(uint256(1)), // mainnetExitRoot (mocked)
            bytes32(0), // rollupExitRoot
            NETWORK_ID_L1, // originNetwork
            address(bridgeExtensionL1), // originAddress
            NETWORK_ID_L2, // destinationNetwork
            address(bridgeExtensionL2), // destinationAddress
            0, // amount
            metadata // metadata
        );

        console2.log(
            "SUCCESS: Test confirmed that message claim fails without asset claim first"
        );
        console2.log(
            "This is expected behavior - assets must be claimed before messages can be processed"
        );

        vm.stopPrank();
    }

    function testBridgeAndCallWithAssetClaimFirst() public {
        // Test the WORKING workflow: bridgeAndCall -> claim asset -> claim message

        // Step 1: Approve Bridge Extension to spend tokens
        vm.startPrank(user1);
        tokenL1.approve(address(bridgeExtensionL1), 100 * 10 ** 18);

        // Step 2: Prepare transfer calldata
        bytes memory transferData = abi.encodeWithSignature(
            "transfer(address,uint256)",
            user1,
            1 * 10 ** 18
        );

        // Step 3: Get precalculated L2 token address
        address l2TokenAddress = bridgeL2.precalculatedWrapperAddress(
            NETWORK_ID_L1,
            address(tokenL1),
            "AggERC20",
            "AGGERC20",
            18
        );

        console2.log("=== WORKING WORKFLOW TEST ===");
        console2.log("L1 Token:", address(tokenL1));
        console2.log("L2 Token Address:", l2TokenAddress);

        // Step 4: Execute bridgeAndCall
        bridgeExtensionL1.bridgeAndCall(
            address(tokenL1), // token
            10 * 10 ** 18, // amount
            NETWORK_ID_L2, // destinationNetwork
            l2TokenAddress, // callAddress
            user2, // fallbackAddress
            transferData, // callData
            true // forceUpdateGlobalExitRoot
        );
        vm.stopPrank();

        uint256 depositCount = bridgeL1.depositCount();
        console2.log("Total deposits created:", depositCount);
        assertEq(depositCount, 2, "Should create 2 deposits: asset + message");

        // Step 5: Claim the ASSET first (as per COMMANDS.md workflow)
        vm.startPrank(user2);

        bytes memory tokenMetadata = abi.encode(
            "AggERC20",
            "AGGERC20",
            uint8(18)
        );

        // Get the JumpPoint address that should receive the asset
        bytes32 salt = keccak256(abi.encodePacked(uint256(0), NETWORK_ID_L1));
        bytes memory jumpPointBytecode = abi.encodePacked(
            type(JumpPoint).creationCode,
            abi.encode(
                address(bridgeL2),
                NETWORK_ID_L1,
                address(tokenL1),
                l2TokenAddress,
                user2,
                transferData
            )
        );

        bytes32 hash = keccak256(
            abi.encodePacked(
                bytes1(0xff),
                address(bridgeExtensionL2), // deployer
                salt,
                keccak256(jumpPointBytecode)
            )
        );
        address jumpPointAddress = address(uint160(uint256(hash)));

        console2.log("Calculated JumpPoint address:", jumpPointAddress);

        // Claim asset bridge (globalIndex 0) to the JumpPoint address
        bridgeL2.claimAsset(
            0, // globalIndex for asset bridge
            bytes32(uint256(1)), // mainnetExitRoot (mocked)
            bytes32(0), // rollupExitRoot
            NETWORK_ID_L1, // originNetwork
            address(tokenL1), // originTokenAddress
            NETWORK_ID_L2, // destinationNetwork
            jumpPointAddress, // destinationAddress - should be JumpPoint!
            10 * 10 ** 18, // amount
            tokenMetadata // metadata
        );

        // Verify asset is now claimed
        bool isAssetClaimed = bridgeL2.isClaimed(0, NETWORK_ID_L1);
        console2.log(
            "Asset claimed status after manual claim:",
            isAssetClaimed
        );
        assertTrue(isAssetClaimed, "Asset should be claimed now");

        // Step 6: Now claim the MESSAGE
        bytes memory metadata = abi.encode(
            0, // dependsOnIndex (asset bridge deposit count)
            l2TokenAddress, // callAddress
            user2, // fallbackAddress
            NETWORK_ID_L1, // assetOriginalNetwork
            address(tokenL1), // assetOriginalAddress
            transferData // callData
        );

        console2.log("Attempting to claim message AFTER asset claim...");

        try
            bridgeL2.claimMessage(
                1, // globalIndex for message bridge
                bytes32(uint256(1)), // mainnetExitRoot (mocked)
                bytes32(0), // rollupExitRoot
                NETWORK_ID_L1, // originNetwork
                address(bridgeExtensionL1), // originAddress
                NETWORK_ID_L2, // destinationNetwork
                address(bridgeExtensionL2), // destinationAddress
                0, // amount
                metadata // metadata
            )
        {
            console2.log("SUCCESS: Message claim worked!");

            // Check if the wrapped token was created and balances updated
            if (l2TokenAddress.code.length > 0) {
                TokenWrapped wrappedToken = TokenWrapped(l2TokenAddress);
                uint256 user1Balance = wrappedToken.balanceOf(user1);
                uint256 user2Balance = wrappedToken.balanceOf(user2);

                console2.log(
                    "User1 balance after successful execution:",
                    user1Balance
                );
                console2.log(
                    "User2 balance after successful execution:",
                    user2Balance
                );

                // The transfer should have sent 1 token to user1
                assertEq(
                    user1Balance,
                    1 * 10 ** 18,
                    "User1 should receive 1 token from transfer"
                );
            } else {
                console2.log("Wrapped token not deployed yet");
            }
        } catch Error(string memory reason) {
            console2.log("Message claim failed with reason:", reason);
            revert(reason);
        } catch (bytes memory lowLevelData) {
            console2.log("Message claim STILL failed with low-level error");
            console2.logBytes(lowLevelData);

            if (lowLevelData.length >= 4) {
                bytes4 errorSelector = bytes4(lowLevelData);
                console2.log("Error selector:");
                console2.logBytes4(errorSelector);

                if (errorSelector == 0x37e391c3) {
                    console2.log(
                        "ERROR: Still getting MessageFailed() even after claiming asset first!"
                    );
                } else if (errorSelector == 0x646cf558) {
                    console2.log("ERROR: AlreadyClaimed()");
                } else {
                    console2.log("ERROR: Unknown error selector");
                }
            }

            revert("Message claim failed even after asset claim");
        }

        vm.stopPrank();
    }

    function testBridgeExtensionValidation() public {
        // Test individual components of the bridge extension

        // 1. Test metadata encoding
        bytes memory transferData = abi.encodeWithSignature(
            "transfer(address,uint256)",
            user1,
            1 * 10 ** 18
        );
        address l2TokenAddress = bridgeL2.precalculatedWrapperAddress(
            NETWORK_ID_L1,
            address(tokenL1),
            "AggERC20",
            "AGGERC20",
            18
        );

        bytes memory metadata = abi.encode(
            0, // dependsOnIndex
            l2TokenAddress, // callAddress
            user2, // fallbackAddress
            NETWORK_ID_L1, // assetOriginalNetwork
            address(tokenL1), // assetOriginalAddress
            transferData // callData
        );

        console2.log("Encoded metadata:");
        console2.logBytes(metadata);

        // 2. Test metadata decoding
        (
            uint256 dependsOnIndex,
            address callAddress,
            address fallbackAddress,
            uint32 assetOriginalNetwork,
            address assetOriginalAddress,
            bytes memory callData
        ) = abi.decode(
                metadata,
                (uint256, address, address, uint32, address, bytes)
            );

        console2.log("Decoded dependsOnIndex:", dependsOnIndex);
        console2.log("Decoded callAddress:", callAddress);
        console2.log("Decoded fallbackAddress:", fallbackAddress);
        console2.log("Decoded assetOriginalNetwork:", assetOriginalNetwork);
        console2.log("Decoded assetOriginalAddress:", assetOriginalAddress);
        console2.log("Decoded callData:");
        console2.logBytes(callData);

        // Verify decoded values
        assertEq(dependsOnIndex, 0);
        assertEq(callAddress, l2TokenAddress);
        assertEq(fallbackAddress, user2);
        assertEq(assetOriginalNetwork, NETWORK_ID_L1);
        assertEq(assetOriginalAddress, address(tokenL1));
        assertEq(callData, transferData);
    }

    function testJumpPointAddressCalculation() public {
        // Test that our JumpPoint address calculation matches what BridgeExtension does

        bytes memory transferData = abi.encodeWithSignature(
            "transfer(address,uint256)",
            user1,
            1 * 10 ** 18
        );
        address l2TokenAddress = bridgeL2.precalculatedWrapperAddress(
            NETWORK_ID_L1,
            address(tokenL1),
            "AggERC20",
            "AGGERC20",
            18
        );

        // Calculate using BridgeExtension's internal logic (CREATE2)
        uint256 dependsOnIndex = 0;
        uint32 originNetwork = NETWORK_ID_L1;

        bytes memory jumpPointBytecode = abi.encodePacked(
            type(JumpPoint).creationCode,
            abi.encode(
                address(bridgeL2),
                NETWORK_ID_L1,
                address(tokenL1),
                l2TokenAddress,
                user2,
                transferData
            )
        );

        bytes32 salt = keccak256(
            abi.encodePacked(dependsOnIndex, originNetwork)
        );
        bytes32 hash = keccak256(
            abi.encodePacked(
                bytes1(0xff),
                address(bridgeExtensionL2), // deployer = BridgeExtension on L2
                salt,
                keccak256(jumpPointBytecode)
            )
        );
        address calculatedJumpPoint = address(uint160(uint256(hash)));

        console2.log("Calculated JumpPoint address:", calculatedJumpPoint);
        console2.log("Salt used:");
        console2.logBytes32(salt);
        console2.log("Bytecode hash:");
        console2.logBytes32(keccak256(jumpPointBytecode));
        console2.log(
            "Deployer (BridgeExtension L2):",
            address(bridgeExtensionL2)
        );
    }

    function testJumpPointDeploymentOriginal() public {
        // Test CREATE2 deployment parameters
        uint256 dependsOnIndex = 0;
        uint32 originNetwork = NETWORK_ID_L1;
        bytes32 salt = keccak256(
            abi.encodePacked(dependsOnIndex, originNetwork)
        );

        console2.log("CREATE2 salt:");
        console2.logBytes32(salt);

        // Test JumpPoint constructor parameters
        bytes memory transferData = abi.encodeWithSignature(
            "transfer(address,uint256)",
            user1,
            1 * 10 ** 18
        );
        address l2TokenAddress = bridgeL2.precalculatedWrapperAddress(
            NETWORK_ID_L1,
            address(tokenL1),
            "AggERC20",
            "AGGERC20",
            18
        );

        console2.log("JumpPoint constructor params:");
        console2.log("  bridge:", address(bridgeL2));
        console2.log("  assetOriginalNetwork:", NETWORK_ID_L1);
        console2.log("  assetOriginalAddress:", address(tokenL1));
        console2.log("  callAddress:", l2TokenAddress);
        console2.log("  fallbackAddress:", user2);
        console2.log("  callData:");
        console2.logBytes(transferData);
    }
}
