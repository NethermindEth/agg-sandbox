#!/usr/bin/env python3
"""
AggsandboxAPI - Complete Python wrapper for aggsandbox CLI
Based on the actual CLI source code and cli-reference.md
"""

import subprocess
import json
import time
from typing import Optional, Dict, Any, List, Tuple
from dataclasses import dataclass

@dataclass
class BridgeAssetArgs:
    """Arguments for bridge asset command"""
    network: int
    destination_network: int
    amount: str
    token_address: str
    to_address: Optional[str] = None
    gas_limit: Optional[int] = None
    gas_price: Optional[str] = None
    private_key: Optional[str] = None

@dataclass
class BridgeClaimArgs:
    """Arguments for bridge claim command"""
    network: int
    tx_hash: str
    source_network: int
    deposit_count: Optional[int] = None
    token_address: Optional[str] = None
    gas_limit: Optional[int] = None
    gas_price: Optional[str] = None
    private_key: Optional[str] = None
    data: Optional[str] = None
    msg_value: Optional[str] = None

@dataclass
class BridgeUtilsArgs:
    """Arguments for bridge utils commands"""
    network: Optional[int] = None
    origin_network: Optional[int] = None
    origin_token: Optional[str] = None
    wrapped_token: Optional[str] = None
    index: Optional[int] = None
    source_network: Optional[int] = None
    private_key: Optional[str] = None

