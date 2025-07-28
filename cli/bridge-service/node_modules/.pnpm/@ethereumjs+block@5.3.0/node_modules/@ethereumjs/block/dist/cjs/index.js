"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __exportStar = (this && this.__exportStar) || function(m, exports) {
    for (var p in m) if (p !== "default" && !Object.prototype.hasOwnProperty.call(exports, p)) __createBinding(exports, m, p);
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.valuesArrayToHeaderData = exports.getDifficulty = exports.BlockHeader = exports.executionPayloadFromBeaconPayload = exports.Block = void 0;
var block_js_1 = require("./block.js");
Object.defineProperty(exports, "Block", { enumerable: true, get: function () { return block_js_1.Block; } });
var from_beacon_payload_js_1 = require("./from-beacon-payload.js");
Object.defineProperty(exports, "executionPayloadFromBeaconPayload", { enumerable: true, get: function () { return from_beacon_payload_js_1.executionPayloadFromBeaconPayload; } });
var header_js_1 = require("./header.js");
Object.defineProperty(exports, "BlockHeader", { enumerable: true, get: function () { return header_js_1.BlockHeader; } });
var helpers_js_1 = require("./helpers.js");
Object.defineProperty(exports, "getDifficulty", { enumerable: true, get: function () { return helpers_js_1.getDifficulty; } });
Object.defineProperty(exports, "valuesArrayToHeaderData", { enumerable: true, get: function () { return helpers_js_1.valuesArrayToHeaderData; } });
__exportStar(require("./types.js"), exports);
//# sourceMappingURL=index.js.map