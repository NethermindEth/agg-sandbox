import { Lock, ValueEncoding } from '@ethereumjs/util';
import { CheckpointDB } from './db/index.js';
import { TrieReadStream as ReadStream } from './util/readStream.js';
import type { EmbeddedNode, FoundNodeFunction, Nibbles, Path, Proof, TrieNode, TrieOpts, TrieOptsWithDefaults, TrieShallowCopyOpts } from './types.js';
import type { OnFound } from './util/asyncWalk.js';
import type { BatchDBOp, DB } from '@ethereumjs/util';
import type { Debugger } from 'debug';
/**
 * The basic trie interface, use with `import { Trie } from '@ethereumjs/trie'`.
 */
export declare class Trie {
    protected readonly _opts: TrieOptsWithDefaults;
    /** The root for an empty trie */
    EMPTY_TRIE_ROOT: Uint8Array;
    /** The backend DB */
    protected _db: CheckpointDB;
    protected _hashLen: number;
    protected _lock: Lock;
    protected _root: Uint8Array;
    /** Debug logging */
    protected DEBUG: boolean;
    protected _debug: Debugger;
    protected debug: (...args: any) => void;
    /**
     * Creates a new trie.
     * @param opts Options for instantiating the trie
     *
     * Note: in most cases, the static {@link Trie.create} constructor should be used.  It uses the same API but provides sensible defaults
     */
    constructor(opts?: TrieOpts);
    /**
     * Create a trie from a given (EIP-1186)[https://eips.ethereum.org/EIPS/eip-1186] proof. A proof contains the encoded trie nodes
     * from the root node to the leaf node storing state data.
     * @param proof an EIP-1186 proof to create trie from
     * @param shouldVerifyRoot If `true`, verifies that the root key of the proof matches the trie root. Throws if this is not the case.
     * @param trieOpts trie opts to be applied to returned trie
     * @returns new trie created from given proof
     */
    static createFromProof(proof: Proof, trieOpts?: TrieOpts, shouldVerifyRoot?: boolean): Promise<Trie>;
    /**
     * Static version of verifyProof function with the same behavior. An (EIP-1186)[https://eips.ethereum.org/EIPS/eip-1186] proof contains the encoded trie nodes
     * from the root node to the leaf node storing state data.
     * @param rootHash Root hash of the trie that this proof was created from and is being verified for
     * @param key Key that is being verified and that the proof is created for
     * @param proof An (EIP-1186)[https://eips.ethereum.org/EIPS/eip-1186] proof contains the encoded trie nodes from the root node to the leaf node storing state data.
     * @param opts optional, the opts may include a custom hashing function to use with the trie for proof verification
     * @throws If proof is found to be invalid.
     * @returns The value from the key, or null if valid proof of non-existence.
     */
    static verifyProof(key: Uint8Array, proof: Proof, opts?: TrieOpts): Promise<Uint8Array | null>;
    /**
     * A range proof is a proof that includes the encoded trie nodes from the root node to leaf node for one or more branches of a trie,
     * allowing an entire range of leaf nodes to be validated. This is useful in applications such as snap sync where contiguous ranges
     * of state trie data is received and validated for constructing world state, locally. Also see {@link verifyRangeProof}. A static
     * version of this function also exists.
     * @param rootHash - root hash of state trie this proof is being verified against.
     * @param firstKey - first key of range being proven.
     * @param lastKey - last key of range being proven.
     * @param keys - key list of leaf data being proven.
     * @param values - value list of leaf data being proven, one-to-one correspondence with keys.
     * @param proof - proof node list, if all-elements-proof where no proof is needed, proof should be null, and both `firstKey` and `lastKey` must be null as well
     * @param opts - optional, the opts may include a custom hashing function to use with the trie for proof verification
     * @returns a flag to indicate whether there exists more trie node in the trie
     */
    static verifyRangeProof(rootHash: Uint8Array, firstKey: Uint8Array | null, lastKey: Uint8Array | null, keys: Uint8Array[], values: Uint8Array[], proof: Uint8Array[] | null, opts?: TrieOpts): Promise<boolean>;
    /**
     * Static version of fromProof function. If a root is provided in the opts param, the proof will be checked to have the same expected root. An
     * (EIP-1186)[https://eips.ethereum.org/EIPS/eip-1186] proof contains the encoded trie nodes from the root node to the leaf node storing state data.
     * @param proof An (EIP-1186)[https://eips.ethereum.org/EIPS/eip-1186] proof contains the encoded trie nodes from the root node to the leaf node storing state data.
     * @deprecated Use `createFromProof`
     */
    static fromProof(proof: Proof, opts?: TrieOpts): Promise<Trie>;
    /**
     * A range proof is a proof that includes the encoded trie nodes from the root node to leaf node for one or more branches of a trie,
     * allowing an entire range of leaf nodes to be validated. This is useful in applications such as snap sync where contiguous ranges
     * of state trie data is received and validated for constructing world state, locally. Also see {@link verifyRangeProof}. A static
     * version of this function also exists.
     * @param rootHash - root hash of state trie this proof is being verified against.
     * @param firstKey - first key of range being proven.
     * @param lastKey - last key of range being proven.
     * @param keys - key list of leaf data being proven.
     * @param values - value list of leaf data being proven, one-to-one correspondence with keys.
     * @param proof - proof node list, if all-elements-proof where no proof is needed, proof should be null, and both `firstKey` and `lastKey` must be null as well
     * @returns a flag to indicate whether there exists more trie node in the trie
     */
    verifyRangeProof(rootHash: Uint8Array, firstKey: Uint8Array | null, lastKey: Uint8Array | null, keys: Uint8Array[], values: Uint8Array[], proof: Uint8Array[] | null): Promise<boolean>;
    /**
     * Creates a proof from a trie and key that can be verified using {@link Trie.verifyProof}. An (EIP-1186)[https://eips.ethereum.org/EIPS/eip-1186] proof contains
     * the encoded trie nodes from the root node to the leaf node storing state data. The returned proof will be in the format of an array that contains Uint8Arrays of
     * serialized branch, extension, and/or leaf nodes.
     * @param key key to create a proof for
     */
    createProof(key: Uint8Array): Promise<Proof>;
    /**
     * Updates a trie from a proof by putting all the nodes in the proof into the trie. If a trie is being updated with multiple proofs, {@param shouldVerifyRoot} can
     * be passed as false in order to not immediately throw on an unexpected root, so that root verification can happen after all proofs and their nodes have been added.
     * An (EIP-1186)[https://eips.ethereum.org/EIPS/eip-1186] proof contains the encoded trie nodes from the root node to the leaf node storing state data.
     * @param proof An (EIP-1186)[https://eips.ethereum.org/EIPS/eip-1186] proof to update the trie from.
     * @param shouldVerifyRoot If `true`, verifies that the root key of the proof matches the trie root. Throws if this is not the case.
     * @returns The root of the proof
     */
    updateFromProof(proof: Proof, shouldVerifyRoot?: boolean): Promise<Uint8Array | undefined>;
    /**
     * Verifies a proof by putting all of its nodes into a trie and attempting to get the proven key. An (EIP-1186)[https://eips.ethereum.org/EIPS/eip-1186] proof
     * contains the encoded trie nodes from the root node to the leaf node storing state data. A static version of this function exists with the same name.
     * @param rootHash Root hash of the trie that this proof was created from and is being verified for
     * @param key Key that is being verified and that the proof is created for
     * @param proof an EIP-1186 proof to verify the key against
     * @throws If proof is found to be invalid.
     * @returns The value from the key, or null if valid proof of non-existence.
     */
    verifyProof(rootHash: Uint8Array, key: Uint8Array, proof: Proof): Promise<Uint8Array | null>;
    /**
     * Create a trie from a given (EIP-1186)[https://eips.ethereum.org/EIPS/eip-1186] proof. An EIP-1186 proof contains the encoded trie nodes from the root
     * node to the leaf node storing state data. This function does not check if the proof has the same expected root. A static version of this function exists
     * with the same name.
     * @param proof an EIP-1186 proof to update the trie from
     * @deprecated Use `updateFromProof`
     */
    fromProof(proof: Proof): Promise<void>;
    static create(opts?: TrieOpts): Promise<Trie>;
    database(db?: DB<string, string | Uint8Array>, valueEncoding?: ValueEncoding): CheckpointDB;
    /**
     * Gets and/or Sets the current root of the `trie`
     */
    root(value?: Uint8Array | null): Uint8Array;
    /**
     * Checks if a given root exists.
     */
    checkRoot(root: Uint8Array): Promise<boolean>;
    /**
     * Gets a value given a `key`
     * @param key - the key to search for
     * @param throwIfMissing - if true, throws if any nodes are missing. Used for verifying proofs. (default: false)
     * @returns A Promise that resolves to `Uint8Array` if a value was found or `null` if no value was found.
     */
    get(key: Uint8Array, throwIfMissing?: boolean): Promise<Uint8Array | null>;
    /**
     * Stores a given `value` at the given `key` or do a delete if `value` is empty
     * (delete operations are only executed on DB with `deleteFromDB` set to `true`)
     * @param key
     * @param value
     * @returns A Promise that resolves once value is stored.
     */
    put(key: Uint8Array, value: Uint8Array | null, skipKeyTransform?: boolean): Promise<void>;
    /**
     * Deletes a value given a `key` from the trie
     * (delete operations are only executed on DB with `deleteFromDB` set to `true`)
     * @param key
     * @returns A Promise that resolves once value is deleted.
     */
    del(key: Uint8Array, skipKeyTransform?: boolean): Promise<void>;
    /**
     * Tries to find a path to the node for the given key.
     * It returns a `stack` of nodes to the closest node.
     * @param key - the search key
     * @param throwIfMissing - if true, throws if any nodes are missing. Used for verifying proofs. (default: false)
     */
    findPath(key: Uint8Array, throwIfMissing?: boolean, partialPath?: {
        stack: TrieNode[];
    }): Promise<Path>;
    /**
     * Walks a trie until finished.
     * @param root
     * @param onFound - callback to call when a node is found. This schedules new tasks. If no tasks are available, the Promise resolves.
     * @returns Resolves when finished walking trie.
     */
    walkTrie(root: Uint8Array, onFound: FoundNodeFunction): Promise<void>;
    walkTrieIterable: (nodeHash: Uint8Array, currentKey?: number[] | undefined, onFound?: OnFound | undefined, filter?: import("./util/asyncWalk.js").NodeFilter | undefined, visited?: Set<string> | undefined) => AsyncIterable<{
        node: TrieNode;
        currentKey: number[];
    }>;
    /**
     * Executes a callback for each node in the trie.
     * @param onFound - callback to call when a node is found.
     * @returns Resolves when finished walking trie.
     */
    walkAllNodes(onFound: OnFound): Promise<void>;
    /**
     * Executes a callback for each value node in the trie.
     * @param onFound - callback to call when a node is found.
     * @returns Resolves when finished walking trie.
     */
    walkAllValueNodes(onFound: OnFound): Promise<void>;
    /**
     * Creates the initial node from an empty tree.
     * @private
     */
    protected _createInitialNode(key: Uint8Array, value: Uint8Array): Promise<void>;
    /**
     * Retrieves a node from db by hash.
     */
    lookupNode(node: Uint8Array | Uint8Array[]): Promise<TrieNode>;
    /**
     * Updates a node.
     * @private
     * @param key
     * @param value
     * @param keyRemainder
     * @param stack
     */
    protected _updateNode(k: Uint8Array, value: Uint8Array, keyRemainder: Nibbles, stack: TrieNode[]): Promise<void>;
    /**
     * Deletes a node from the trie.
     * @private
     */
    protected _deleteNode(k: Uint8Array, stack: TrieNode[]): Promise<void>;
    /**
     * Saves a stack of nodes to the database.
     *
     * @param key - the key. Should follow the stack
     * @param stack - a stack of nodes to the value given by the key
     * @param opStack - a stack of levelup operations to commit at the end of this function
     */
    saveStack(key: Nibbles, stack: TrieNode[], opStack: BatchDBOp[]): Promise<void>;
    /**
     * Formats node to be saved by `levelup.batch`.
     * @private
     * @param node - the node to format.
     * @param topLevel - if the node is at the top level.
     * @param opStack - the opStack to push the node's data.
     * @param remove - whether to remove the node
     * @returns The node's hash used as the key or the rawNode.
     */
    _formatNode(node: TrieNode, topLevel: boolean, opStack: BatchDBOp[], remove?: boolean): Uint8Array | (EmbeddedNode | null)[];
    /**
     * The given hash of operations (key additions or deletions) are executed on the trie
     * (delete operations are only executed on DB with `deleteFromDB` set to `true`)
     * @example
     * const ops = [
     *    { type: 'del', key: Uint8Array.from('father') }
     *  , { type: 'put', key: Uint8Array.from('name'), value: Uint8Array.from('Yuri Irsenovich Kim') }
     *  , { type: 'put', key: Uint8Array.from('dob'), value: Uint8Array.from('16 February 1941') }
     *  , { type: 'put', key: Uint8Array.from('spouse'), value: Uint8Array.from('Kim Young-sook') }
     *  , { type: 'put', key: Uint8Array.from('occupation'), value: Uint8Array.from('Clown') }
     * ]
     * await trie.batch(ops)
     * @param ops
     */
    batch(ops: BatchDBOp[], skipKeyTransform?: boolean): Promise<void>;
    verifyPrunedIntegrity(): Promise<boolean>;
    /**
     * The `data` event is given an `Object` that has two properties; the `key` and the `value`. Both should be Uint8Arrays.
     * @return Returns a [stream](https://nodejs.org/dist/latest-v12.x/docs/api/stream.html#stream_class_stream_readable) of the contents of the `trie`
     */
    createReadStream(): ReadStream;
    /**
     * Returns a copy of the underlying trie.
     *
     * Note on db: the copy will create a reference to the
     * same underlying database.
     *
     * Note on cache: for memory reasons a copy will by default
     * not recreate a new LRU cache but initialize with cache
     * being deactivated. This behavior can be overwritten by
     * explicitly setting `cacheSize` as an option on the method.
     *
     * @param includeCheckpoints - If true and during a checkpoint, the copy will contain the checkpointing metadata and will use the same scratch as underlying db.
     */
    shallowCopy(includeCheckpoints?: boolean, opts?: TrieShallowCopyOpts): Trie;
    /**
     * Persists the root hash in the underlying database
     */
    persistRoot(): Promise<void>;
    /**
     * Finds all nodes that are stored directly in the db
     * (some nodes are stored raw inside other nodes)
     * called by {@link ScratchReadStream}
     * @private
     */
    protected _findDbNodes(onFound: FoundNodeFunction): Promise<void>;
    /**
     * Returns the key practically applied for trie construction
     * depending on the `useKeyHashing` option being set or not.
     * @param key
     */
    protected appliedKey(key: Uint8Array): Uint8Array;
    protected hash(msg: Uint8Array): Uint8Array;
    /**
     * Is the trie during a checkpoint phase?
     */
    hasCheckpoints(): boolean;
    /**
     * Creates a checkpoint that can later be reverted to or committed.
     * After this is called, all changes can be reverted until `commit` is called.
     */
    checkpoint(): void;
    /**
     * Commits a checkpoint to disk, if current checkpoint is not nested.
     * If nested, only sets the parent checkpoint as current checkpoint.
     * @throws If not during a checkpoint phase
     */
    commit(): Promise<void>;
    /**
     * Reverts the trie to the state it was at when `checkpoint` was first called.
     * If during a nested checkpoint, sets root to most recent checkpoint, and sets
     * parent checkpoint as current.
     */
    revert(): Promise<void>;
    /**
     * Flushes all checkpoints, restoring the initial checkpoint state.
     */
    flushCheckpoints(): void;
}
//# sourceMappingURL=trie.d.ts.map