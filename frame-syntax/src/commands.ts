import * as vscode from "vscode";
import { LanguageClient } from "vscode-languageclient/node";
import * as fs from "fs";
import * as path from "path";

function getFrameBinary(): string {
  return (
    vscode.workspace.getConfiguration("frame").get<string>("path") || "frame"
  );
}

function getWorkspaceRoot(): string {
  const configured = vscode.workspace
    .getConfiguration("frame")
    .get<string>("workspaceRoot");
  if (configured) return configured;
  return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || "";
}

function runInTerminal(args: string[]): void {
  const terminal = vscode.window.createTerminal({
    name: "Frame",
    shellPath: getFrameBinary(),
    shellArgs: args,
    iconPath: new vscode.ThemeIcon("symbol-color"),
  });
  terminal.show();
}

async function scaffoldFile(
  name: string,
  template: string,
  relativeDir: string,
): Promise<void> {
  const root = getWorkspaceRoot();
  if (!root) {
    vscode.window.showErrorMessage("No workspace folder open.");
    return;
  }

  const targetDir = path.join(root, relativeDir);
  if (!fs.existsSync(targetDir)) {
    fs.mkdirSync(targetDir, { recursive: true });
  }

  const filePath = path.join(targetDir, `${name}.fr`);
  if (fs.existsSync(filePath)) {
    vscode.window.showErrorMessage(
      `File ${name}.fr already exists in ${relativeDir}.`,
    );
    return;
  }

  fs.writeFileSync(filePath, template, "utf-8");
  const doc = await vscode.workspace.openTextDocument(filePath);
  vscode.window.showTextDocument(doc);
}

