import * as vscode from 'vscode';
import * as fs from 'fs';
import { LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';
import { registerCommands } from './commands';
import { registerStatusBar } from './status-bar';
import { registerDecorations } from './decorations';
import { registerImportManager } from './import-manager';
import { registerCompletionProviders } from './completion-provider';
import { registerIconPicker } from './icon-picker';
import { registerSidePanel } from './side-panel';

let client: LanguageClient | undefined;

function findFrameBinary(): string | undefined {
  const configPath = vscode.workspace.getConfiguration('frame').get<string>('path');
  if (configPath) {
    try {
      fs.accessSync(configPath, fs.constants.X_OK);
      return configPath;
    } catch {}
  }

  const envPath = process.env.PATH || '';
  const pathDirs = envPath.split(':').filter(Boolean);
  for (const dir of pathDirs) {
    const candidate = `${dir}/frame`;
    try {
      fs.accessSync(candidate, fs.constants.X_OK);
      return candidate;
    } catch {}
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
    } catch {}
  }

  return undefined;
}

export async function activate(context: vscode.ExtensionContext) {
  const binary = findFrameBinary();
  if (!binary) {
    const action = await vscode.window.showErrorMessage(
      'Frame binary not found. Please install Frame or set "frame.path" in settings.',
      'Install Frame',
      'Open Settings'
    );
    if (action === 'Install Frame') {
      vscode.env.openExternal(vscode.Uri.parse('https://frame-lang.org/install'));
    } else if (action === 'Open Settings') {
      vscode.commands.executeCommand('workbench.action.openSettings', 'frame.path');
    }
    return;
  }

  const serverOptions: ServerOptions = {
    command: binary,
    args: ['lsp'],
    options: { env: { ...process.env, RUST_LOG: 'error' } },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'frame' }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*.fr'),
    },
    outputChannel: vscode.window.createOutputChannel('Frame Language Server'),
    traceOutputChannel: vscode.window.createOutputChannel('Frame LSP Trace'),
    progressOnInitialization: true,
    initializationOptions: {
      workspaceRoot: vscode.workspace.getConfiguration('frame').get<string>('workspaceRoot') ||
        (vscode.workspace.workspaceFolders?.[0]?.uri.fsPath),
    },
  };

  client = new LanguageClient('frame-lsp', 'Frame Language Server', serverOptions, clientOptions);

  client.start();

  context.subscriptions.push(
    ...registerCommands(context, client),
    ...registerStatusBar(context, client),
    ...registerDecorations(context),
    ...registerImportManager(context, client),
    ...registerCompletionProviders(context),
    ...registerIconPicker(context),
    ...registerSidePanel(context),
  );
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) return undefined;
  return client.stop();
}
