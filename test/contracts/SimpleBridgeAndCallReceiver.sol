// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IERC20 {
    function transfer(address to, uint256 amount) external returns (bool);
    function balanceOf(address account) external view returns (uint256);
}

contract SimpleBridgeAndCallReceiver {
    event CallReceived(
        address indexed token,
        uint256 amount,
        address indexed from,
        string message
    );
    
    event TokensReceived(
        address indexed token,
        uint256 amount,
        address indexed from
    );
    
    struct CallData {
        address token;
        uint256 amount;
        address from;
        string message;
        uint256 timestamp;
    }
    
    CallData[] public callHistory;
    mapping(address => uint256) public tokenBalances;
    
    // Function to be called by bridge-and-call
    function receiveTokensWithMessage(
        address token,
        uint256 amount,
        string memory message
    ) external {
        // Record the call
        callHistory.push(CallData({
            token: token,
            amount: amount,
            from: msg.sender,
            message: message,
            timestamp: block.timestamp
        }));
        
        // Update token balance tracking
        tokenBalances[token] += amount;
        
        // Emit events
        emit CallReceived(token, amount, msg.sender, message);
        emit TokensReceived(token, amount, msg.sender);
    }
    
    // Get the number of calls received
    function getCallCount() external view returns (uint256) {
        return callHistory.length;
    }
    
    // Get specific call data
    function getCall(uint256 index) external view returns (
        address token,
        uint256 amount,
        address from,
        string memory message,
        uint256 timestamp
    ) {
        require(index < callHistory.length, "Index out of bounds");
        CallData memory call = callHistory[index];
        return (call.token, call.amount, call.from, call.message, call.timestamp);
    }
    
    // Get last call message
    function getLastMessage() external view returns (string memory) {
        require(callHistory.length > 0, "No calls received");
        return callHistory[callHistory.length - 1].message;
    }
    
    // Transfer tokens out (for testing)
    function withdrawTokens(address token, address to, uint256 amount) external {
        require(IERC20(token).transfer(to, amount), "Transfer failed");
        tokenBalances[token] -= amount;
    }
}