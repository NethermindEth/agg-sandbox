// SPDX-License-Identifier: AGPL-3.0

pragma solidity ^0.8.22;

import "./IVerifierRollup.sol";
import "./IPolygonRollupBase.sol";
import {ITransparentUpgradeableProxy} from "@openzeppelin/contracts/proxy/transparent/TransparentUpgradeableProxy.sol";

interface IPolygonRollupManager {
    /**
     * @dev Thrown when sender is not the PolygonZkEVM address
     */
    error UpdateToSameRollupTypeID();

    /**
     * @dev Thrown when sender is not the PolygonZkEVM address
     */
    error RollupMustExist();

    /**
     * @dev Thrown when sender is not the PolygonZkEVM address
     */
    error SenderMustBeRollup();

    /**
     * @dev Thrown when sender is not the PolygonZkEVM address
     */
    error TrustedAggregatorTimeoutNotExpired();

    /**
     * @dev Thrown when sender is not the PolygonZkEVM address
     */
    error ExceedMaxVerifyBatches();

    /**
     * @dev Thrown when attempting to access a pending state that does not exist
     */
    error PendingStateDoesNotExist();

    /**
     * @dev Thrown when the init num batch does not match with the one in the pending state
     */
    error InitNumBatchDoesNotMatchPendingState();

    /**
     * @dev Thrown when the old state root of a certain batch does not exist
     */
    error OldStateRootDoesNotExist();

    /**
     * @dev Thrown when the init verification batch is above the last verification batch
     */
    error InitNumBatchAboveLastVerifiedBatch();

    /**
     * @dev Thrown when the final verification batch is below or equal the last verification batch
     */
    error FinalNumBatchBelowLastVerifiedBatch();

    /**
     * @dev Thrown when the zkproof is not valid
     */
    error InvalidProof();

    /**
     * @dev Thrown when attempting to consolidate a pending state not yet consolidable
     */
    error PendingStateNotConsolidable();

    /**
     * @dev Thrown when attempting to consolidate a pending state that is already consolidated or does not exist
     */
    error PendingStateInvalid();

    /**
     * @dev Thrown when the new accumulate input hash does not exist
     */
    error NewAccInputHashDoesNotExist();

    /**
     * @dev Thrown when the new state root is not inside prime
     */
    error NewStateRootNotInsidePrime();

    /**
     * @dev Thrown when the final pending state num is not in a valid range
     */
    error FinalPendingStateNumInvalid();

    /**
     * @dev Thrown when the final num batch does not match with the one in the pending state
     */
    error FinalNumBatchDoesNotMatchPendingState();

    /**
     * @dev Thrown when the stored root matches the new root proving a different state
     */
    error StoredRootMustBeDifferentThanNewRoot();

    /**
     * @dev Thrown when the halt timeout is not expired when attempting to activate the emergency state
     */
    error HaltTimeoutNotExpired();

    /**
     * @dev Thrown when the old accumulate input hash does not exist
     */
    error OldAccInputHashDoesNotExist();

    /**
     * @dev Thrown when attempting to set a new trusted aggregator timeout equal or bigger than current one
     */
    error NewTrustedAggregatorTimeoutMustBeLower();

    /**
     * @dev Thrown when attempting to set a new pending state timeout equal or bigger than current one
     */
    error NewPendingStateTimeoutMustBeLower();

    /**
     * @dev Thrown when attempting to set a new multiplier batch fee in a invalid range of values
     */
    error InvalidRangeMultiplierBatchFee();

    /**
     * @dev Thrown when attempting to set a batch time target in an invalid range of values
     */
    error InvalidRangeBatchTimeTarget();

    /**
     * @dev Thrown when the caller is not the pending admin
     */
    error ChainIDAlreadyExist();

    /**
     * @dev Thrown when the caller is not the pending admin
     */
    error MustSequenceSomeBatch();

    /**
     * @dev When a rollup type does not exist
     */
    error RollupTypeDoesNotExist();

    /**
     * @dev When a rollup type does not exist
     */
    error RollupTypeObsolete();

    /**
     * @dev When a rollup type does not exist
     */
    error InitBatchMustMatchCurrentForkID();

    /**
     * @dev When a rollup type does not exist
     */
    error UpdateNotCompatible();

    /**
     * @dev When a rollup type does not exist
     */
    error BatchFeeOutOfRange();

    /**
     * @dev When a rollup type does not exist
     */
    error AllzkEVMSequencedBatchesMustBeVerified();

    /**
     * @dev When adding an existing rollup where the rollup address already was added
     */
    error RollupAddressAlreadyExist();

    /**
     * @dev When verifying proof for multiple roolups and they are not ordered by ID
     */
    error RollupIDNotAscendingOrder();

    /**
     * @dev When try to create a new rollup and set a chainID bigger than 32 bits
     */
    error ChainIDOutOfRange();

    /**
     * @dev When try to upgrade a rollup a sender that's not the admin of the rollup
     */
    error OnlyRollupAdmin();

    /**
     * @dev When try to update a rollup with sequences pending to verify
     */
    error AllSequencedMustBeVerified();

    /**
     * @dev Thrown when do not sequence any blob
     */
    error MustSequenceSomeBlob();

    /**
     * @dev Thrown when the final verification sequence is below or equal the last verification sequence
     */
    error FinalNumSequenceBelowLastVerifiedSequence();

    /**
     * @dev When the init sequence was verified in another forkID
     */
    error InitSequenceMustMatchCurrentForkID();

