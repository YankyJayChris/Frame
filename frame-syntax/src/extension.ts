import * as vscode from "vscode";
import * as fs from "fs";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";
import { registerCommands } from "./commands";
import { registerStatusBar } from "./status-bar";
import { registerDecorations } from "./decorations";
import { registerImportManager } from "./import-manager";
import { registerCompletionProviders } from "./completion-provider";
import { registerIconPicker } from "./icon-picker";
import { registerAppIconManager } from "./app-icon-manager";
import { registerSidePanel } from "./side-panel";
import { registerFileDecorations } from "./file-decorations";

let client: LanguageClient | undefined;

function findFrameBinary(): string | undefined {
  const configPath = vscode.workspace
    .getConfiguration("frame")
    .get<string>("path");
  if (configPath) {
    try {
      fs.accessSync(configPath, fs.constants.X_OK);
      return configPath;
    } catch {}
  }

  const envPath = process.env.PATH || "";
  const pathDirs = envPath.split(":").filter(Boolean);
  for (const dir of pathDirs) {
    const candidate = `${dir}/frame`;
    try {
      fs.accessSync(candidate, fs.constants.X_OK);
      return candidate;
    } catch {}
  }

  const home = process.env.HOME || "";
  const commonLocations = [
    "/usr/local/bin/frame",
    "/opt/homebrew/bin/frame",
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
  // Always register UI features — these work regardless of whether the Frame binary is installed
  context.subscriptions.push(
    ...registerFileDecorations(context),
    ...registerCompletionProviders(context),
    ...registerIconPicker(context),
    ...registerAppIconManager(context),
    ...registerSidePanel(context),
    ...registerDecorations(context),
  );

  // Apply Material Icon Theme associations for .fr files if that theme is active
  applyMaterialIconAssociations();

  const binary = findFrameBinary();
  if (!binary) {
    const action = await vscode.window.showWarningMessage(
      'Frame binary not found. Syntax features active; install Frame CLI for build/LSP support.',
      "Install Frame",
      "Open Settings",
    );
    if (action === "Install Frame") {
      vscode.env.openExternal(
        vscode.Uri.parse("https://frame-lang.org/install"),
      );
    } else if (action === "Open Settings") {
      vscode.commands.executeCommand(
        "workbench.action.openSettings",
        "frame.path",
      );
    }
    // LSP-powered commands unavailable without binary — UI features above still work
    return;
  }

  const serverOptions: ServerOptions = {
    command: binary,
    args: ["lsp"],
    options: { env: { ...process.env, RUST_LOG: "error" } },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "frame" }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher("**/*.fr"),
    },
    outputChannel: vscode.window.createOutputChannel("Frame Language Server"),
    traceOutputChannel: vscode.window.createOutputChannel("Frame LSP Trace"),
    progressOnInitialization: true,
    initializationOptions: {
      workspaceRoot:
        vscode.workspace
          .getConfiguration("frame")
          .get<string>("workspaceRoot") ||
        vscode.workspace.workspaceFolders?.[0]?.uri.fsPath,
    },
  };

  client = new LanguageClient(
    "frame-lsp",
    "Frame Language Server",
    serverOptions,
    clientOptions,
  );

  client.start();

  context.subscriptions.push(
    ...registerCommands(context, client),
    ...registerStatusBar(context, client),
    ...registerImportManager(context, client),
  );
}

/**
 * If Material Icon Theme is active, add .fr file associations so it shows
 * appropriate icons (using its "document" icon family as a fallback,
 * or custom associations the user can override).
 */
function applyMaterialIconAssociations() {
  const iconTheme = vscode.workspace
    .getConfiguration("workbench")
    .get<string>("iconTheme");

  if (iconTheme !== "material-icon-theme") return;

  const config = vscode.workspace.getConfiguration("material-icon-theme");
  const existing = config.get<Record<string, string>>("files.associations") ?? {};

  const frAssociations: Record<string, string> = {
    "*.fr":        "document",
    "*.test.fr":   "test",
    "project.fr":  "settings",
  };

  // Only write if something is actually new (avoid triggering unnecessary reloads)
  const merged = { ...existing };
  let changed = false;
  for (const [pattern, icon] of Object.entries(frAssociations)) {
    if (!merged[pattern]) {
      merged[pattern] = icon;
      changed = true;
    }
  }

  if (changed) {
    config.update(
      "files.associations",
      merged,
      vscode.ConfigurationTarget.Global,
    );
  }
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) return undefined;
  return client.stop();
}
