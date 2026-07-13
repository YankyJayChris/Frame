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
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const fs = __importStar(require("fs"));
const node_1 = require("vscode-languageclient/node");
const commands_1 = require("./commands");
const status_bar_1 = require("./status-bar");
const decorations_1 = require("./decorations");
const import_manager_1 = require("./import-manager");
const completion_provider_1 = require("./completion-provider");
const icon_picker_1 = require("./icon-picker");
const side_panel_1 = require("./side-panel");
let client;
function findFrameBinary() {
    const configPath = vscode.workspace.getConfiguration('frame').get('path');
    if (configPath) {
        try {
            fs.accessSync(configPath, fs.constants.X_OK);
            return configPath;
        }
        catch { }
    }
    const envPath = process.env.PATH || '';
    const pathDirs = envPath.split(':').filter(Boolean);
    for (const dir of pathDirs) {
        const candidate = `${dir}/frame`;
        try {
            fs.accessSync(candidate, fs.constants.X_OK);
            return candidate;
        }
        catch { }
    }
    const home = process.env.HOME || '';
    const commonLocations = [
        '/usr/local/bin/frame',
        '/opt/homebrew/bin/frame',
        `${home}/.frame/bin/frame`,
        `${home}/.cargo/bin/frame`,
        `${home}/.local/bin/frame`,
    ];
    for (const loc of commonLocations) {
        try {
            fs.accessSync(loc, fs.constants.X_OK);
            return loc;
        }
        catch { }
    }
    return undefined;
}
async function activate(context) {
    const binary = findFrameBinary();
    if (!binary) {
        const action = await vscode.window.showErrorMessage('Frame binary not found. Please install Frame or set "frame.path" in settings.', 'Install Frame', 'Open Settings');
        if (action === 'Install Frame') {
            vscode.env.openExternal(vscode.Uri.parse('https://frame-lang.org/install'));
        }
        else if (action === 'Open Settings') {
            vscode.commands.executeCommand('workbench.action.openSettings', 'frame.path');
        }
        return;
    }
    const serverOptions = {
        command: binary,
        args: ['lsp'],
        options: { env: { ...process.env, RUST_LOG: 'error' } },
    };
    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'frame' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.fr'),
        },
        outputChannel: vscode.window.createOutputChannel('Frame Language Server'),
        traceOutputChannel: vscode.window.createOutputChannel('Frame LSP Trace'),
        progressOnInitialization: true,
        initializationOptions: {
            workspaceRoot: vscode.workspace.getConfiguration('frame').get('workspaceRoot') ||
                (vscode.workspace.workspaceFolders?.[0]?.uri.fsPath),
        },
    };
    client = new node_1.LanguageClient('frame-lsp', 'Frame Language Server', serverOptions, clientOptions);
    client.start();
    context.subscriptions.push(...(0, commands_1.registerCommands)(context, client), ...(0, status_bar_1.registerStatusBar)(context, client), ...(0, decorations_1.registerDecorations)(context), ...(0, import_manager_1.registerImportManager)(context, client), ...(0, completion_provider_1.registerCompletionProviders)(context), ...(0, icon_picker_1.registerIconPicker)(context), ...(0, side_panel_1.registerSidePanel)(context));
}
function deactivate() {
    if (!client)
        return undefined;
    return client.stop();
}
//# sourceMappingURL=extension.js.map