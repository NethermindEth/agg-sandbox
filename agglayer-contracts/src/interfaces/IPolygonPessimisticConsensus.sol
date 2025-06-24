// SPDX-License-Identifier: AGPL-3.0

pragma solidity ^0.8.22;

interface IPolygonPessimisticConsensus {
    function getConsensusHash() external view returns (bytes32);
}