    /**
     * @dev Thrown when the init num sequence does not match with the one in the pending state
     */
    error InitSequenceNumDoesNotMatchPendingState();

    /**
     * @dev Thrown when the final num sequence does not match with the one in the pending state
     */
    error FinalNumSequenceDoesNotMatchPendingState();

    /**
     * @dev Thrown when attempting to set a new multiplier zkgas in a invalid range of values
     */
    error InvalidRangeMultiplierZkGasPrice();

    /**
     * @dev Thrown when attempting to set a seuqnece time target in an invalid range of values
     */
    error InvalidRangeSequenceTimeTarget();

    /**
     * @dev When a set a zkgasprice out of range
     */
    error zkGasPriceOfRange();

    /**
     * @dev Cannot update from network admin with unconsolidated pending state
     */
    error CannotUpdateWithUnconsolidatedPendingState();

    /**
     * @dev Try to verify batches without any sequence data
     */
    error EmptyVerifySequencesData();

    /**
     * @dev Update to old rollup ID
     */
    error UpdateToOldRollupTypeID();

    /**
     * @dev All batches must be verified before the upgrade
     */
    error AllBatchesMustBeVerified();

    /**
     * @dev Rollback batch is not sequenced
     */
    error RollbackBatchIsNotValid();

    /**
     * @dev Rollback batch is not the end of any sequence
     */
    error RollbackBatchIsNotEndOfSequence();

    /**
     * @dev rollbackBatches is called from a non authorized address
     */
    error NotAllowedAddress();

    /**
     * @dev Invalid Rollup type parameters
     */
    error InvalidRollupType();

    /**
     * @dev Invalid Rollup parameters
     */
    error InvalidRollup();

    /**
     * @dev Not valid L1 info tree leaf count
     */
    error L1InfoTreeLeafCountInvalid();

    /**
     * @dev Only State Transition Chains
     */
    error OnlyStateTransitionChains();

    /**
     * @dev Pending state num exist
     */
    error PendingStateNumExist();

    /**
     * @dev Only Chains with Pesismistic proofs
     */
    error OnlyChainsWithPessimisticProofs();

    /**
     * @dev Invalid Pessimistic proof
     */
    error InvalidPessimisticProof();

    /**
     * @dev Invalid Verifier Type when getting rollup data
     */
    error InvalidVerifierType();

    enum VerifierType {
        StateTransition,
        Pessimistic
    }

    function addNewRollupType(
        address consensusImplementation,
        address verifier,
        uint64 forkID,
        VerifierType rollupVerifierType,
        bytes32 initRoot,
        string memory description,
        bytes32 programVKey
    ) external;

    function obsoleteRollupType(uint32 rollupTypeID) external;

    function createNewRollup(
        uint32 rollupTypeID,
        uint64 chainID,
        address admin,
        address sequencer,
        address gasTokenAddress,
        string memory sequencerURL,
        string memory networkName
    ) external;

    function addExistingRollup(
        IPolygonRollupBase rollupAddress,
        address verifier,
        uint64 forkID,
        uint64 chainID,
        bytes32 initRoot,
        VerifierType rollupVerifierType,
        bytes32 programVKey
    ) external;

    function updateRollupByRollupAdmin(ITransparentUpgradeableProxy rollupContract, uint32 newRollupTypeID) external;

    function updateRollup(ITransparentUpgradeableProxy rollupContract, uint32 newRollupTypeID, bytes memory upgradeData)
        external;

    function rollbackBatches(IPolygonRollupBase rollupContract, uint64 targetBatch) external;

    function onSequenceBatches(uint64 newSequencedBatches, bytes32 newAccInputHash) external returns (uint64);

    function verifyBatchesTrustedAggregator(
        uint32 rollupID,
        uint64 pendingStateNum,
        uint64 initNumBatch,
        uint64 finalNewBatch,
        bytes32 newLocalExitRoot,
        bytes32 newStateRoot,
        address beneficiary,
        bytes32[24] calldata proof
    ) external;

    function verifyPessimisticTrustedAggregator(
        uint32 rollupID,
        uint32 l1InfoTreeLeafCount,
        bytes32 newLocalExitRoot,
        bytes32 newPessimisticRoot,
        bytes calldata proof
    ) external;

    function activateEmergencyState() external;

    function deactivateEmergencyState() external;

    function setBatchFee(uint256 newBatchFee) external;

    function getRollupExitRoot() external view returns (bytes32);

    function getLastVerifiedBatch(uint32 rollupID) external view returns (uint64);

    function calculateRewardPerBatch() external view returns (uint256);

    function getBatchFee() external view returns (uint256);

    function getForcedBatchFee() external view returns (uint256);

    function getInputPessimisticBytes(
        uint32 rollupID,
        bytes32 selectedGlobalExitRoot,
        bytes32 newLocalExitRoot,
        bytes32 newPessimisticRoot
    ) external view returns (bytes memory);

    function getInputSnarkBytes(
        uint32 rollupID,
        uint64 initNumBatch,
        uint64 finalNewBatch,
        bytes32 newLocalExitRoot,
        bytes32 oldStateRoot,
        bytes32 newStateRoot
    ) external view returns (bytes memory);

    function getRollupBatchNumToStateRoot(uint32 rollupID, uint64 batchNum) external view returns (bytes32);

    function lastDeactivatedEmergencyStateTimestamp() external view returns (uint64);
}
