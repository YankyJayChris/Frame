import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';

type NodeType = 'page' | 'component' | 'store' | 'function' | 'icon' | 'plugin' | 'action' | 'folder';

interface TreeNode {
  label: string;
  description?: string;
  type: NodeType;
  filePath?: string;
  children?: TreeNode[];
  command?: string;
  iconPath?: vscode.ThemeIcon;
}

class FrameExplorerProvider implements vscode.TreeDataProvider<TreeNode> {
  private _onDidChangeTreeData = new vscode.EventEmitter<TreeNode | undefined>();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  constructor(private workspaceRoot: string) {}

  refresh(): void {
    this._onDidChangeTreeData.fire(undefined);
  }

  getTreeItem(element: TreeNode): vscode.TreeItem {
    const treeItem = new vscode.TreeItem(element.label);

    if (element.type === 'page') {
      treeItem.iconPath = new vscode.ThemeIcon('file');
      treeItem.contextValue = 'pageFile';
      treeItem.command = {
        command: 'vscode.open',
        title: 'Open File',
        arguments: [element.filePath ? vscode.Uri.file(element.filePath) : undefined],
      };
    } else if (element.type === 'component') {
      treeItem.iconPath = new vscode.ThemeIcon('symbol-class');
      treeItem.contextValue = 'componentFile';
      treeItem.command = {
        command: 'vscode.open',
        title: 'Open File',
        arguments: [element.filePath ? vscode.Uri.file(element.filePath) : undefined],
      };
    } else if (element.type === 'store') {
      treeItem.iconPath = new vscode.ThemeIcon('symbol-variable');
      treeItem.contextValue = 'storeFile';
      treeItem.command = {
        command: 'vscode.open',
        title: 'Open File',
        arguments: [element.filePath ? vscode.Uri.file(element.filePath) : undefined],
      };
    } else if (element.type === 'function') {
      treeItem.iconPath = new vscode.ThemeIcon('symbol-function');
      treeItem.contextValue = 'functionFile';
      treeItem.command = {
        command: 'vscode.open',
        title: 'Open File',
        arguments: [element.filePath ? vscode.Uri.file(element.filePath) : undefined],
      };
    } else if (element.type === 'icon') {
      treeItem.iconPath = new vscode.ThemeIcon('symbol-icon');
    } else if (element.type === 'plugin') {
      treeItem.iconPath = new vscode.ThemeIcon('extensions');
    } else if (element.type === 'action') {
      treeItem.iconPath = element.iconPath || new vscode.ThemeIcon('play');
      treeItem.command = {
        command: element.command || '',
        title: element.label,
      };
    } else {
      treeItem.iconPath = new vscode.ThemeIcon('folder');
      treeItem.collapsibleState = vscode.TreeItemCollapsibleState.Collapsed;
    }

    if (element.children) {
      treeItem.collapsibleState = vscode.TreeItemCollapsibleState.Collapsed;
    }

    treeItem.description = element.description;
    return treeItem;
  }

  async getChildren(element?: TreeNode): Promise<TreeNode[]> {
    if (!element) {
      return this.getRootNodes();
    }
    if (element.children) {
      return element.children;
    }
    return [];
  }

  private async getRootNodes(): Promise<TreeNode[]> {
    const nodes: TreeNode[] = [];

    const projectNode: TreeNode = {
      label: 'Project',
      type: 'folder',
      children: await this.getProjectFiles(),
    };
    nodes.push(projectNode);

    const iconsNode: TreeNode = {
      label: 'Icons',
      type: 'folder',
      children: this.getIconCategories(),
    };
    nodes.push(iconsNode);

    const actionsNode: TreeNode = {
      label: 'Quick Actions',
      type: 'folder',
      children: this.getQuickActions(),
    };
    nodes.push(actionsNode);

    return nodes;
  }

  private async getProjectFiles(): Promise<TreeNode[]> {
    const children: TreeNode[] = [];

    if (!this.workspaceRoot || !fs.existsSync(this.workspaceRoot)) {
      return children;
    }

    const frFiles = await vscode.workspace.findFiles('**/*.fr');

    const pages: TreeNode[] = [];
    const components: TreeNode[] = [];
    const stores: TreeNode[] = [];
    const functions: TreeNode[] = [];

    const componentRegex = /component\s+([A-Z][a-zA-Z0-9_]*)\s*:/g;
    const pageRegex = /page:\s*\{[^}]*name:\s*"([^"]+)"/g;
    const storeRegex = /:store\s+([A-Z][a-zA-Z0-9_]*)\s*\{/g;
    const fnRegex = /fn\s+([a-z_][a-zA-Z0-9_]*)\s*(?=:)/g;

    for (const file of frFiles) {
      try {
        const content = fs.readFileSync(file.fsPath, 'utf-8');
        const relPath = path.relative(this.workspaceRoot, file.fsPath);

        let match: RegExpExecArray | null;

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
      } catch {}
    }

    const pageFolder: TreeNode = {
      label: `Pages (${pages.length})`,
      type: 'folder',
      children: pages,
    };
    children.push(pageFolder);

    const componentFolder: TreeNode = {
      label: `Components (${components.length})`,
      type: 'folder',
      children: components,
    };
    children.push(componentFolder);

    const storeFolder: TreeNode = {
      label: `Stores (${stores.length})`,
      type: 'folder',
      children: stores,
    };
    children.push(storeFolder);

    const functionFolder: TreeNode = {
      label: `Functions (${functions.length})`,
      type: 'folder',
      children: functions,
    };
    children.push(functionFolder);

    return children;
  }

  private getIconCategories(): TreeNode[] {
    const categories = new Map<string, TreeNode[]>();

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
      type: 'folder' as NodeType,
      children: cat.icons.map(iconName => ({
        label: iconName,
        type: 'icon' as NodeType,
      })),
    }));
  }

  private getQuickActions(): TreeNode[] {
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

export function registerSidePanel(context: vscode.ExtensionContext): vscode.Disposable[] {
  const disposables: vscode.Disposable[] = [];

  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || '';
  const provider = new FrameExplorerProvider(workspaceRoot);

  disposables.push(
    vscode.window.registerTreeDataProvider('frameExplorer', provider)
  );

  disposables.push(
    vscode.commands.registerCommand('frameExplorer.refresh', () => provider.refresh())
  );

  if (workspaceRoot) {
    const watcher = vscode.workspace.createFileSystemWatcher('**/*.fr');
    watcher.onDidChange(() => provider.refresh());
    watcher.onDidCreate(() => provider.refresh());
    watcher.onDidDelete(() => provider.refresh());
    disposables.push(watcher);
  }

  return disposables;
}
