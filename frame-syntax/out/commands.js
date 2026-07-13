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
exports.registerCommands = registerCommands;
const vscode = __importStar(require("vscode"));
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
function getFrameBinary() {
    return vscode.workspace.getConfiguration('frame').get('path') || 'frame';
}
function getWorkspaceRoot() {
    const configured = vscode.workspace.getConfiguration('frame').get('workspaceRoot');
    if (configured)
        return configured;
    return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || '';
}
function runInTerminal(args) {
    const terminal = vscode.window.createTerminal({
        name: 'Frame',
        shellPath: getFrameBinary(),
        shellArgs: args,
        iconPath: new vscode.ThemeIcon('symbol-color'),
    });
    terminal.show();
}
async function scaffoldFile(name, template, relativeDir) {
    const root = getWorkspaceRoot();
    if (!root) {
        vscode.window.showErrorMessage('No workspace folder open.');
        return;
    }
    const targetDir = path.join(root, relativeDir);
    if (!fs.existsSync(targetDir)) {
        fs.mkdirSync(targetDir, { recursive: true });
    }
    const filePath = path.join(targetDir, `${name}.fr`);
    if (fs.existsSync(filePath)) {
        vscode.window.showErrorMessage(`File ${name}.fr already exists in ${relativeDir}.`);
        return;
    }
    fs.writeFileSync(filePath, template, 'utf-8');
    const doc = await vscode.workspace.openTextDocument(filePath);
    vscode.window.showTextDocument(doc);
}
function registerCommands(context, client) {
    const disposables = [];
    disposables.push(vscode.commands.registerCommand('frame.build', () => {
        runInTerminal(['build']);
    }));
    disposables.push(vscode.commands.registerCommand('frame.buildWatch', () => {
        runInTerminal(['build', '--watch']);
    }));
    disposables.push(vscode.commands.registerCommand('frame.test', () => {
        runInTerminal(['test']);
    }));
    disposables.push(vscode.commands.registerCommand('frame.testFilter', async () => {
        const tests = await client.sendRequest('frame/listTests');
        const items = tests.map(t => ({ label: t }));
        const selected = await vscode.window.showQuickPick(items, {
            placeHolder: 'Select a test to run',
        });
        if (selected) {
            runInTerminal(['test', '--filter', selected.label]);
        }
    }));
    disposables.push(vscode.commands.registerCommand('frame.deploy', async () => {
        const target = await vscode.window.showQuickPick([
            { label: 'iOS', description: 'Deploy to iOS simulator/device' },
            { label: 'Android', description: 'Deploy to Android emulator/device' },
        ], { placeHolder: 'Select deployment target' });
        if (target) {
            runInTerminal(['deploy', target.label.toLowerCase()]);
        }
    }));
    disposables.push(vscode.commands.registerCommand('frame.lint', () => {
        runInTerminal(['lint']);
    }));
    disposables.push(vscode.commands.registerCommand('frame.lintFile', () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showWarningMessage('No active editor.');
            return;
        }
        const filePath = editor.document.uri.fsPath;
        runInTerminal(['lint', filePath]);
    }));
    disposables.push(vscode.commands.registerCommand('frame.pluginAdd', async () => {
        const name = await vscode.window.showInputBox({
            placeHolder: 'plugin-name',
            prompt: 'Enter the plugin name to add',
            validateInput: (value) => value.trim() ? null : 'Plugin name is required',
        });
        if (name) {
            runInTerminal(['plugin', 'add', name.trim()]);
        }
    }));
    disposables.push(vscode.commands.registerCommand('frame.pluginList', async () => {
        try {
            const plugins = await client.sendRequest('frame/listPlugins');
            const items = plugins.map(p => ({
                label: p,
                iconPath: new vscode.ThemeIcon('extensions'),
            }));
            if (items.length === 0) {
                vscode.window.showInformationMessage('No plugins installed.');
                return;
            }
            await vscode.window.showQuickPick(items, {
                placeHolder: 'Installed plugins',
                matchOnDescription: true,
            });
        }
        catch {
            vscode.window.showErrorMessage('Failed to fetch plugin list from LSP.');
        }
    }));
    disposables.push(vscode.commands.registerCommand('frame.iconList', async () => {
        try {
            const icons = await client.sendRequest('frame/listIcons');
            const items = icons.map(i => ({
                label: i,
                iconPath: new vscode.ThemeIcon('symbol-icon'),
            }));
            if (items.length === 0) {
                vscode.window.showInformationMessage('No icons registered.');
                return;
            }
            const selected = await vscode.window.showQuickPick(items, {
                placeHolder: 'Registered icons',
                matchOnDescription: true,
            });
            if (selected && vscode.window.activeTextEditor) {
                vscode.window.activeTextEditor.edit(edit => {
                    const pos = vscode.window.activeTextEditor.selection.active;
                    edit.insert(pos, selected.label);
                });
            }
        }
        catch {
            vscode.window.showErrorMessage('Failed to fetch icon list from LSP.');
        }
    }));
    disposables.push(vscode.commands.registerCommand('frame.iconGenerate', async () => {
        const target = await vscode.window.showQuickPick([
            { label: 'iOS', description: 'Generate iOS icons' },
            { label: 'Android', description: 'Generate Android icons' },
            { label: 'All', description: 'Generate icons for all platforms' },
        ], { placeHolder: 'Select platform' });
        if (target) {
            const arg = target.label.toLowerCase() === 'all' ? '--all' : `--${target.label.toLowerCase()}`;
            runInTerminal(['icons', 'generate', arg]);
        }
    }));
    disposables.push(vscode.commands.registerCommand('frame.newPage', async () => {
        const name = await vscode.window.showInputBox({
            placeHolder: 'HomePage',
            prompt: 'Enter the page name (PascalCase)',
            validateInput: (value) => /^[A-Z][a-zA-Z0-9]*$/.test(value) ? null : 'Must be PascalCase (e.g. HomePage)',
        });
        if (!name)
            return;
        const routeName = name
            .replace(/([A-Z])/g, '-$1')
            .toLowerCase()
            .replace(/^-/, '/');
        const template = `page: {
  name: "${name}"
  route: "${routeName}"
  styles: { width: 100%  height: 100% }
  children: [
    scaffold: {
      styles: { safe_area: true }
      children: [
        $0
      ]
    }
  ]
}
`;
        await scaffoldFile(name, template, 'src/pages');
    }));
    disposables.push(vscode.commands.registerCommand('frame.newComponent', async () => {
        const name = await vscode.window.showInputBox({
            placeHolder: 'MyComponent',
            prompt: 'Enter the component name (PascalCase)',
            validateInput: (value) => /^[A-Z][a-zA-Z0-9]*$/.test(value) ? null : 'Must be PascalCase (e.g. MyComponent)',
        });
        if (!name)
            return;
        const template = `component ${name}: {
  props: {
    style: object = {}
  }
  styles: { padding: 8 }
  children: [
    text: {
      content: "${name}"
      styles: { font_size: 16sp }
    }
  ]
}
`;
        await scaffoldFile(name, template, 'src/components');
    }));
    disposables.push(vscode.commands.registerCommand('frame.newStore', async () => {
        const name = await vscode.window.showInputBox({
            placeHolder: 'CounterStore',
            prompt: 'Enter the store name (PascalCase)',
            validateInput: (value) => /^[A-Z][a-zA-Z0-9]*$/.test(value) ? null : 'Must be PascalCase (e.g. CounterStore)',
        });
        if (!name)
            return;
        const template = `:store ${name} {
  :var mut count: int = 0

  fn increment: async () => {
    count = count + 1
  }

  fn decrement: async () => {
    count = count - 1
  }
}
`;
        await scaffoldFile(name, template, 'src/stores');
    }));
    disposables.push(vscode.commands.registerCommand('frame.openDocs', () => {
        vscode.env.openExternal(vscode.Uri.parse('https://frame-lang.org/docs'));
    }));
    disposables.push(vscode.commands.registerCommand('frame.restartLsp', async () => {
        vscode.window.showInformationMessage('Restarting Frame Language Server...');
        await client.restart();
        vscode.window.showInformationMessage('Frame Language Server restarted.');
    }));
    return disposables;
}
//# sourceMappingURL=commands.js.map