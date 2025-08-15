// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "../interfaces/IBridgeMessageReceiver.sol";

contract Counter is IBridgeMessageReceiver {
    uint256 public count;
    uint256 public totalEthReceived;
    mapping(address => uint256) public userCounts;

    event Incremented(address indexed caller, uint256 newCount, uint256 ethAmount);
    event MessageReceived(address indexed originAddress, uint32 originNetwork, bytes data, uint256 ethAmount);
    event EthReceived(address indexed sender, uint256 amount);

    constructor() payable {}

    receive() external payable {
        emit EthReceived(msg.sender, msg.value);
    }

    function increment() external payable {
        count++;
        userCounts[msg.sender]++;

        if (msg.value > 0) {
            totalEthReceived += msg.value;
            emit EthReceived(msg.sender, msg.value);
        }

        emit Incremented(msg.sender, count, msg.value);
    }

    function incrementBy(uint256 amount) external payable {
        count += amount;
        userCounts[msg.sender] += amount;

        if (msg.value > 0) {
            totalEthReceived += msg.value;
            emit EthReceived(msg.sender, msg.value);
        }

        emit Incremented(msg.sender, count, msg.value);
    }

    function onMessageReceived(address originAddress, uint32 originNetwork, bytes memory data)
        external
        payable
        override
    {
        if (msg.value > 0) {
            totalEthReceived += msg.value;
            emit EthReceived(msg.sender, msg.value);
        }

        emit MessageReceived(originAddress, originNetwork, data, msg.value);

        if (data.length > 0) {
            (bool success,) = address(this).call(data);
            require(success, "Contract call failed");
        }
    }

    function getCount() external view returns (uint256) {
        return count;
    }

    function getUserCount(address user) external view returns (uint256) {
        return userCounts[user];
    }
}
