import * as vscode from "vscode";
import * as fs from "fs";
import * as path from "path";

interface IconInfo {
  path: string;
  format: "svg" | "png" | "jpeg" | "default";
  exists: boolean;
  isValid: boolean;
  fileSize?: number;
}

const SUPPORTED_FORMATS = [".svg", ".png", ".jpg", ".jpeg"];
const DEFAULT_ICON_PATH = "assets/icons/frame-default.svg";

export function registerAppIconManager(
  context: vscode.ExtensionContext,
): vscode.Disposable[] {
  const disposables: vscode.Disposable[] = [];

  // Command: Open app icon picker
  disposables.push(
    vscode.commands.registerCommand("frame.openAppIconPicker", async () => {
      const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
      if (!workspaceRoot) {
        vscode.window.showErrorMessage("No workspace folder open.");
        return;
      }

      const configPath = path.join(workspaceRoot, "frame.config.json");
      if (!fs.existsSync(configPath)) {
        vscode.window.showErrorMessage("frame.config.json not found.");
        return;
      }

      // Show quick pick with options
      const choice = await vscode.window.showQuickPick(
        [
          {
            label: "$(symbol-file) Use Default Icon",
            description: "Frame default lime green icon",
            detail: "Recommended for quick setup",
            id: "default",
          },
          {
            label: "$(file-media) Select Custom File",
            description: "SVG, PNG, or JPEG from workspace",
            detail: "Support for SVG, PNG, JPEG formats",
            id: "custom",
          },
          {
            label: "$(inspect) View Icon Info",
            description: "Check current icon configuration",
            detail: "Inspect current settings",
            id: "info",
          },
        ],
        { placeHolder: "Choose app icon action" },
      );

      if (!choice) return;

      switch (choice.id) {
        case "default":
          await setDefaultIcon(configPath, workspaceRoot);
          break;
        case "custom":
          await selectCustomIcon(configPath, workspaceRoot);
          break;
        case "info":
          await showIconInfo(configPath, workspaceRoot);
          break;
      }
    }),
  );

  // Command: Validate icons
  disposables.push(
    vscode.commands.registerCommand("frame.validateIcons", async () => {
      const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
      if (!workspaceRoot) {
        vscode.window.showErrorMessage("No workspace folder open.");
        return;
      }

      const configPath = path.join(workspaceRoot, "frame.config.json");
      if (!fs.existsSync(configPath)) {
        vscode.window.showErrorMessage("frame.config.json not found.");
        return;
      }

      const iconInfo = getIconInfo(configPath, workspaceRoot);
      const icon = iconInfo[0];

      let message = "";
      let type: "info" | "error" = "info";

      if (icon.format === "default") {
        message = `✓ Using default Frame icon (${DEFAULT_ICON_PATH})`;
      } else if (icon.exists && icon.isValid) {
        message = `✓ Custom icon valid: ${icon.path}\n(Format: ${icon.format.toUpperCase()}, Size: ${formatFileSize(icon.fileSize || 0)})`;
      } else if (!icon.exists) {
        message = `✗ Icon file not found: ${icon.path}`;
        type = "error";
      } else {
        message = `✗ Invalid icon format: ${icon.path}\nSupported formats: ${SUPPORTED_FORMATS.join(", ")}`;
        type = "error";
      }

      if (type === "info") {
        vscode.window.showInformationMessage(message);
      } else {
        vscode.window.showErrorMessage(message);
      }
    }),
  );

  // Command: Preview icon path (in config)
  disposables.push(
    vscode.commands.registerCommand("frame.previewAppIcon", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || editor.document.fileName !== "frame.config.json") {
        vscode.window.showWarningMessage(
          "Open frame.config.json to preview app icon.",
        );
        return;
      }

      const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
      if (!workspaceRoot) return;

      const icons = getIconInfo(editor.document.uri.fsPath, workspaceRoot);
      if (icons.length === 0) return;

      const icon = icons[0];
      if (icon.format === "default") {
        vscode.window.showInformationMessage(
          `$(file-media) Using default Frame app icon\n\nLocation: ${DEFAULT_ICON_PATH}\nColor: Lime green (#BCFB70)\nSize: 1080×1080px`,
        );
      } else {
        let msg = `$(file-media) Custom App Icon\n\n`;
        msg += `Path: ${icon.path}\n`;
        msg += `Format: ${icon.format.toUpperCase()}\n`;
        msg += `Status: ${icon.isValid ? "✓ Valid" : "✗ Invalid"}\n`;
        if (icon.fileSize) {
          msg += `Size: ${formatFileSize(icon.fileSize)}`;
        }
        vscode.window.showInformationMessage(msg);
      }
    }),
  );

  return disposables;
}

