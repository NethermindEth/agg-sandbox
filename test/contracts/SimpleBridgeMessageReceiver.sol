// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IBridgeMessageReceiver {
    function onMessageReceived(address originAddress, uint32 originNetwork, bytes memory data) external payable;
}

contract SimpleBridgeMessageReceiver is IBridgeMessageReceiver {
    // Events
    event MessageReceived(
        address indexed originAddress,
        uint32 indexed originNetwork,
        bytes data,
        uint256 value
    );
    
    event EthReceived(address indexed from, uint256 amount);
    
    // State variables
    mapping(bytes32 => bool) public processedMessages;
    uint256 public totalMessagesReceived;
    uint256 public totalEthReceived;
    
    // Last message details
    address public lastOriginAddress;
    uint32 public lastOriginNetwork;
    bytes public lastMessageData;
    uint256 public lastMessageValue;
    
    /**
     * @notice Handles incoming bridge messages
     * @param originAddress The address that initiated the bridge on the origin network
     * @param originNetwork The network ID where the message originated
     * @param data The message data
     */
    function onMessageReceived(
        address originAddress,
        uint32 originNetwork,
        bytes memory data
    ) external payable override {
        // Store message details
        lastOriginAddress = originAddress;
        lastOriginNetwork = originNetwork;
        lastMessageData = data;
        lastMessageValue = msg.value;
        
        // Update counters
        totalMessagesReceived++;
        totalEthReceived += msg.value;
        
        // Create unique message ID
        bytes32 messageId = keccak256(abi.encodePacked(originAddress, originNetwork, data, block.timestamp));
        processedMessages[messageId] = true;
        
        // Emit event
        emit MessageReceived(originAddress, originNetwork, data, msg.value);
        
        if (msg.value > 0) {
            emit EthReceived(msg.sender, msg.value);
        }
    }
    
    /**
     * @notice Get the last received message details
     */
    function getLastMessage() external view returns (
        address originAddress,
        uint32 originNetwork,
        bytes memory data,
        uint256 value
    ) {
        return (lastOriginAddress, lastOriginNetwork, lastMessageData, lastMessageValue);
    }
    
    /**
     * @notice Allow the contract to receive ETH directly
     */
    receive() external payable {
        emit EthReceived(msg.sender, msg.value);
        totalEthReceived += msg.value;
    }
    
    /**
     * @notice Withdraw accumulated ETH (for testing)
     */
    function withdraw() external {
        uint256 balance = address(this).balance;
        require(balance > 0, "No ETH to withdraw");
        payable(msg.sender).transfer(balance);
    }
}