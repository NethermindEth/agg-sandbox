// SPDX-License-Identifier: AGPL-3.0

pragma solidity ^0.8.22;

import "./IPolygonZkEVMErrors.sol";

interface IPolygonZkEVMEtrogErrors is IPolygonZkEVMErrors {
    /**
     * @dev Thrown when the caller is not the trusted sequencer
     */
    error OnlyRollupManager();

    /**
     * @dev Thrown when the caller is not the trusted sequencer
     */
    error NotEnoughPOLAmount();

    /**
     * @dev Thrown when the caller is not the trusted sequencer
     */
    error InvalidInitializeTransaction();

    /**
     * @dev Thrown when the caller is not the trusted sequencer
     */
    error GasTokenNetworkMustBeZeroOnEther();

    /**
     * @dev Thrown when the try to initialize with a gas token with huge metadata
     */
    error HugeTokenMetadataNotSupported();

    /**
     * @dev Thrown when trying force a batch during emergency state
     */
    error ForceBatchesNotAllowedOnEmergencyState();

    /**
     * @dev Thrown when the try to sequence force batches before the halt timeout period
     */
    error HaltTimeoutNotExpiredAfterEmergencyState();

    /**
     * @dev Thrown when the try to update the force batch address once is set to address(0)
     */
    error ForceBatchesDecentralized();

    /**
     * @dev Thrown when the last sequenced batch nmber does not match the init sequeced batch number
     */
    error InitSequencedBatchDoesNotMatch();

    /**
     * @dev Thrown when the max timestamp is out of range
     */
    error MaxTimestampSequenceInvalid();

    /**
     * @dev Thrown when l1 info tree leafCount does not exist
     */
    error L1InfoTreeLeafCountInvalid();

    /**
     * @dev Thrown when the acc input hash does not match the predicted by the sequencer
     */
    error FinalAccInputHashDoesNotMatch();
}
