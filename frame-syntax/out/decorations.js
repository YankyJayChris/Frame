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
exports.registerDecorations = registerDecorations;
const vscode = __importStar(require("vscode"));
const hexColorRegex = /"#([0-9A-Fa-f]{6}|[0-9A-Fa-f]{3})"/;
const dimensionUnitRegex = /(\d+(?:\.\d+)?)(dp|px|%|ms|sp|em|rem|vw|vh|deg)/g;
function registerDecorations(context) {
    const disposables = [];
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
    function updateDecorations(editor) {
        if (!editor || editor.document.languageId !== 'frame') {
            return;
        }
        const text = editor.document.getText();
        const colorRanges = [];
        const dimRanges = [];
        const colorMatch = new RegExp(hexColorRegex.source, 'g');
        let match;
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
//# sourceMappingURL=decorations.js.map