async function setDefaultIcon(
  configPath: string,
  workspaceRoot: string,
): Promise<void> {
  try {
    const content = fs.readFileSync(configPath, "utf-8");
    const config = JSON.parse(content);

    // Remove icon field to use default
    if ("icon" in config) {
      delete config.icon;
      fs.writeFileSync(configPath, JSON.stringify(config, null, 2), "utf-8");
      vscode.window.showInformationMessage(
        `✓ Set to default Frame icon (${DEFAULT_ICON_PATH})`,
      );
    } else {
      vscode.window.showInformationMessage(
        `✓ Already using default Frame icon`,
      );
    }

    // Update active editor if it's frame.config.json
    const activeEditor = vscode.window.activeTextEditor;
    if (
      activeEditor &&
      activeEditor.document.fileName.endsWith("frame.config.json")
    ) {
      const doc = await vscode.workspace.openTextDocument(configPath);
      await vscode.window.showTextDocument(doc);
    }
  } catch (error) {
    vscode.window.showErrorMessage(`Failed to update config: ${error}`);
  }
}

async function selectCustomIcon(
  configPath: string,
  workspaceRoot: string,
): Promise<void> {
  try {
    // Find all potential icon files
    const iconFiles = findIconFiles(workspaceRoot);

    if (iconFiles.length === 0) {
      vscode.window.showWarningMessage(
        "No icon files (SVG, PNG, JPEG) found in workspace. Create one in assets/icons/ directory.",
      );
      return;
    }

    const quickPickItems = iconFiles.map((file) => ({
      label: path.basename(file),
      description: path.relative(workspaceRoot, file),
      filePath: file,
    }));

    const selected = await vscode.window.showQuickPick(quickPickItems, {
      placeHolder: "Select an icon file",
    });

    if (!selected) return;

    // Get relative path from workspace root
    const relativePath = path.relative(workspaceRoot, selected.filePath);

    // Update config.json
    const content = fs.readFileSync(configPath, "utf-8");
    const config = JSON.parse(content);
    config.icon = relativePath;

    fs.writeFileSync(configPath, JSON.stringify(config, null, 2), "utf-8");

    const fileSize = fs.statSync(selected.filePath).size;
    vscode.window.showInformationMessage(
      `✓ Set custom icon: ${relativePath}\n(${formatFileSize(fileSize)})`,
    );

    // Update active editor if it's frame.config.json
    const activeEditor = vscode.window.activeTextEditor;
    if (
      activeEditor &&
      activeEditor.document.fileName.endsWith("frame.config.json")
    ) {
      const doc = await vscode.workspace.openTextDocument(configPath);
      await vscode.window.showTextDocument(doc);
    }
  } catch (error) {
    vscode.window.showErrorMessage(`Failed to update icon: ${error}`);
  }
}

async function showIconInfo(
  configPath: string,
  workspaceRoot: string,
): Promise<void> {
  try {
    const icons = getIconInfo(configPath, workspaceRoot);
    if (icons.length === 0) return;

    const icon = icons[0];

    let message = "**App Icon Information**\n\n";

    if (icon.format === "default") {
      message += `**Status:** Using default Frame icon\n`;
      message += `**Path:** ${DEFAULT_ICON_PATH}\n`;
      message += `**Format:** SVG (vector)\n`;
      message += `**Color:** Lime green (#BCFB70)\n`;
      message += `**Size:** 1080×1080 px\n`;
      message += `**Generated Files:**\n`;
      message += `  • iOS: 12 icon sizes (AppIcon.appiconset)\n`;
      message += `  • Android: 6 densities × 2 variants (mipmap)`;
    } else {
      message += `**Status:** ${icon.exists ? (icon.isValid ? "✓ Valid" : "✗ Invalid") : "✗ Not found"}\n`;
      message += `**Path:** ${icon.path}\n`;
      message += `**Format:** ${icon.format.toUpperCase()}\n`;
      if (icon.fileSize) {
        message += `**Size:** ${formatFileSize(icon.fileSize)}\n`;
      }
      if (icon.exists && icon.isValid) {
        message += `**Generated Files:**\n`;
        message += `  • iOS: 12 icon sizes (AppIcon.appiconset)\n`;
        message += `  • Android: 6 densities × 2 variants (mipmap)`;
      }
    }

    const panel = vscode.window.createWebviewPanel(
      "frameIconInfo",
      "Frame App Icon Info",
      vscode.ViewColumn.Beside,
      {},
    );

    panel.webview.html = `
<!DOCTYPE html>
<html>
<head>
  <style>
    body {
      font-family: var(--vscode-font-family);
      padding: 20px;
      line-height: 1.6;
    }
    h2 { color: var(--vscode-foreground); }
    .status-ok { color: #4ec9b0; }
    .status-error { color: #f48771; }
    .info-block {
      background: var(--vscode-editor-background);
      padding: 10px;
      border-radius: 4px;
      margin: 10px 0;
      border-left: 3px solid #0e639c;
    }
    code { background: var(--vscode-editor-inlineChat-background); padding: 2px 4px; }
  </style>
</head>
<body>
  <h2>App Icon Configuration</h2>
  <div class="info-block">
    <strong>Path:</strong> <code>${icon.format === "default" ? DEFAULT_ICON_PATH : icon.path}</code><br>
    <strong>Format:</strong> ${icon.format.toUpperCase()}<br>
    <strong>Status:</strong> <span class="${icon.isValid ? "status-ok" : "status-error"}">
      ${icon.format === "default" ? "✓ Using default" : icon.isValid ? "✓ Valid" : "✗ Invalid"}
    </span>
  </div>
  <p>Run <code>frame build</code> to generate platform-specific icons.</p>
</body>
</html>`;
  } catch (error) {
    vscode.window.showErrorMessage(`Failed to show icon info: ${error}`);
  }
}

