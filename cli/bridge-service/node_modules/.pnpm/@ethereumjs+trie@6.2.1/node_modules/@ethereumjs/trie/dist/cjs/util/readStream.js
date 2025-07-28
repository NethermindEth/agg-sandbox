"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TrieReadStream = void 0;
// eslint-disable-next-line implicit-dependencies/no-implicit
const readable_stream_1 = require("readable-stream");
const index_js_1 = require("../node/index.js");
const nibbles_js_1 = require("./nibbles.js");
const _findValueNodes = async (trie, onFound) => {
    const outerOnFound = async (nodeRef, node, key, walkController) => {
        let fullKey = key;
        if (node instanceof index_js_1.LeafNode) {
            fullKey = key.concat(node.key());
            // found leaf node!
            onFound(nodeRef, node, fullKey, walkController);
        }
        else if (node instanceof index_js_1.BranchNode && node.value()) {
            // found branch with value
            onFound(nodeRef, node, fullKey, walkController);
        }
        else {
            // keep looking for value nodes
            if (node !== null) {
                walkController.allChildren(node, key);
            }
        }
    };
    await trie.walkTrie(trie.root(), outerOnFound);
};
class TrieReadStream extends readable_stream_1.Readable {
    constructor(trie) {
        super({ objectMode: true });
        this.trie = trie;
        this._started = false;
    }
    async _read() {
        if (this._started) {
            return;
        }
        this._started = true;
        try {
            await _findValueNodes(this.trie, async (_, node, key, walkController) => {
                if (node !== null) {
                    this.push({
                        key: (0, nibbles_js_1.nibblestoBytes)(key),
                        value: node.value(),
                    });
                    walkController.allChildren(node, key);
                }
            });
        }
        catch (error) {
            if (error.message === 'Missing node in DB') {
                // pass
            }
            else {
                throw error;
            }
        }
        this.push(null);
    }
}
exports.TrieReadStream = TrieReadStream;
//# sourceMappingURL=readStream.js.map