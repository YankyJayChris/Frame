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
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.registerFileDecorations = registerFileDecorations;
const vscode = __importStar(require("vscode"));
/**
 * Adds a subtle green color tint to all .fr files in the explorer,
 * reinforcing the Frame brand color without per-type badges.
 */
class FrameFileDecorationProvider {
    constructor() {
        this._onChange = new vscode.EventEmitter();
        this.onDidChangeFileDecorations = this._onChange.event;
    }
    provideFileDecoration(uri) {
        if (!uri.fsPath.endsWith(".fr"))
            return undefined;
        return {
            color: new vscode.ThemeColor("charts.green"),
            tooltip: "Frame source file",
            propagate: false,
        };
    }
    refresh() {
        this._onChange.fire(undefined);
    }
}
function registerFileDecorations(context) {
    const provider = new FrameFileDecorationProvider();
    const disposables = [];
    disposables.push(vscode.window.registerFileDecorationProvider(provider));
    const watcher = vscode.workspace.createFileSystemWatcher("**/*.fr");
    watcher.onDidCreate(() => provider.refresh());
    watcher.onDidDelete(() => provider.refresh());
    disposables.push(watcher);
    return disposables;
}
//# sourceMappingURL=file-decorations.js.map