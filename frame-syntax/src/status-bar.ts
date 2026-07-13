import * as vscode from 'vscode';
import { LanguageClient, State } from 'vscode-languageclient/node';

export function registerStatusBar(context: vscode.ExtensionContext, client: LanguageClient): vscode.Disposable[] {
  const disposables: vscode.Disposable[] = [];

  const logoItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
  logoItem.text = '$(symbol-color) Frame';
  logoItem.tooltip = 'Frame Language';
  logoItem.command = 'frame.openDocs';
  logoItem.show();
  disposables.push(logoItem);

  const diagItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
  disposables.push(diagItem);

  const updateDiagnostics = () => {
    const activeEditor = vscode.window.activeTextEditor;
    if (!activeEditor) {
      diagItem.hide();
      return;
    }

    const uri = activeEditor.document.uri;
    const diagnostics = vscode.languages.getDiagnostics(uri);
    const errors = diagnostics.filter(d => d.severity === vscode.DiagnosticSeverity.Error).length;
    const warnings = diagnostics.filter(d => d.severity === vscode.DiagnosticSeverity.Warning).length;

    if (errors === 0 && warnings === 0) {
      diagItem.text = '$(check) 0 issues';
      diagItem.tooltip = 'No problems detected';
      diagItem.backgroundColor = undefined;
    } else if (errors > 0) {
      diagItem.text = `$(error) ${errors} error${errors > 1 ? 's' : ''}${
        warnings > 0 ? `  $(warning) ${warnings} warning${warnings > 1 ? 's' : ''}` : ''
      }`;
      diagItem.backgroundColor = new vscode.ThemeColor('statusBarItem.errorBackground');
      diagItem.tooltip = `${errors} error(s), ${warnings} warning(s) — click to open Problems`;
    } else {
      diagItem.text = `$(warning) ${warnings} warning${warnings > 1 ? 's' : ''}`;
      diagItem.backgroundColor = new vscode.ThemeColor('statusBarItem.warningBackground');
      diagItem.tooltip = `${warnings} warning(s) — click to open Problems`;
    }
    diagItem.command = 'workbench.action.problems.focus';
    diagItem.show();
  };

  disposables.push(vscode.window.onDidChangeActiveTextEditor(updateDiagnostics));
  disposables.push(vscode.languages.onDidChangeDiagnostics(updateDiagnostics));
  updateDiagnostics();

  const buildItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 90);
  buildItem.text = '$(tools) Build';
  buildItem.tooltip = 'Build Frame project';
  buildItem.command = 'frame.build';
  buildItem.show();
  disposables.push(buildItem);

  const lspItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 80);
  lspItem.text = '$(hubot) LSP: starting...';
  lspItem.tooltip = 'Frame Language Server status';
  lspItem.show();
  disposables.push(lspItem);

  client.onDidChangeState((event) => {
    switch (event.newState) {
      case State.Running:
        lspItem.text = '$(hubot) LSP: connected';
        lspItem.tooltip = 'Frame Language Server is running';
        lspItem.backgroundColor = undefined;
        break;
      case State.Starting:
        lspItem.text = '$(loading~spin) LSP: starting...';
        lspItem.tooltip = 'Frame Language Server is starting';
        break;
      case State.Stopped:
        lspItem.text = '$(hubot) LSP: disconnected';
        lspItem.tooltip = 'Frame Language Server is stopped';
        lspItem.backgroundColor = new vscode.ThemeColor('statusBarItem.errorBackground');
        break;
    }
  });

  return disposables;
}
