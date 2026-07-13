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
exports.ImportManager = void 0;
exports.registerImportManager = registerImportManager;
const vscode = __importStar(require("vscode"));
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
class ImportManager {
    constructor(client) {
        this.client = client;
        this.cache = new Map();
        this.cacheTimestamp = 0;
        this.cacheTTL = 30000;
    }
    async scanWorkspaceComponents() {
        const now = Date.now();
        if (this.cache.size > 0 && now - this.cacheTimestamp < this.cacheTTL) {
            return Array.from(this.cache.values()).flat();
        }
        this.cache.clear();
        const components = [];
        const files = await vscode.workspace.findFiles('**/*.fr');
        const componentRegex = /component\s+([A-Z][a-zA-Z0-9_]*)\s*:/g;
        for (const file of files) {
            try {
                const content = fs.readFileSync(file.fsPath, 'utf-8');
                let match;
                while ((match = componentRegex.exec(content)) !== null) {
                    const entry = { name: match[1], filePath: file.fsPath };
                    components.push(entry);
                    const dir = path.dirname(file.fsPath);
                    if (!this.cache.has(dir)) {
                        this.cache.set(dir, []);
                    }
                    this.cache.get(dir).push(entry);
                }
            }
            catch { }
        }
        this.cacheTimestamp = now;
        return components;
    }
    findUnimportedComponent(document, range, components) {
        const text = document.getText(range);
        const name = text.trim();
        if (!/^[A-Z][a-zA-Z0-9_]*$/.test(name))
            return undefined;
        const imports = this.getImportedNames(document);
        if (imports.has(name))
            return undefined;
        const docDir = path.dirname(document.uri.fsPath);
        const defined = this.getDefinedNamesInDocument(document);
        if (defined.has(name))
            return undefined;
        return components.find(c => c.name === name && c.filePath !== document.uri.fsPath);
    }
    getImportedNames(document) {
        const names = new Set();
        const importRegex = /import\s*\{([^}]+)\}/g;
        let match;
        while ((match = importRegex.exec(document.getText())) !== null) {
            match[1].split(',').forEach(n => {
                const trimmed = n.trim();
                if (trimmed)
                    names.add(trimmed);
            });
        }
        return names;
    }
    getDefinedNamesInDocument(document) {
        const names = new Set();
        const text = document.getText();
        const defRegex = /(?:component|page|:store)\s+([A-Z][a-zA-Z0-9_]*)\s*[:{]/g;
        let match;
        while ((match = defRegex.exec(text)) !== null) {
            names.add(match[1]);
        }
        return names;
    }
    isInChildrenBlock(document, position) {
        const text = document.getText();
        const offset = document.offsetAt(position);
        const beforeText = text.substring(0, offset);
        const lastChildrenOpen = beforeText.lastIndexOf('children: [');
        if (lastChildrenOpen === -1)
            return false;
        const afterChildren = text.substring(lastChildrenOpen);
        let depth = 0;
        for (let i = 0; i < afterChildren.length; i++) {
            const ch = afterChildren[i];
            if (ch === '[')
                depth++;
            else if (ch === ']')
                depth--;
            if (depth === 0 && i < offset - lastChildrenOpen)
                return false;
            if (depth < 0)
                return false;
        }
        return depth > 0;
    }
    async provideCodeActions(document, range, _context, _token) {
        if (!this.isInChildrenBlock(document, range.start))
            return undefined;
        const components = await this.scanWorkspaceComponents();
        const component = this.findUnimportedComponent(document, range, components);
        if (!component)
            return undefined;
        const documentDir = path.dirname(document.uri.fsPath);
        let relativePath = path.relative(documentDir, component.filePath);
        if (!relativePath.startsWith('.')) {
            relativePath = `./${relativePath}`;
        }
        relativePath = relativePath.replace(/\.fr$/, '');
        const action = new vscode.CodeAction(`Import '${component.name}' from '${relativePath}'`, vscode.CodeActionKind.QuickFix);
        action.edit = new vscode.WorkspaceEdit();
        const importStatement = `import { ${component.name} } "${relativePath}"\n`;
        const firstLine = document.lineAt(0);
        action.edit.insert(document.uri, new vscode.Position(0, 0), importStatement);
        return [action];
    }
}
exports.ImportManager = ImportManager;
function registerImportManager(context, client) {
    const disposables = [];
    const provider = new ImportManager(client);
    disposables.push(vscode.languages.registerCodeActionsProvider('frame', provider, {
        providedCodeActionKinds: [vscode.CodeActionKind.QuickFix],
    }));
    return disposables;
}
//# sourceMappingURL=import-manager.js.map