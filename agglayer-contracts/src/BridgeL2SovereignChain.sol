// SPDX-License-Identifier: AGPL-3.0

pragma solidity ^0.8.22;

import "./interfaces/IBridgeL2SovereignChains.sol";
import "./PolygonZkEVMBridgeV2.sol";

/**
 * Sovereign chains bridge that will be deployed on all Sovereign chains
 * Contract responsible to manage the token interactions with other networks
 * This contract is not meant to replace the current zkEVM bridge contract, but deployed on sovereign networks
 */
contract BridgeL2SovereignChain is PolygonZkEVMBridgeV2, IBridgeL2SovereignChains {
    using SafeERC20 for IERC20;

    // Map to store wrappedAddresses that are not mintable
    mapping(address wrappedAddress => bool isNotMintable) public wrappedAddressIsNotMintable;

    // Bridge manager address; can set custom mapping for any token. It's highly recommend to set a timelock at this address after bootstrapping phase
    address public bridgeManager;

    /**
     * @dev Emitted when a bridge manager is updated
     */
    event SetBridgeManager(address bridgeManager);

    /**
     * @dev Emitted when a claim is unset
     */
    event UnsetClaim(uint32 leafIndex, uint32 sourceBridgeNetwork);

    /**
     * @dev Emitted when a token address is remapped by a sovereign token address
     */
    event SetSovereignTokenAddress(
        uint32 originNetwork, address originTokenAddress, address sovereignTokenAddress, bool isNotMintable
    );

    /**
     * @dev Emitted when a legacy token is migrated to a new token
     */
    event MigrateLegacyToken(address sender, address legacyTokenAddress, address updatedTokenAddress, uint256 amount);

    /**
     * @dev Emitted when a remapped token is removed from mapping
     */
    event RemoveLegacySovereignTokenAddress(address sovereignTokenAddress);

    /**
     * @dev Emitted when a WETH address is remapped by a sovereign WETH address
     */
    event SetSovereignWETHAddress(address sovereignWETHTokenAddress, bool isNotMintable);

    /**
     * Disable initializers on the implementation following the best practices
     * For development/testing, we allow direct initialization
     */
    constructor() {
        // _disableInitializers(); // Commented out for direct deployment
    }

    /**
     * @param _networkID networkID
     * @param _gasTokenAddress gas token address
     * @param _gasTokenNetwork gas token network
     * @param _globalExitRootManager global exit root manager address
     * @param _polygonRollupManager Rollup manager address
     * @notice The value of `_polygonRollupManager` on the L2 deployment of the contract will be address(0), so
     * emergency state is not possible for the L2 deployment of the bridge, intentionally
     * @param _gasTokenMetadata Abi encoded gas token metadata
     * @param _bridgeManager bridge manager address
     * @param _sovereignWETHAddress sovereign WETH address
     * @param _sovereignWETHAddressIsNotMintable Flag to indicate if the wrapped ETH is not mintable
     */
    function initialize(
        uint32 _networkID,
        address _gasTokenAddress,
        uint32 _gasTokenNetwork,
        IBasePolygonZkEVMGlobalExitRoot _globalExitRootManager,
        address _polygonRollupManager,
        bytes memory _gasTokenMetadata,
        address _bridgeManager,
        address _sovereignWETHAddress,
        bool _sovereignWETHAddressIsNotMintable
    ) public virtual initializer {
        networkID = _networkID;
        globalExitRootManager = _globalExitRootManager;
        polygonRollupManager = _polygonRollupManager;
        bridgeManager = _bridgeManager;

        // Set gas token
        if (_gasTokenAddress == address(0)) {
            // Gas token will be ether
            if (_gasTokenNetwork != 0) {
                revert GasTokenNetworkMustBeZeroOnEther();
            }
            // Health check for sovereign WETH address
            if (_sovereignWETHAddress != address(0) || _sovereignWETHAddressIsNotMintable) {
                revert InvalidSovereignWETHAddressParams();
            }
            // WETHToken, gasTokenAddress and gasTokenNetwork will be 0
            // gasTokenMetadata will be empty
        } else {
            // Gas token will be an erc20
            gasTokenAddress = _gasTokenAddress;
            gasTokenNetwork = _gasTokenNetwork;
            gasTokenMetadata = _gasTokenMetadata;

            // Set sovereign weth token or create new if not provided
            if (_sovereignWETHAddress == address(0)) {
                // Health check for sovereign WETH address is mintable
                if (_sovereignWETHAddressIsNotMintable == true) {
                    revert InvalidSovereignWETHAddressParams();
                }
                // Create a wrapped token for WETH, with salt == 0
                WETHToken = _deployWrappedToken(
                    0, // salt
                    abi.encode("Wrapped Ether", "WETH", 18)
                );
            } else {
                WETHToken = TokenWrapped(_sovereignWETHAddress);
                wrappedAddressIsNotMintable[_sovereignWETHAddress] = _sovereignWETHAddressIsNotMintable;
            }
        }

        // Initialize OZ contracts
    }

    /**
     * @notice Override the function to prevent the contract from being initialized with this initializer
     */
    function initialize(
        uint32, // _networkID
        address, //_gasTokenAddress
        uint32, //_gasTokenNetwork
        IBasePolygonZkEVMGlobalExitRoot, //_globalExitRootManager
        address, //_polygonRollupManager
        bytes memory //_gasTokenMetadata
    ) external override(IPolygonZkEVMBridgeV2, PolygonZkEVMBridgeV2) initializer {
        revert InvalidInitializeFunction();
    }

    modifier onlyBridgeManager() {
        if (bridgeManager != msg.sender) {
            revert OnlyBridgeManager();
        }
        _;
    }

    /**
     * @notice Remap multiple wrapped tokens to a new sovereign token address
     * @dev This function is a "multi/batch call" to `setSovereignTokenAddress`
     * @param originNetworks Array of Origin networks
     * @param originTokenAddresses Array od Origin token addresses, 0 address is reserved for ether
     * @param sovereignTokenAddresses Array of Addresses of the sovereign wrapped token
     * @param isNotMintable Array of Flags to indicate if the wrapped token is not mintable
     */
    function setMultipleSovereignTokenAddress(
        uint32[] memory originNetworks,
        address[] memory originTokenAddresses,
        address[] memory sovereignTokenAddresses,
        bool[] memory isNotMintable
    ) external onlyBridgeManager {
        if (
            originNetworks.length != originTokenAddresses.length
                || originNetworks.length != sovereignTokenAddresses.length || originNetworks.length != isNotMintable.length
        ) {
            revert InputArraysLengthMismatch();
        }

        // Make multiple calls to setSovereignTokenAddress
        for (uint256 i = 0; i < sovereignTokenAddresses.length; i++) {
            _setSovereignTokenAddress(
                originNetworks[i], originTokenAddresses[i], sovereignTokenAddresses[i], isNotMintable[i]
            );
        }
    }

    /**
     * @notice Remap a wrapped token to a new sovereign token address
     * @dev This function is used to allow any existing token to be mapped with
     *      origin token.
     * @notice If this function is called multiple times for the same existingTokenAddress,
     * this will override the previous calls and only keep the last sovereignTokenAddress.
     * @notice The tokenInfoToWrappedToken mapping  value is replaced by the new sovereign address but it's not the case for the wrappedTokenToTokenInfo map where the value is added, this way user will always be able to withdraw their tokens
     * @notice The number of decimals between sovereign token and origin token is not checked, it doesn't affect the bridge functionality but the UI.
     * @param originNetwork Origin network
     * @param originTokenAddress Origin token address, 0 address is reserved for gas token address. If WETH address is zero, means this gas token is ether, else means is a custom erc20 gas token
     * @param sovereignTokenAddress Address of the sovereign wrapped token
     * @param isNotMintable Flag to indicate if the wrapped token is not mintable
     */
    function _setSovereignTokenAddress(
        uint32 originNetwork,
        address originTokenAddress,
        address sovereignTokenAddress,
        bool isNotMintable
    ) internal {
        // origin and sovereign token address are not 0
        if (originTokenAddress == address(0) || sovereignTokenAddress == address(0)) {
            revert InvalidZeroAddress();
        }
        // originNetwork != current network, wrapped tokens are always from other networks
        if (originNetwork == networkID) {
            revert OriginNetworkInvalid();
        }
        // Check if the token is already mapped
        if (wrappedTokenToTokenInfo[sovereignTokenAddress].originTokenAddress != address(0)) {
            revert TokenAlreadyMapped();
        }

        // Compute token info hash
        bytes32 tokenInfoHash = keccak256(abi.encodePacked(originNetwork, originTokenAddress));
        // Set the address of the wrapper
        tokenInfoToWrappedToken[tokenInfoHash] = sovereignTokenAddress;
        // Set the token info mapping
        // @note wrappedTokenToTokenInfo mapping is not overwritten while tokenInfoToWrappedToken it is
        wrappedTokenToTokenInfo[sovereignTokenAddress] = TokenInformation(originNetwork, originTokenAddress);
        wrappedAddressIsNotMintable[sovereignTokenAddress] = isNotMintable;
        emit SetSovereignTokenAddress(originNetwork, originTokenAddress, sovereignTokenAddress, isNotMintable);
    }

    /**
     * @notice Remove the address of a remapped token from the mapping. Used to stop supporting legacy sovereign tokens
     * @notice It also removes the token from the isNotMintable mapping
     * @notice Although the token is removed from the mapping, the user will still be able to withdraw their tokens using tokenInfoToWrappedToken mapping
     * @param legacySovereignTokenAddress Address of the sovereign wrapped token
     */
    function removeLegacySovereignTokenAddress(address legacySovereignTokenAddress) external onlyBridgeManager {
        // Only allow to remove already remapped tokens
        TokenInformation memory tokenInfo = wrappedTokenToTokenInfo[legacySovereignTokenAddress];
        bytes32 tokenInfoHash = keccak256(abi.encodePacked(tokenInfo.originNetwork, tokenInfo.originTokenAddress));

        if (
            tokenInfoToWrappedToken[tokenInfoHash] == address(0)
                || tokenInfoToWrappedToken[tokenInfoHash] == legacySovereignTokenAddress
        ) {
            revert TokenNotRemapped();
        }
        delete wrappedTokenToTokenInfo[legacySovereignTokenAddress];
        delete wrappedAddressIsNotMintable[legacySovereignTokenAddress];
        emit RemoveLegacySovereignTokenAddress(legacySovereignTokenAddress);
    }

    /**
     * @notice Set the custom wrapper for weth
     * @notice If this function is called multiple times this will override the previous calls and only keep the last WETHToken.
     * @notice WETH will not maintain legacy versions.Users easily should be able to unwrapp the legacy WETH and unwrapp it with the new one.
     * @param sovereignWETHTokenAddress Address of the sovereign weth token
     * @param isNotMintable Flag to indicate if the wrapped token is not mintable
     */
    function setSovereignWETHAddress(address sovereignWETHTokenAddress, bool isNotMintable)
        external
        onlyBridgeManager
    {
        if (gasTokenAddress == address(0)) {
            revert WETHRemappingNotSupportedOnGasTokenNetworks();
        }
        WETHToken = TokenWrapped(sovereignWETHTokenAddress);
        wrappedAddressIsNotMintable[sovereignWETHTokenAddress] = isNotMintable;
        emit SetSovereignWETHAddress(sovereignWETHTokenAddress, isNotMintable);
    }

    /**
     * @notice Moves old native or remapped token (legacy) to the new mapped token. If the token is mintable, it will be burnt and minted, otherwise it will be transferred
     * @param legacyTokenAddress Address of legacy token to migrate
     * @param amount Legacy token balance to migrate
     */
    function migrateLegacyToken(address legacyTokenAddress, uint256 amount) external {
        // Get current wrapped token address
        TokenInformation memory legacyTokenInfo = wrappedTokenToTokenInfo[legacyTokenAddress];
        if (legacyTokenInfo.originTokenAddress == address(0)) {
            revert TokenNotMapped();
        }

        // Check current token mapped is proposed updatedTokenAddress
        address currentTokenAddress = tokenInfoToWrappedToken[keccak256(
            abi.encodePacked(legacyTokenInfo.originNetwork, legacyTokenInfo.originTokenAddress)
        )];

        if (currentTokenAddress == legacyTokenAddress) {
            revert TokenAlreadyUpdated();
        }

        // Proceed to migrate the token
        _bridgeWrappedAsset(TokenWrapped(legacyTokenAddress), amount);
        _claimWrappedAsset(TokenWrapped(currentTokenAddress), msg.sender, amount);

        // Trigger event
        emit MigrateLegacyToken(msg.sender, legacyTokenAddress, currentTokenAddress, amount);
    }

    /**
     * @notice unset multiple claims from the claimedBitmap
     * @dev This function is a "multi/batch call" to `unsetClaimedBitmap`
     * @param leafIndexes Array of Index
     * @param sourceBridgeNetworks Array of Origin networks
     */
    function unsetMultipleClaimedBitmap(uint32[] memory leafIndexes, uint32[] memory sourceBridgeNetworks)
        external
        onlyBridgeManager
    {
        if (leafIndexes.length != sourceBridgeNetworks.length) {
            revert InputArraysLengthMismatch();
        }

        for (uint256 i = 0; i < leafIndexes.length; i++) {
            _unsetClaimedBitmap(leafIndexes[i], sourceBridgeNetworks[i]);
        }
    }

    /**
     * @notice Updated bridge manager address, recommended to set a timelock at this address after bootstrapping phase
     * @param _bridgeManager Bridge manager address
     */
    function setBridgeManager(address _bridgeManager) external onlyBridgeManager {
        bridgeManager = _bridgeManager;
        emit SetBridgeManager(bridgeManager);
    }

    /**
     * @notice Burn tokens from wrapped token to execute the bridge, if the token is not mintable it will be transferred
     * note This function has been extracted to be able to override it by other contracts like Bridge2SovereignChain
     * @param tokenWrapped Wrapped token to burnt
     * @param amount Amount of tokens
     */
    function _bridgeWrappedAsset(TokenWrapped tokenWrapped, uint256 amount) internal override {
        // The token is either (1) a correctly wrapped token from another network
        // or (2) wrapped with custom contract from origin network
        if (wrappedAddressIsNotMintable[address(tokenWrapped)]) {
            // Don't use burn but transfer to bridge
            IERC20(address(tokenWrapped)).transferFrom(msg.sender, address(this), amount);
        } else {
            // Burn tokens
            tokenWrapped.burn(msg.sender, amount);
        }
    }

    /**
     * @notice Mints tokens from wrapped token to proceed with the claim, if the token is not mintable it will be transferred
     * note This function has been extracted to be able to override it by other contracts like BridgeL2SovereignChain
     * @param tokenWrapped Wrapped token to mint
     * @param destinationAddress Minted token receiver
     * @param amount Amount of tokens
     */
    function _claimWrappedAsset(TokenWrapped tokenWrapped, address destinationAddress, uint256 amount)
        internal
        override
    {
        // If is not mintable transfer instead of mint
        if (wrappedAddressIsNotMintable[address(tokenWrapped)]) {
            // Transfer tokens
            IERC20(address(tokenWrapped)).transfer(destinationAddress, amount);
        } else {
            // Claim tokens
            tokenWrapped.mint(destinationAddress, amount);
        }
    }

    /*
     * @notice unset a claim from the claimedBitmap
     * @param leafIndex Index
     * @param sourceBridgeNetwork Origin network
     */
    function _unsetClaimedBitmap(uint32 leafIndex, uint32 sourceBridgeNetwork) private {
        uint256 globalIndex = uint256(leafIndex) + uint256(sourceBridgeNetwork) * _MAX_LEAFS_PER_NETWORK;
        (uint256 wordPos, uint256 bitPos) = _bitmapPositions(globalIndex);
        uint256 mask = 1 << bitPos;
        uint256 flipped = claimedBitMap[wordPos] ^= mask;
        if (flipped & mask != 0) {
            revert ClaimNotSet();
        }
        emit UnsetClaim(leafIndex, sourceBridgeNetwork);
    }

    /**
     * @notice Function to check if an index is claimed or not
     * @dev function overridden to improve a bit the performance and bytecode not checking unnecessary conditions for sovereign chains context
     * @param leafIndex Index
     * @param sourceBridgeNetwork Origin network
     */
    function isClaimed(uint32 leafIndex, uint32 sourceBridgeNetwork) external view override returns (bool) {
        uint256 globalIndex = uint256(leafIndex) + uint256(sourceBridgeNetwork) * _MAX_LEAFS_PER_NETWORK;

        (uint256 wordPos, uint256 bitPos) = _bitmapPositions(globalIndex);
        uint256 mask = (1 << bitPos);
        return (claimedBitMap[wordPos] & mask) == mask;
    }

    /**
     * @notice Function to check that an index is not claimed and set it as claimed
     * @dev function overridden to improve a bit the performance and bytecode not checking unnecessary conditions for sovereign chains context
     * @param leafIndex Index
     * @param sourceBridgeNetwork Origin network
     */
    function _setAndCheckClaimed(uint32 leafIndex, uint32 sourceBridgeNetwork) internal override {
        uint256 globalIndex = uint256(leafIndex) + uint256(sourceBridgeNetwork) * _MAX_LEAFS_PER_NETWORK;
        (uint256 wordPos, uint256 bitPos) = _bitmapPositions(globalIndex);
        uint256 mask = 1 << bitPos;
        uint256 flipped = claimedBitMap[wordPos] ^= mask;
        if (flipped & mask == 0) {
            revert AlreadyClaimed();
        }
    }

    /**
     * @notice Function to call token permit method of extended ERC20
     * @dev function overridden from PolygonZkEVMBridgeV2 to improve a bit the performance and bytecode not checking unnecessary conditions for sovereign chains context
     *  + @param token ERC20 token address
     * @param amount Quantity that is expected to be allowed
     * @param permitData Raw data of the call `permit` of the token
     */
    function _permit(address token, uint256 amount, bytes calldata permitData) internal override {
        bytes4 sig = bytes4(permitData[:4]);
        if (sig == _PERMIT_SIGNATURE) {
            (address owner, address spender, uint256 value, uint256 deadline, uint8 v, bytes32 r, bytes32 s) =
                abi.decode(permitData[4:], (address, address, uint256, uint256, uint8, bytes32, bytes32));

            if (value != amount) {
                revert NotValidAmount();
            }

            // we call without checking the result, in case it fails and he doesn't have enough balance
            // the following transferFrom should be fail. This prevents DoS attacks from using a signature
            // before the smartcontract call
            /* solhint-disable avoid-low-level-calls */
            address(token).call(abi.encodeWithSelector(_PERMIT_SIGNATURE, owner, spender, value, deadline, v, r, s));
        } else {
            if (sig != _PERMIT_SIGNATURE_DAI) {
                revert NotValidSignature();
            }

            (
                address holder,
                address spender,
                uint256 nonce,
                uint256 expiry,
                bool allowed,
                uint8 v,
                bytes32 r,
                bytes32 s
            ) = abi.decode(permitData[4:], (address, address, uint256, uint256, bool, uint8, bytes32, bytes32));

            // we call without checking the result, in case it fails and he doesn't have enough balance
            // the following transferFrom should be fail. This prevents DoS attacks from using a signature
            // before the smartcontract call
            /* solhint-disable avoid-low-level-calls */
            address(token).call(
                abi.encodeWithSelector(_PERMIT_SIGNATURE_DAI, holder, spender, nonce, expiry, allowed, v, r, s)
            );
        }
    }

    // @note This function is not used in the current implementation. We overwrite it to improve deployed bytecode size
    function activateEmergencyState() external pure override(IPolygonZkEVMBridgeV2, PolygonZkEVMBridgeV2) {
        revert EmergencyStateNotAllowed();
    }

    function deactivateEmergencyState() external pure override(IPolygonZkEVMBridgeV2, PolygonZkEVMBridgeV2) {
        revert EmergencyStateNotAllowed();
    }
}
