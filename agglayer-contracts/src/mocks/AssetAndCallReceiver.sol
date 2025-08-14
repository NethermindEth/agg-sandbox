// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "../interfaces/IBridgeMessageReceiver.sol";

contract AssetAndCallReceiver is IBridgeMessageReceiver {
    uint256 public totalTransferred;
    uint256 public callCounter;

    error InvalidMsgValue(uint256 expected, uint256 actual);

    event AssetReceived(address indexed sender, uint256 amount);
    event CallExecuted(
        address indexed caller,
        uint256 assetAmount,
        uint256 totalTransferred,
        uint256 callCounter
    );
    
    event MessageReceived(
        address indexed originAddress,
        uint32 originNetwork,
        bytes data,
        uint256 ethAmount
    );

    // Payable constructor to accept initial funding
    constructor() payable {
        emit AssetReceived(msg.sender, msg.value);
    }

    // Receive function to accept ETH transfers from bridge claims
    receive() external payable {
        emit AssetReceived(msg.sender, msg.value);
    }

    // This function receives an asset (ETH) and processes a call.
    // The sent ETH (msg.value) must match the specified assetAmount.
    function processTransferAndCall(uint256 assetAmount) external payable {
        if (msg.value != assetAmount) {
            revert InvalidMsgValue(assetAmount, msg.value);
        }

        totalTransferred += assetAmount;
        callCounter++;

        emit AssetReceived(msg.sender, assetAmount);
        emit CallExecuted(
            msg.sender,
            assetAmount,
            totalTransferred,
            callCounter
        );
    }

    // Implementation of IBridgeMessageReceiver
    function onMessageReceived(
        address originAddress,
        uint32 originNetwork,
        bytes memory data
    ) external payable override {
        // Update counters
        totalTransferred += msg.value;
        callCounter++;
        
        // Emit events
        emit MessageReceived(originAddress, originNetwork, data, msg.value);
        emit AssetReceived(msg.sender, msg.value);
        
        // If there's data, try to decode and execute it
        if (data.length > 0) {
            // Try to decode the data as a function call to processTransferAndCall
            // This allows the bridge message to include function call data
            (bool success,) = address(this).call(data);
            if (success) {
                emit CallExecuted(
                    originAddress,
                    msg.value,
                    totalTransferred,
                    callCounter
                );
            }
        }
    }
}
