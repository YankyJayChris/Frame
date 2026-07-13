import * as vscode from 'vscode';
import { LanguageClient } from 'vscode-languageclient/node';
import * as fs from 'fs';
import * as path from 'path';

interface ComponentEntry {
  name: string;
  filePath: string;
}

export class ImportManager implements vscode.CodeActionProvider {
  private cache: Map<string, ComponentEntry[]> = new Map();
  private cacheTimestamp: number = 0;
  private readonly cacheTTL: number = 30000;

  constructor(private client: LanguageClient) {}

  private async scanWorkspaceComponents(): Promise<ComponentEntry[]> {
    const now = Date.now();
    if (this.cache.size > 0 && now - this.cacheTimestamp < this.cacheTTL) {
      return Array.from(this.cache.values()).flat();
    }

    this.cache.clear();
    const components: ComponentEntry[] = [];

    const files = await vscode.workspace.findFiles('**/*.fr');
    const componentRegex = /component\s+([A-Z][a-zA-Z0-9_]*)\s*:/g;

    for (const file of files) {
      try {
        const content = fs.readFileSync(file.fsPath, 'utf-8');
        let match: RegExpExecArray | null;
        while ((match = componentRegex.exec(content)) !== null) {
          const entry = { name: match[1], filePath: file.fsPath };
          components.push(entry);

          const dir = path.dirname(file.fsPath);
          if (!this.cache.has(dir)) {
            this.cache.set(dir, []);
          }
          this.cache.get(dir)!.push(entry);
        }
      } catch {}
    }

    this.cacheTimestamp = now;
    return components;
  }

  private findUnimportedComponent(
    document: vscode.TextDocument,
    range: vscode.Range,
    components: ComponentEntry[]
  ): ComponentEntry | undefined {
    const text = document.getText(range);
    const name = text.trim();

    if (!/^[A-Z][a-zA-Z0-9_]*$/.test(name)) return undefined;

    const imports = this.getImportedNames(document);
    if (imports.has(name)) return undefined;

    const docDir = path.dirname(document.uri.fsPath);
    const defined = this.getDefinedNamesInDocument(document);
    if (defined.has(name)) return undefined;

    return components.find(c => c.name === name && c.filePath !== document.uri.fsPath);
  }

  private getImportedNames(document: vscode.TextDocument): Set<string> {
    const names = new Set<string>();
    const importRegex = /import\s*\{([^}]+)\}/g;
    let match: RegExpExecArray | null;
    while ((match = importRegex.exec(document.getText())) !== null) {
      match[1].split(',').forEach(n => {
        const trimmed = n.trim();
        if (trimmed) names.add(trimmed);
      });
    }
    return names;
  }

  private getDefinedNamesInDocument(document: vscode.TextDocument): Set<string> {
    const names = new Set<string>();
    const text = document.getText();
    const defRegex = /(?:component|page|:store)\s+([A-Z][a-zA-Z0-9_]*)\s*[:{]/g;
    let match: RegExpExecArray | null;
    while ((match = defRegex.exec(text)) !== null) {
      names.add(match[1]);
    }
    return names;
  }

  private isInChildrenBlock(document: vscode.TextDocument, position: vscode.Position): boolean {
    const text = document.getText();
    const offset = document.offsetAt(position);
    const beforeText = text.substring(0, offset);

    const lastChildrenOpen = beforeText.lastIndexOf('children: [');
    if (lastChildrenOpen === -1) return false;

    const afterChildren = text.substring(lastChildrenOpen);
    let depth = 0;
    for (let i = 0; i < afterChildren.length; i++) {
      const ch = afterChildren[i];
      if (ch === '[') depth++;
      else if (ch === ']') depth--;
      if (depth === 0 && i < offset - lastChildrenOpen) return false;
      if (depth < 0) return false;
    }
    return depth > 0;
  }

  async provideCodeActions(
    document: vscode.TextDocument,
    range: vscode.Range | vscode.Selection,
    _context: vscode.CodeActionContext,
    _token: vscode.CancellationToken
  ): Promise<vscode.CodeAction[] | undefined> {
    if (!this.isInChildrenBlock(document, range.start)) return undefined;

    const components = await this.scanWorkspaceComponents();
    const component = this.findUnimportedComponent(document, range, components);
    if (!component) return undefined;

    const documentDir = path.dirname(document.uri.fsPath);
    let relativePath = path.relative(documentDir, component.filePath);
    if (!relativePath.startsWith('.')) {
      relativePath = `./${relativePath}`;
    }
    relativePath = relativePath.replace(/\.fr$/, '');

    const action = new vscode.CodeAction(
      `Import '${component.name}' from '${relativePath}'`,
      vscode.CodeActionKind.QuickFix
    );
    action.edit = new vscode.WorkspaceEdit();

    const importStatement = `import { ${component.name} } "${relativePath}"\n`;
    const firstLine = document.lineAt(0);
    action.edit.insert(document.uri, new vscode.Position(0, 0), importStatement);

    return [action];
  }
}

export function registerImportManager(context: vscode.ExtensionContext, client: LanguageClient): vscode.Disposable[] {
  const disposables: vscode.Disposable[] = [];

  const provider = new ImportManager(client);
  disposables.push(
    vscode.languages.registerCodeActionsProvider('frame', provider, {
      providedCodeActionKinds: [vscode.CodeActionKind.QuickFix],
    })
  );

  return disposables;
}