function getIconInfo(configPath: string, workspaceRoot: string): IconInfo[] {
  try {
    const content = fs.readFileSync(configPath, "utf-8");
    const config = JSON.parse(content);

    if (!config.icon) {
      // Using default
      return [
        {
          path: DEFAULT_ICON_PATH,
          format: "default",
          exists: true,
          isValid: true,
        },
      ];
    }

    const iconPath = path.join(workspaceRoot, config.icon);
    const exists = fs.existsSync(iconPath);
    const ext = path.extname(config.icon).toLowerCase();
    const format = getIconFormat(ext);
    const isValid = SUPPORTED_FORMATS.includes(ext);

    const info: IconInfo = {
      path: config.icon,
      format: format as any,
      exists,
      isValid: exists && isValid,
    };

    if (exists) {
      info.fileSize = fs.statSync(iconPath).size;
    }

    return [info];
  } catch {
    return [];
  }
}

function getIconFormat(ext: string): string {
  const lower = ext.toLowerCase();
  if (lower === ".svg") return "svg";
  if (lower === ".png") return "png";
  if (lower === ".jpg" || lower === ".jpeg") return "jpeg";
  return "default";
}

function findIconFiles(workspaceRoot: string): string[] {
  const iconFiles: string[] = [];

  // Search common icon directories
  const searchDirs = [
    "assets/icons",
    "assets/images",
    "src/assets/icons",
    "resources/icons",
  ];

  for (const dir of searchDirs) {
    const fullPath = path.join(workspaceRoot, dir);
    if (!fs.existsSync(fullPath)) continue;

    try {
      const files = fs.readdirSync(fullPath);
      for (const file of files) {
        const ext = path.extname(file).toLowerCase();
        if (SUPPORTED_FORMATS.includes(ext)) {
          iconFiles.push(path.join(fullPath, file));
        }
      }
    } catch {
      // Skip directories we can't read
    }
  }

  return iconFiles;
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

export function createIconCompletionItems(
  workspaceRoot: string | undefined,
): vscode.CompletionItem[] {
  const items: vscode.CompletionItem[] = [];

  // Suggest default icon
  const defaultItem = new vscode.CompletionItem(
    "$(symbol-file) Default Icon",
    vscode.CompletionItemKind.EnumMember,
  );
  defaultItem.detail = "Use Frame default lime green icon";
  defaultItem.insertText = "";
  defaultItem.documentation = new vscode.MarkdownString(
    `**Default Frame App Icon**\n\n- Color: Lime green (#BCFB70)\n- Size: 1080×1080 px\n- Format: SVG\n- Leave \`icon\` field empty or remove it to use default`,
  );
  items.push(defaultItem);

  if (!workspaceRoot) return items;

  // Find available custom icons
  const customIcons = findIconFiles(workspaceRoot);
  for (const file of customIcons) {
    const relativePath = path.relative(workspaceRoot, file);
    const fileName = path.basename(file);
    const ext = path.extname(file).toLowerCase();

    const item = new vscode.CompletionItem(
      fileName,
      vscode.CompletionItemKind.File,
    );
    item.detail = `Custom ${ext.substring(1).toUpperCase()} icon`;
    item.insertText = `"${relativePath}"`;
    item.documentation = new vscode.MarkdownString(
      `**${fileName}**\n\n- Path: \`${relativePath}\`\n- Size: ${formatFileSize(fs.statSync(file).size)}`,
    );
    items.push(item);
  }

  return items;
}
