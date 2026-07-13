import * as vscode from "vscode";

/**
 * Adds a subtle green color tint to all .fr files in the explorer,
 * reinforcing the Frame brand color without per-type badges.
 */
class FrameFileDecorationProvider implements vscode.FileDecorationProvider {
  private _onChange = new vscode.EventEmitter<vscode.Uri | vscode.Uri[] | undefined>();
  readonly onDidChangeFileDecorations = this._onChange.event;

  provideFileDecoration(uri: vscode.Uri): vscode.FileDecoration | undefined {
    if (!uri.fsPath.endsWith(".fr")) return undefined;
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

export function registerFileDecorations(
  context: vscode.ExtensionContext
): vscode.Disposable[] {
  const provider = new FrameFileDecorationProvider();
  const disposables: vscode.Disposable[] = [];

  disposables.push(vscode.window.registerFileDecorationProvider(provider));

  const watcher = vscode.workspace.createFileSystemWatcher("**/*.fr");
  watcher.onDidCreate(() => provider.refresh());
  watcher.onDidDelete(() => provider.refresh());
  disposables.push(watcher);

  return disposables;
}