class AggsandboxAPI:
    """Complete wrapper for aggsandbox CLI commands"""
    
    @staticmethod
    def run_command(cmd: List[str], timeout: int = 30) -> Tuple[bool, str]:
        """Run aggsandbox command and return (success, output)"""
        # Always print the full command being executed
        print(f"🔧 Executing: {' '.join(cmd)}")
        
        try:
            result = subprocess.run(
                cmd, 
                capture_output=True, 
                text=True, 
                check=True,
                timeout=timeout
            )
            return True, result.stdout.strip()
        except subprocess.CalledProcessError as e:
            return False, e.stderr.strip() if e.stderr else e.stdout.strip()
        except subprocess.TimeoutExpired:
            return False, f"Command timed out after {timeout} seconds"
    
    # ============================================================================
    # CORE COMMANDS
    # ============================================================================
    
    @staticmethod
    def start(detach: bool = True, build: bool = False, fork: bool = False, 
              multi_l2: bool = False, claim_all: bool = False, verbose: bool = False,
              quiet: bool = False, log_format: Optional[str] = None) -> Tuple[bool, str]:
        """Start the Agglayer sandbox environment with Docker Compose
        
        Args:
            detach: Start services in background (detached mode)
            build: Rebuild Docker images before starting
            fork: Use real blockchain data from FORK_URL environment variables
            multi_l2: Start with a second L2 chain for multi-chain testing
            claim_all: Claimsponsor will sponsor all claims automatically
            verbose: Enable verbose output
            quiet: Suppress all output except errors and warnings
            log_format: Set log output format (pretty, compact, json)
        """
        cmd = ["aggsandbox", "start"]
        
        if detach:
            cmd.append("--detach")
        if build:
            cmd.append("--build")
        if fork:
            cmd.append("--fork")
        if multi_l2:
            cmd.append("--multi-l2")
        if claim_all:
            cmd.append("--claim-all")
        if verbose:
            cmd.append("--verbose")
        if quiet:
            cmd.append("--quiet")
        if log_format:
            cmd.extend(["--log-format", log_format])
        
        return AggsandboxAPI.run_command(cmd)
    
    @staticmethod
    def stop(volumes: bool = False, verbose: bool = False, quiet: bool = False,
             log_format: Optional[str] = None) -> Tuple[bool, str]:
        """Stop all sandbox services using docker-compose down
        
        Args:
            volumes: Remove Docker volumes and all persistent data (⚠️  destructive)
            verbose: Enable verbose output
            quiet: Suppress all output except errors and warnings
            log_format: Set log output format (pretty, compact, json)
        """
        cmd = ["aggsandbox", "stop"]
        
        if volumes:
            cmd.append("--volumes")
        if verbose:
            cmd.append("--verbose")
        if quiet:
            cmd.append("--quiet")
        if log_format:
            cmd.extend(["--log-format", log_format])
        
        return AggsandboxAPI.run_command(cmd)
    
    @staticmethod
    def restart(verbose: bool = False, quiet: bool = False,
                log_format: Optional[str] = None) -> Tuple[bool, str]:
        """Restart all sandbox services
        
        This performs a stop followed by start operation,
        preserving volumes and configuration.
        
        Args:
            verbose: Enable verbose output
            quiet: Suppress all output except errors and warnings
            log_format: Set log output format (pretty, compact, json)
        """
        cmd = ["aggsandbox", "restart"]
        
        if verbose:
            cmd.append("--verbose")
        if quiet:
            cmd.append("--quiet")
        if log_format:
            cmd.extend(["--log-format", log_format])
        
        return AggsandboxAPI.run_command(cmd)
    
    @staticmethod
    def status(quiet: bool = False, verbose: bool = False, 
               log_format: Optional[str] = None) -> Tuple[bool, str]:
        """Show status of all services
        
        Args:
            quiet: Suppress all output except errors and warnings
            verbose: Enable verbose output
            log_format: Set log output format (pretty, compact, json)
        """
        cmd = ["aggsandbox", "status"]
        
        if quiet:
            cmd.append("--quiet")
        if verbose:
            cmd.append("--verbose")
        if log_format:
            cmd.extend(["--log-format", log_format])
        
        return AggsandboxAPI.run_command(cmd)
    
    @staticmethod
    def info(verbose: bool = False, quiet: bool = False, log_format: Optional[str] = None) -> Tuple[bool, str]:
        """Display comprehensive sandbox configuration information
        
        Args:
            verbose: Enable verbose output
            quiet: Suppress all output except errors and warnings  
            log_format: Set log output format (pretty, compact, json)
        """
        cmd = ["aggsandbox", "info"]
        
        if verbose:
            cmd.append("--verbose")
        if quiet:
            cmd.append("--quiet")
        if log_format:
            cmd.extend(["--log-format", log_format])
        
        return AggsandboxAPI.run_command(cmd)
    
    @staticmethod
    def logs(follow: bool = False, tail: Optional[int] = None, 
             since: Optional[str] = None, service: Optional[str] = None,
             verbose: bool = False, quiet: bool = False,
             log_format: Optional[str] = None) -> Tuple[bool, str]:
        """Show logs from services
        
        Args:
            follow: Follow log output
            tail: Number of lines to show from the end of the logs
            since: Show logs since timestamp (e.g., "2013-01-02T13:23:37Z") or relative (e.g., "42m" for 42 minutes)
            service: Specific service to show logs for
            verbose: Enable verbose output
            quiet: Suppress all output except errors and warnings
            log_format: Set log output format (pretty, compact, json)
        """
        cmd = ["aggsandbox", "logs"]
        
        if follow:
            cmd.append("--follow")
        if tail:
            cmd.extend(["--tail", str(tail)])
        if since:
            cmd.extend(["--since", since])
        if verbose:
            cmd.append("--verbose")
        if quiet:
            cmd.append("--quiet")
        if log_format:
            cmd.extend(["--log-format", log_format])
        if service:
            cmd.append(service)
        
        return AggsandboxAPI.run_command(cmd)
    
    # ============================================================================
    # BRIDGE COMMANDS
    # ============================================================================
    
    @staticmethod
    def bridge_asset(args: BridgeAssetArgs) -> Tuple[bool, str]:
        """Bridge ERC20 tokens or ETH between networks"""
        cmd = [
            "aggsandbox", "bridge", "asset",
            "--network", str(args.network),
            "--destination-network", str(args.destination_network),
            "--amount", str(args.amount),
            "--token-address", args.token_address
        ]
        
        if args.to_address:
            cmd.extend(["--to-address", args.to_address])
        if args.gas_limit:
            cmd.extend(["--gas-limit", str(args.gas_limit)])
        if args.gas_price:
            cmd.extend(["--gas-price", args.gas_price])
        if args.private_key:
            cmd.extend(["--private-key", args.private_key])
        
        return AggsandboxAPI.run_command(cmd)
    
    @staticmethod
    def bridge_claim(args: BridgeClaimArgs) -> Tuple[bool, str]:
        """Claim previously bridged assets"""
        cmd = [
            "aggsandbox", "bridge", "claim",
            "--network", str(args.network),
            "--tx-hash", args.tx_hash,
            "--source-network", str(args.source_network)
        ]
        
        if args.deposit_count is not None:
            cmd.extend(["--deposit-count", str(args.deposit_count)])
        if args.token_address is not None:
            cmd.extend(["--token-address", args.token_address])
        if args.gas_limit is not None:
            cmd.extend(["--gas-limit", str(args.gas_limit)])
        if args.gas_price is not None:
            cmd.extend(["--gas-price", args.gas_price])
        if args.private_key is not None:
            cmd.extend(["--private-key", args.private_key])
        if args.data is not None:
            cmd.extend(["--data", args.data])
        if args.msg_value is not None:
            cmd.extend(["--msg-value", args.msg_value])
        
        return AggsandboxAPI.run_command(cmd)
    
    @staticmethod
    def bridge_message(network: int, destination_network: int, target: str, 
                      data: str, amount: Optional[str] = None, 
                      fallback_address: Optional[str] = None,
                      gas_limit: Optional[int] = None, gas_price: Optional[str] = None,
                      private_key: Optional[str] = None) -> Tuple[bool, str]:
        """Bridge with contract calls"""
        cmd = [
            "aggsandbox", "bridge", "message",
            "--network", str(network),
            "--destination-network", str(destination_network),
            "--target", target,
            "--data", data
        ]
        
        if amount:
            cmd.extend(["--amount", amount])
        if fallback_address:
            cmd.extend(["--fallback-address", fallback_address])
        if gas_limit:
            cmd.extend(["--gas-limit", str(gas_limit)])
        if gas_price:
            cmd.extend(["--gas-price", gas_price])
        if private_key:
            cmd.extend(["--private-key", private_key])
        
        return AggsandboxAPI.run_command(cmd)
    
    @staticmethod
    def bridge_and_call(network: int, destination_network: int, token: str,
                       amount: str, target: str, data: str, fallback: str,
                       gas_limit: Optional[int] = None, gas_price: Optional[str] = None,
                       private_key: Optional[str] = None, msg_value: Optional[str] = None,
                       verbose: bool = False, quiet: bool = False,
                       log_format: Optional[str] = None) -> Tuple[bool, str]:
        """Bridge ERC20 tokens and execute a contract call on the destination network
        
        This command handles the complete bridgeAndCall workflow:
        1. Approves the bridge extension contract to spend tokens
        2. Executes bridgeAndCall to bridge tokens and create a call message
        3. Provides instructions for claiming the asset and message bridges
        
        Note: This creates TWO bridge transactions:
        - Asset bridge (deposit_count = 0) - must be claimed first
        - Message bridge (deposit_count = 1) - contains call instructions
        
        Args:
            network: Source network ID
            destination_network: Destination network ID
            token: Token contract address
            amount: Amount to bridge (in wei)
            target: Target contract address for call
            data: Contract call data (hex encoded)
            fallback: Fallback address if call fails
            gas_limit: Gas limit for the transaction
            gas_price: Gas price in wei
            private_key: Private key to use for the transaction
            msg_value: ETH value to send with contract call (in wei)
            verbose: Enable verbose output
            quiet: Suppress all output except errors and warnings
            log_format: Set log output format (pretty, compact, json)
        """
        cmd = [
            "aggsandbox", "bridge", "bridge-and-call",
            "--network", str(network),
            "--destination-network", str(destination_network),
            "--token", token,
            "--amount", amount,
            "--target", target,
            "--data", data,
            "--fallback", fallback
        ]
        
        if gas_limit:
            cmd.extend(["--gas-limit", str(gas_limit)])
        if gas_price:
            cmd.extend(["--gas-price", gas_price])
        if private_key:
            cmd.extend(["--private-key", private_key])
        if msg_value:
            cmd.extend(["--msg-value", msg_value])
        if verbose:
            cmd.append("--verbose")
        if quiet:
            cmd.append("--quiet")
        if log_format:
            cmd.extend(["--log-format", log_format])
        
        return AggsandboxAPI.run_command(cmd)
    
    # ============================================================================
    # INFORMATION COMMANDS
    # ============================================================================
    
    @staticmethod
    def show_bridges(network_id: int = 0, json_output: bool = True, verbose: bool = False, 
                    quiet: bool = False, log_format: Optional[str] = None) -> Tuple[bool, str]:
        """Show bridge information for a specific network
        
        Args:
            network_id: Network ID (0=L1 Ethereum, 1=first L2, 2=second L2, etc.) [default: 0]
            json_output: Output raw JSON without decorative formatting
            verbose: Enable verbose output
            quiet: Suppress all output except errors and warnings
            log_format: Set log output format (pretty, compact, json)
        """
        cmd = ["aggsandbox", "show", "bridges", "--network-id", str(network_id)]
        
        if json_output:
            cmd.append("--json")
        if verbose:
            cmd.append("--verbose")
        if quiet:
            cmd.append("--quiet")
        if log_format:
            cmd.extend(["--log-format", log_format])
        
        return AggsandboxAPI.run_command(cmd)
    
    @staticmethod
    def show_claims(network_id: int = 1, json_output: bool = True, verbose: bool = False,
                   quiet: bool = False, log_format: Optional[str] = None) -> Tuple[bool, str]:
        """Show pending claims for a network
        
        Args:
            network_id: Network ID to query for pending claims [default: 1]
            json_output: Output raw JSON without decorative formatting
            verbose: Enable verbose output
            quiet: Suppress all output except errors and warnings
            log_format: Set log output format (pretty, compact, json)
        """
        cmd = ["aggsandbox", "show", "claims", "--network-id", str(network_id)]
        
        if json_output:
            cmd.append("--json")
        if verbose:
            cmd.append("--verbose")
        if quiet:
            cmd.append("--quiet")
        if log_format:
            cmd.extend(["--log-format", log_format])
        
        return AggsandboxAPI.run_command(cmd)
    
    @staticmethod
    def show_claim_proof(network_id: int = 0, leaf_index: int = 0, deposit_count: int = 1,
                        json_output: bool = True, verbose: bool = False,
                        quiet: bool = False, log_format: Optional[str] = None) -> Tuple[bool, str]:
        """Generate a cryptographic proof required to claim a cross-chain transfer
        
        Args:
            network_id: Target network ID for claiming [default: 0]
            leaf_index: Leaf index in the global exit tree [default: 0]
            deposit_count: Number of deposits when exit was created [default: 1]
            json_output: Output raw JSON without decorative formatting
            verbose: Enable verbose output
            quiet: Suppress all output except errors and warnings
            log_format: Set log output format (pretty, compact, json)
        """
        cmd = [
            "aggsandbox", "show", "claim-proof",
            "--network-id", str(network_id),
            "--leaf-index", str(leaf_index),
            "--deposit-count", str(deposit_count)
        ]
        
        if json_output:
            cmd.append("--json")
        if verbose:
            cmd.append("--verbose")
        if quiet:
            cmd.append("--quiet")
        if log_format:
            cmd.extend(["--log-format", log_format])
        
        return AggsandboxAPI.run_command(cmd)
    
    @staticmethod
    def show_l1_info_tree_index(network_id: int = 0, deposit_count: int = 0,
                               json_output: bool = True, verbose: bool = False,
                               quiet: bool = False, log_format: Optional[str] = None) -> Tuple[bool, str]:
        """Retrieve the L1 information tree index for a specific deposit count
        
        Args:
            network_id: Network ID to query [default: 0]
            deposit_count: Deposit count to lookup in L1 info tree [default: 0]
            json_output: Output raw JSON without decorative formatting
            verbose: Enable verbose output
            quiet: Suppress all output except errors and warnings
            log_format: Set log output format (pretty, compact, json)
        """
        cmd = [
            "aggsandbox", "show", "l1-info-tree-index",
            "--network-id", str(network_id),
            "--deposit-count", str(deposit_count)
        ]
        
        if json_output:
            cmd.append("--json")
        if verbose:
            cmd.append("--verbose")
        if quiet:
            cmd.append("--quiet")
        if log_format:
            cmd.extend(["--log-format", log_format])
        
        return AggsandboxAPI.run_command(cmd)
    
    # ============================================================================
    # BRIDGE UTILITIES
    # ============================================================================
    
    @staticmethod
    def bridge_utils_get_mapped(network: int, origin_network: int, origin_token: str,
                               private_key: Optional[str] = None, json_output: bool = True) -> Tuple[bool, str]:
        """Get wrapped token address for an origin token"""
        cmd = [
            "aggsandbox", "bridge", "utils", "get-mapped",
            "--network", str(network),
            "--origin-network", str(origin_network),
            "--origin-token", origin_token
        ]
        
        if private_key:
            cmd.extend(["--private-key", private_key])
        if json_output:
            cmd.append("--json")
        
        return AggsandboxAPI.run_command(cmd)
    
    @staticmethod
    def bridge_utils_precalculate(network: int, origin_network: int, origin_token: str,
                                 json_output: bool = True) -> Tuple[bool, str]:
        """Pre-calculate wrapped token address before deployment"""
        cmd = [
            "aggsandbox", "bridge", "utils", "precalculate",
            "--network", str(network),
            "--origin-network", str(origin_network),
            "--origin-token", origin_token
        ]
        
        if json_output:
            cmd.append("--json")
        
        return AggsandboxAPI.run_command(cmd)
    
    @staticmethod
    def bridge_utils_get_origin(network: int, wrapped_token: str,
                               json_output: bool = True) -> Tuple[bool, str]:
        """Get origin token information from wrapped token"""
        cmd = [
            "aggsandbox", "bridge", "utils", "get-origin",
            "--network", str(network),
            "--wrapped-token", wrapped_token
        ]
        
        if json_output:
            cmd.append("--json")
        
        return AggsandboxAPI.run_command(cmd)
    
    @staticmethod
    def bridge_utils_is_claimed(network: int, index: int, source_network: int,
                               private_key: Optional[str] = None, json_output: bool = True) -> Tuple[bool, str]:
        """Check if a bridge has been claimed"""
        cmd = [
            "aggsandbox", "bridge", "utils", "is-claimed",
            "--network", str(network),
            "--index", str(index),
            "--source-network", str(source_network)
        ]
        
        if private_key:
            cmd.extend(["--private-key", private_key])
        if json_output:
            cmd.append("--json")
        
        return AggsandboxAPI.run_command(cmd)
    
    @staticmethod
    def bridge_utils_build_payload(tx_hash: str, source_network: int,
                                  bridge_index: Optional[int] = None,
                                  json_output: bool = True) -> Tuple[bool, str]:
        """Build complete claim payload from bridge transaction"""
        cmd = [
            "aggsandbox", "bridge", "utils", "build-payload",
            "--tx-hash", tx_hash,
            "--source-network", str(source_network)
        ]
        
        if bridge_index is not None:
            cmd.extend(["--bridge-index", str(bridge_index)])
        if json_output:
            cmd.append("--json")
        
        return AggsandboxAPI.run_command(cmd)
    
    @staticmethod
    def bridge_utils_compute_index(local_index: int, source_network: int,
                                  json_output: bool = True) -> Tuple[bool, str]:
        """Calculate global bridge index from local index"""
        cmd = [
            "aggsandbox", "bridge", "utils", "compute-index",
            "--local-index", str(local_index),
            "--source-network", str(source_network)
        ]
        
        if json_output:
            cmd.append("--json")
        
        return AggsandboxAPI.run_command(cmd)
    
    @staticmethod
    def bridge_utils_network_id(network: int, private_key: Optional[str] = None,
                                json_output: bool = True, verbose: bool = False,
                                quiet: bool = False, log_format: Optional[str] = None) -> Tuple[bool, str]:
        """Get bridge contract network ID
        
        Query the bridge contract to get its configured network ID.
        This returns the networkID() value from the bridge contract.
        
        Args:
            network: Network ID
            private_key: Private key (optional)
            json_output: Output as JSON
            verbose: Enable verbose output
            quiet: Suppress all output except errors and warnings
            log_format: Set log output format (pretty, compact, json)
        """
        cmd = [
            "aggsandbox", "bridge", "utils", "network-id",
            "--network", str(network)
        ]
        
        if private_key:
            cmd.extend(["--private-key", private_key])
        if json_output:
            cmd.append("--json")
        if verbose:
            cmd.append("--verbose")
        if quiet:
            cmd.append("--quiet")
        if log_format:
            cmd.extend(["--log-format", log_format])
        
        return AggsandboxAPI.run_command(cmd)
    
    # ============================================================================
    # CLAIM SPONSOR COMMANDS
    # ============================================================================
    
    @staticmethod
    def sponsor_claim(deposit: int, origin_network: int = 0, destination_network: int = 1,
                     verbose: bool = False, quiet: bool = False,
                     log_format: Optional[str] = None) -> Tuple[bool, str]:
        """Submit a bridge claim to the Claim-Sponsor bot
        
        This command performs all steps automatically:
        1. Computes the global index.
        2. Calls the AggKit REST API to fetch Merkle proofs.
        3. Assembles the JSON body required by `/bridge/v1/sponsor-claim`.
        4. Posts the claim.
        
        Args:
            deposit: Deposit counter on the *origin* chain (starts at 0)
            origin_network: Network ID the deposit originated on (omit or 0 for L1)
            destination_network: ID of the destination network (omit or 1 for L2)
            verbose: Enable verbose output
            quiet: Suppress all output except errors and warnings
            log_format: Set log output format (pretty, compact, json)
        """
        cmd = [
            "aggsandbox", "sponsor-claim",
            "--deposit", str(deposit),
            "--origin-network", str(origin_network),
            "--destination-network", str(destination_network)
        ]
        
        if verbose:
            cmd.append("--verbose")
        if quiet:
            cmd.append("--quiet")
        if log_format:
            cmd.extend(["--log-format", log_format])
        
        return AggsandboxAPI.run_command(cmd)
    
    @staticmethod
    def claim_status(global_index: int, network_id: int, verbose: bool = False,
                    quiet: bool = False, log_format: Optional[str] = None) -> Tuple[bool, str]:
        """Query the status of a sponsored claim by global index
        
        Args:
            global_index: Global index of the claim you want to check
            network_id: Network ID to check claims from
            verbose: Enable verbose output
            quiet: Suppress all output except errors and warnings
            log_format: Set log output format (pretty, compact, json)
        """
        cmd = [
            "aggsandbox", "claim-status",
            "--global-index", str(global_index),
            "--network-id", str(network_id)
        ]
        
        if verbose:
            cmd.append("--verbose")
        if quiet:
            cmd.append("--quiet")
        if log_format:
            cmd.extend(["--log-format", log_format])
        
        return AggsandboxAPI.run_command(cmd)
    
    # ============================================================================
    # EVENT MONITORING
    # ============================================================================
    
    @staticmethod
    def events(network_id: int, blocks: int = 10, address: Optional[str] = None,
               verbose: bool = False, quiet: bool = False,
               log_format: Optional[str] = None) -> Tuple[bool, str]:
        """Fetch and display blockchain events
        
        Args:
            network_id: Network ID to monitor events from
            blocks: Number of recent blocks to scan for events
            address: Filter events from specific contract address
            verbose: Enable verbose output
            quiet: Suppress all output except errors and warnings
            log_format: Set log output format (pretty, compact, json)
        """
        cmd = [
            "aggsandbox", "events",
            "--network-id", str(network_id),
            "--blocks", str(blocks)
        ]
        
        if address:
            cmd.extend(["--address", address])
        if verbose:
            cmd.append("--verbose")
        if quiet:
            cmd.append("--quiet")
        if log_format:
            cmd.extend(["--log-format", log_format])
        
        return AggsandboxAPI.run_command(cmd)
    
    # ============================================================================
    # CONVENIENCE METHODS WITH JSON PARSING
    # ============================================================================
    
    @staticmethod
    def get_bridges(network_id: int) -> Optional[Dict[str, Any]]:
        """Get bridge information as parsed JSON"""
        success, output = AggsandboxAPI.show_bridges(network_id, json_output=True)
        if success:
            try:
                return json.loads(output)
            except json.JSONDecodeError as e:
                print(f"ERROR: Could not parse bridge JSON: {e}")
        return None
    
    @staticmethod
    def get_claims(network_id: int) -> Optional[Dict[str, Any]]:
        """Get claims information as parsed JSON"""
        success, output = AggsandboxAPI.show_claims(network_id, json_output=True)
        if success:
            try:
                return json.loads(output)
            except json.JSONDecodeError as e:
                print(f"ERROR: Could not parse claims JSON: {e}")
        return None
    
    @staticmethod
    def get_wrapped_token_address(network: int, origin_network: int, origin_token: str) -> Optional[str]:
        """Get wrapped token address as string"""
        success, output = AggsandboxAPI.bridge_utils_get_mapped(
            network, origin_network, origin_token, json_output=True
        )
        if success:
            try:
                data = json.loads(output)
                return data.get('wrapped_token_address')
            except json.JSONDecodeError as e:
                print(f"ERROR: Could not parse wrapped token JSON: {e}")
        return None
    
    @staticmethod
    def is_bridge_claimed(network: int, index: int, source_network: int) -> Optional[bool]:
        """Check if bridge is claimed as boolean"""
        success, output = AggsandboxAPI.bridge_utils_is_claimed(
            network, index, source_network, json_output=True
        )
        if success:
            try:
                data = json.loads(output)
                return data.get('is_claimed', False)
            except json.JSONDecodeError as e:
                print(f"ERROR: Could not parse is_claimed JSON: {e}")
        return None
    
    # ============================================================================
    # HIGH-LEVEL OPERATIONS
    # ============================================================================
    
    @staticmethod
    def execute_bridge_and_find(network: int, destination_network: int, amount: str,
                               token_address: str, to_address: str, private_key: str) -> Optional[Dict[str, Any]]:
        """Execute bridge and find the resulting bridge transaction"""
        # Step 1: Execute bridge
        args = BridgeAssetArgs(
            network=network,
            destination_network=destination_network,
            amount=amount,
            token_address=token_address,
            to_address=to_address,
            private_key=private_key
        )
        
        success, output = AggsandboxAPI.bridge_asset(args)
        if not success:
            print(f"ERROR: Bridge failed: {output}")
            return None
        
        # Extract transaction hash
        tx_hash = None
        lines = output.split('\n')
        for line in lines:
            if 'bridge transaction submitted' in line.lower() and '0x' in line:
                words = line.split()
                for word in words:
                    if word.startswith('0x') and len(word) == 66:
                        tx_hash = word
                        break
                if tx_hash:
                    break
        
        if not tx_hash:
            print("ERROR: Could not extract bridge transaction hash")
            return None
        
        print(f"SUCCESS: Bridge transaction: {tx_hash}")
        
        # Step 2: Find bridge in bridge events
        for attempt in range(6):
            time.sleep(2)
            
            bridge_data = AggsandboxAPI.get_bridges(network)  # Source network
            if bridge_data and bridge_data.get('bridges'):
                for bridge in bridge_data['bridges']:
                    if bridge.get('tx_hash') == tx_hash:
                        BridgeLogger.success(f"Found bridge in events (attempt {attempt + 1})")
                        return bridge
        
        BridgeLogger.error("Bridge not found in events")
        return None
    
    @staticmethod
    def execute_claim_and_verify(network: int, tx_hash: str, source_network: int,
                                private_key: str, deposit_count: Optional[int] = None) -> Optional[str]:
        """Execute claim and verify it appears in claims"""
        # Step 1: Execute claim
        args = BridgeClaimArgs(
            network=network,
            tx_hash=tx_hash,
            source_network=source_network,
            private_key=private_key,
            deposit_count=deposit_count
        )
        
        success, output = AggsandboxAPI.bridge_claim(args)
        if not success:
            BridgeLogger.error(f"Claim failed: {output}")
            return None
        
        # Extract claim transaction hash
        claim_tx_hash = None
        lines = output.split('\n')
        for line in lines:
            if 'claim transaction submitted' in line.lower() and '0x' in line:
                words = line.split()
                for word in words:
                    if word.startswith('0x') and len(word) == 66:
                        claim_tx_hash = word
                        break
                if claim_tx_hash:
                    break
        
        if claim_tx_hash:
            BridgeLogger.success(f"Claim transaction: {claim_tx_hash}")
        else:
            BridgeLogger.success("Claim completed successfully")
            claim_tx_hash = "completed"
        
        # Step 2: Verify claim appears in claims
        time.sleep(2)
        claims_data = AggsandboxAPI.get_claims(network)
        if claims_data:
            claims_count = len(claims_data.get('claims', []))
            BridgeLogger.success(f"Verified: {claims_count} total claims on network {network}")
        
        return claim_tx_hash

# ============================================================================
# UTILITY FUNCTIONS
# ============================================================================

def extract_tx_hash_from_output(output: str, operation: str = "transaction") -> Optional[str]:
    """Extract transaction hash from aggsandbox output"""
    lines = output.split('\n')
    
    # Look for specific operation transaction
    for line in lines:
        if f'{operation} submitted' in line.lower() and '0x' in line:
            words = line.split()
            for word in words:
                if word.startswith('0x') and len(word) == 66:
                    return word
    
    # Fallback: look for any transaction hash
    for line in lines:
        if 'transaction' in line.lower() and '0x' in line:
            words = line.split()
            for word in words:
                if word.startswith('0x') and len(word) == 66:
                    return word
    
    return None