export function registerCommands(
  context: vscode.ExtensionContext,
  client: LanguageClient,
): vscode.Disposable[] {
  const disposables: vscode.Disposable[] = [];

  disposables.push(
    vscode.commands.registerCommand("frame.build", () => {
      runInTerminal(["build"]);
    }),
  );

  disposables.push(
    vscode.commands.registerCommand("frame.buildWatch", () => {
      runInTerminal(["build", "--watch"]);
    }),
  );

  disposables.push(
    vscode.commands.registerCommand("frame.test", () => {
      runInTerminal(["test"]);
    }),
  );

  disposables.push(
    vscode.commands.registerCommand("frame.testFilter", async () => {
      const tests = await client.sendRequest<string[]>("frame/listTests");
      const items = tests.map((t) => ({ label: t }));
      const selected = await vscode.window.showQuickPick(items, {
        placeHolder: "Select a test to run",
      });
      if (selected) {
        runInTerminal(["test", "--filter", selected.label]);
      }
    }),
  );

  disposables.push(
    vscode.commands.registerCommand("frame.deploy", async () => {
      const target = await vscode.window.showQuickPick(
        [
          { label: "iOS", description: "Deploy to iOS simulator/device" },
          {
            label: "Android",
            description: "Deploy to Android emulator/device",
          },
        ],
        { placeHolder: "Select deployment target" },
      );
      if (target) {
        runInTerminal(["deploy", target.label.toLowerCase()]);
      }
    }),
  );

  disposables.push(
    vscode.commands.registerCommand("frame.lint", () => {
      runInTerminal(["lint"]);
    }),
  );

  disposables.push(
    vscode.commands.registerCommand("frame.lintFile", () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        vscode.window.showWarningMessage("No active editor.");
        return;
      }
      const filePath = editor.document.uri.fsPath;
      runInTerminal(["lint", filePath]);
    }),
  );

  disposables.push(
    vscode.commands.registerCommand("frame.pluginAdd", async () => {
      const name = await vscode.window.showInputBox({
        placeHolder: "plugin-name",
        prompt: "Enter the plugin name to add",
        validateInput: (value) =>
          value.trim() ? null : "Plugin name is required",
      });
      if (name) {
        runInTerminal(["plugin", "add", name.trim()]);
      }
    }),
  );

  disposables.push(
    vscode.commands.registerCommand("frame.pluginList", async () => {
      try {
        const plugins = await client.sendRequest<string[]>("frame/listPlugins");
        const items = plugins.map((p) => ({
          label: p,
          iconPath: new vscode.ThemeIcon("extensions"),
        }));
        if (items.length === 0) {
          vscode.window.showInformationMessage("No plugins installed.");
          return;
        }
        await vscode.window.showQuickPick(items, {
          placeHolder: "Installed plugins",
          matchOnDescription: true,
        });
      } catch {
        vscode.window.showErrorMessage("Failed to fetch plugin list from LSP.");
      }
    }),
  );

  disposables.push(
    vscode.commands.registerCommand("frame.iconList", async () => {
      try {
        const icons = await client.sendRequest<string[]>("frame/listIcons");
        const items = icons.map((i) => ({
          label: i,
          iconPath: new vscode.ThemeIcon("symbol-icon"),
        }));
        if (items.length === 0) {
          vscode.window.showInformationMessage("No icons registered.");
          return;
        }
        const selected = await vscode.window.showQuickPick(items, {
          placeHolder: "Registered icons",
          matchOnDescription: true,
        });
        if (selected && vscode.window.activeTextEditor) {
          vscode.window.activeTextEditor.edit((edit) => {
            const pos = vscode.window.activeTextEditor!.selection.active;
            edit.insert(pos, selected.label);
          });
        }
      } catch {
        vscode.window.showErrorMessage("Failed to fetch icon list from LSP.");
      }
    }),
  );

  disposables.push(
    vscode.commands.registerCommand("frame.iconGenerate", async () => {
      const target = await vscode.window.showQuickPick(
        [
          { label: "iOS", description: "Generate iOS icons" },
          { label: "Android", description: "Generate Android icons" },
          { label: "All", description: "Generate icons for all platforms" },
        ],
        { placeHolder: "Select platform" },
      );
      if (target) {
        const arg =
          target.label.toLowerCase() === "all"
            ? "--all"
            : `--${target.label.toLowerCase()}`;
        runInTerminal(["icons", "generate", arg]);
      }
    }),
  );

  disposables.push(
    vscode.commands.registerCommand("frame.newPage", async () => {
      const name = await vscode.window.showInputBox({
        placeHolder: "HomePage",
        prompt: "Enter the page name (PascalCase)",
        validateInput: (value) =>
          /^[A-Z][a-zA-Z0-9]*$/.test(value)
            ? null
            : "Must be PascalCase (e.g. HomePage)",
      });
      if (!name) return;

      const routeName = name
        .replace(/([A-Z])/g, "-$1")
        .toLowerCase()
        .replace(/^-/, "/");

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
      await scaffoldFile(name, template, "src/pages");
    }),
  );

  disposables.push(
    vscode.commands.registerCommand("frame.newComponent", async () => {
      const name = await vscode.window.showInputBox({
        placeHolder: "MyComponent",
        prompt: "Enter the component name (PascalCase)",
        validateInput: (value) =>
          /^[A-Z][a-zA-Z0-9]*$/.test(value)
            ? null
            : "Must be PascalCase (e.g. MyComponent)",
      });
      if (!name) return;

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
      await scaffoldFile(name, template, "src/components");
    }),
  );

  disposables.push(
    vscode.commands.registerCommand("frame.newStore", async () => {
      const name = await vscode.window.showInputBox({
        placeHolder: "CounterStore",
        prompt: "Enter the store name (PascalCase)",
        validateInput: (value) =>
          /^[A-Z][a-zA-Z0-9]*$/.test(value)
            ? null
            : "Must be PascalCase (e.g. CounterStore)",
      });
      if (!name) return;

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
      await scaffoldFile(name, template, "src/stores");
    }),
  );

  disposables.push(
    vscode.commands.registerCommand("frame.openDocs", () => {
      vscode.env.openExternal(vscode.Uri.parse("https://frame-lang.org/docs"));
    }),
  );

  disposables.push(
    vscode.commands.registerCommand("frame.openAppIconDocs", () => {
      vscode.env.openExternal(
        vscode.Uri.parse("https://frame-lang.org/docs/app-icons"),
      );
    }),
  );

  disposables.push(
    vscode.commands.registerCommand("frame.restartLsp", async () => {
      vscode.window.showInformationMessage(
        "Restarting Frame Language Server...",
      );
      await client.restart();
      vscode.window.showInformationMessage("Frame Language Server restarted.");
    }),
  );

  return disposables;
}
