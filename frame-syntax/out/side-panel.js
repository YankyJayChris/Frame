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
exports.registerSidePanel = registerSidePanel;
const vscode = __importStar(require("vscode"));
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
class FrameExplorerProvider {
    constructor(workspaceRoot) {
        this.workspaceRoot = workspaceRoot;
        this._onDidChangeTreeData = new vscode.EventEmitter();
        this.onDidChangeTreeData = this._onDidChangeTreeData.event;
    }
    refresh() {
        this._onDidChangeTreeData.fire(undefined);
    }
    getTreeItem(element) {
        const treeItem = new vscode.TreeItem(element.label);
        if (element.type === 'page') {
            treeItem.iconPath = new vscode.ThemeIcon('file');
            treeItem.contextValue = 'pageFile';
            treeItem.command = {
                command: 'vscode.open',
                title: 'Open File',
                arguments: [element.filePath ? vscode.Uri.file(element.filePath) : undefined],
            };
        }
        else if (element.type === 'component') {
            treeItem.iconPath = new vscode.ThemeIcon('symbol-class');
            treeItem.contextValue = 'componentFile';
            treeItem.command = {
                command: 'vscode.open',
                title: 'Open File',
                arguments: [element.filePath ? vscode.Uri.file(element.filePath) : undefined],
            };
        }
        else if (element.type === 'store') {
            treeItem.iconPath = new vscode.ThemeIcon('symbol-variable');
            treeItem.contextValue = 'storeFile';
            treeItem.command = {
                command: 'vscode.open',
                title: 'Open File',
                arguments: [element.filePath ? vscode.Uri.file(element.filePath) : undefined],
            };
        }
        else if (element.type === 'function') {
            treeItem.iconPath = new vscode.ThemeIcon('symbol-function');
            treeItem.contextValue = 'functionFile';
            treeItem.command = {
                command: 'vscode.open',
                title: 'Open File',
                arguments: [element.filePath ? vscode.Uri.file(element.filePath) : undefined],
            };
        }
        else if (element.type === 'icon') {
            treeItem.iconPath = new vscode.ThemeIcon('symbol-icon');
        }
        else if (element.type === 'plugin') {
            treeItem.iconPath = new vscode.ThemeIcon('extensions');
        }
        else if (element.type === 'action') {
            treeItem.iconPath = element.iconPath || new vscode.ThemeIcon('play');
            treeItem.command = {
                command: element.command || '',
                title: element.label,
            };
        }
        else {
            treeItem.iconPath = new vscode.ThemeIcon('folder');
            treeItem.collapsibleState = vscode.TreeItemCollapsibleState.Collapsed;
        }
        if (element.children) {
            treeItem.collapsibleState = vscode.TreeItemCollapsibleState.Collapsed;
        }
        treeItem.description = element.description;
        return treeItem;
    }
    async getChildren(element) {
        if (!element) {
            return this.getRootNodes();
        }
        if (element.children) {
            return element.children;
        }
        return [];
    }
    async getRootNodes() {
        const nodes = [];
        const projectNode = {
            label: 'Project',
            type: 'folder',
            children: await this.getProjectFiles(),
        };
        nodes.push(projectNode);
        const iconsNode = {
            label: 'Icons',
            type: 'folder',
            children: this.getIconCategories(),
        };
        nodes.push(iconsNode);
        const actionsNode = {
            label: 'Quick Actions',
            type: 'folder',
            children: this.getQuickActions(),
        };
        nodes.push(actionsNode);
        return nodes;
    }
    async getProjectFiles() {
        const children = [];
        if (!this.workspaceRoot || !fs.existsSync(this.workspaceRoot)) {
            return children;
        }
        const frFiles = await vscode.workspace.findFiles('**/*.fr');
        const pages = [];
        const components = [];
        const stores = [];
        const functions = [];
        const componentRegex = /component\s+([A-Z][a-zA-Z0-9_]*)\s*:/g;
        const pageRegex = /page:\s*\{[^}]*name:\s*"([^"]+)"/g;
        const storeRegex = /:store\s+([A-Z][a-zA-Z0-9_]*)\s*\{/g;
        const fnRegex = /fn\s+([a-z_][a-zA-Z0-9_]*)\s*(?=:)/g;
        for (const file of frFiles) {
            try {
                const content = fs.readFileSync(file.fsPath, 'utf-8');
                const relPath = path.relative(this.workspaceRoot, file.fsPath);
                let match;
                while ((match = pageRegex.exec(content)) !== null) {
                    pages.push({
                        label: match[1],
                        description: relPath,
                        type: 'page',
                        filePath: file.fsPath,
                    });
                }
                while ((match = componentRegex.exec(content)) !== null) {
                    components.push({
                        label: match[1],
                        description: relPath,
                        type: 'component',
                        filePath: file.fsPath,
                    });
                }
                while ((match = storeRegex.exec(content)) !== null) {
                    stores.push({
                        label: match[1],
                        description: relPath,
                        type: 'store',
                        filePath: file.fsPath,
                    });
                }
                while ((match = fnRegex.exec(content)) !== null) {
                    functions.push({
                        label: match[1],
                        description: relPath,
                        type: 'function',
                        filePath: file.fsPath,
                    });
                }
            }
            catch { }
        }
        const pageFolder = {
            label: `Pages (${pages.length})`,
            type: 'folder',
            children: pages,
        };
        children.push(pageFolder);
        const componentFolder = {
            label: `Components (${components.length})`,
            type: 'folder',
            children: components,
        };
        children.push(componentFolder);
        const storeFolder = {
            label: `Stores (${stores.length})`,
            type: 'folder',
            children: stores,
        };
        children.push(storeFolder);
        const functionFolder = {
            label: `Functions (${functions.length})`,
            type: 'folder',
            children: functions,
        };
        children.push(functionFolder);
        return children;
    }
    getIconCategories() {
        const categories = new Map();
        const icons = [
            { name: 'Actions', icons: ['plus', 'minus', 'checkmark', 'xmark', 'trash', 'pencil', 'ellipsis', 'link'] },
            { name: 'Navigation', icons: ['house', 'line.3.horizontal', 'list.bullet', 'square.grid.2x2'] },
            { name: 'Communication', icons: ['envelope', 'phone', 'message', 'bubble.left'] },
            { name: 'Media', icons: ['play', 'pause', 'stop', 'camera', 'photo', 'mic', 'video'] },
            { name: 'Status', icons: ['cloud', 'wifi', 'battery.100', 'info', 'questionmark', 'exclamationmark'] },
            { name: 'Arrows', icons: ['arrow.up', 'arrow.down', 'arrow.left', 'arrow.right', 'chevron.up', 'chevron.down'] },
        ];
        return icons.map(cat => ({
            label: cat.name,
            type: 'folder',
            children: cat.icons.map(iconName => ({
                label: iconName,
                type: 'icon',
            })),
        }));
    }
    getQuickActions() {
        return [
            { label: 'Build Project', type: 'action', command: 'frame.build', iconPath: new vscode.ThemeIcon('tools') },
            { label: 'Build & Watch', type: 'action', command: 'frame.buildWatch', iconPath: new vscode.ThemeIcon('watch') },
            { label: 'Run Tests', type: 'action', command: 'frame.test', iconPath: new vscode.ThemeIcon('beaker') },
            { label: 'Lint Project', type: 'action', command: 'frame.lint', iconPath: new vscode.ThemeIcon('checklist') },
            { label: 'Add Plugin', type: 'action', command: 'frame.pluginAdd', iconPath: new vscode.ThemeIcon('extensions') },
            { label: 'Deploy iOS', type: 'action', command: 'frame.deploy', iconPath: new vscode.ThemeIcon('device-mobile') },
            { label: 'New Page', type: 'action', command: 'frame.newPage', iconPath: new vscode.ThemeIcon('new-file') },
            { label: 'New Component', type: 'action', command: 'frame.newComponent', iconPath: new vscode.ThemeIcon('new-file') },
            { label: 'New Store', type: 'action', command: 'frame.newStore', iconPath: new vscode.ThemeIcon('new-file') },
            { label: 'Browse Icons', type: 'action', command: 'frame.showIcons', iconPath: new vscode.ThemeIcon('symbol-icon') },
            { label: 'Documentation', type: 'action', command: 'frame.openDocs', iconPath: new vscode.ThemeIcon('book') },
        ];
    }
}
function registerSidePanel(context) {
    const disposables = [];
    const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || '';
    const provider = new FrameExplorerProvider(workspaceRoot);
    disposables.push(vscode.window.registerTreeDataProvider('frameExplorer', provider));
    disposables.push(vscode.commands.registerCommand('frameExplorer.refresh', () => provider.refresh()));
    if (workspaceRoot) {
        const watcher = vscode.workspace.createFileSystemWatcher('**/*.fr');
        watcher.onDidChange(() => provider.refresh());
        watcher.onDidCreate(() => provider.refresh());
        watcher.onDidDelete(() => provider.refresh());
        disposables.push(watcher);
    }
    return disposables;
}
//# sourceMappingURL=side-panel.js.map