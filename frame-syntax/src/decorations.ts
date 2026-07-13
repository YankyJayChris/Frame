import * as vscode from 'vscode';

const hexColorRegex = /"#([0-9A-Fa-f]{6}|[0-9A-Fa-f]{3})"/;
const dimensionUnitRegex = /(\d+(?:\.\d+)?)(dp|px|%|ms|sp|em|rem|vw|vh|deg)/g;

export function registerDecorations(context: vscode.ExtensionContext): vscode.Disposable[] {
  const disposables: vscode.Disposable[] = [];

  const colorSwatchDecoration = vscode.window.createTextEditorDecorationType({
    before: {
      contentText: '●',
      margin: '0 4px 0 0',
      width: '12px',
      height: '12px',
    },
    rangeBehavior: vscode.DecorationRangeBehavior.ClosedClosed,
  });

  const dimDecoration = vscode.window.createTextEditorDecorationType({
    opacity: '0.4',
    rangeBehavior: vscode.DecorationRangeBehavior.ClosedClosed,
  });

  function updateDecorations(editor: vscode.TextEditor | undefined) {
    if (!editor || editor.document.languageId !== 'frame') {
      return;
    }

    const text = editor.document.getText();
    const colorRanges: vscode.DecorationOptions[] = [];
    const dimRanges: vscode.DecorationOptions[] = [];

    const colorMatch = new RegExp(hexColorRegex.source, 'g');
    let match: RegExpExecArray | null;

    while ((match = colorMatch.exec(text)) !== null) {
      const startPos = editor.document.positionAt(match.index + 1);
      const endPos = editor.document.positionAt(match.index + match[0].length - 1);
      const range = new vscode.Range(startPos, endPos);

      const colorHex = match[1];

      colorRanges.push({
        range,
        renderOptions: {
          before: {
            color: `#${colorHex}`,
            contentText: '●',
          },
        },
      });
    }

    const dimMatch = new RegExp(dimensionUnitRegex.source, 'g');
    while ((match = dimMatch.exec(text)) !== null) {
      const unitStart = match.index + match[0].length - match[2].length;
      const startPos = editor.document.positionAt(unitStart);
      const endPos = editor.document.positionAt(match.index + match[0].length);
      dimRanges.push({
        range: new vscode.Range(startPos, endPos),
      });
    }

    editor.setDecorations(colorSwatchDecoration, colorRanges);
    editor.setDecorations(dimDecoration, dimRanges);
  }

  disposables.push(vscode.window.onDidChangeActiveTextEditor(updateDecorations));
  disposables.push(vscode.workspace.onDidChangeTextDocument((event) => {
    const editor = vscode.window.activeTextEditor;
    if (editor && event.document === editor.document) {
      updateDecorations(editor);
    }
  }));

  if (vscode.window.activeTextEditor) {
    updateDecorations(vscode.window.activeTextEditor);
  }

  disposables.push(colorSwatchDecoration);
  disposables.push(dimDecoration);

  return disposables;
}
