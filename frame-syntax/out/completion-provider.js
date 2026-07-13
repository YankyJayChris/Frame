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
exports.registerCompletionProviders = registerCompletionProviders;
const vscode = __importStar(require("vscode"));
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
const app_icon_manager_1 = require("./app-icon-manager");
const COMMON_ICONS = [
    { label: "plus", category: "Actions" },
    { label: "minus", category: "Actions" },
    { label: "checkmark", category: "Actions" },
    { label: "xmark", category: "Actions" },
    { label: "trash", category: "Actions" },
    { label: "pencil", category: "Actions" },
    { label: "plus.circle", category: "Actions" },
    { label: "checkmark.circle", category: "Actions" },
    { label: "xmark.circle", category: "Actions" },
    { label: "info", category: "Info" },
    { label: "info.circle", category: "Info" },
    { label: "questionmark", category: "Info" },
    { label: "questionmark.circle", category: "Info" },
    { label: "exclamationmark", category: "Info" },
    { label: "exclamationmark.triangle", category: "Info" },
    { label: "arrow.up", category: "Arrows" },
    { label: "arrow.down", category: "Arrows" },
    { label: "arrow.left", category: "Arrows" },
    { label: "arrow.right", category: "Arrows" },
    { label: "arrow.up.arrow.down", category: "Arrows" },
    { label: "chevron.up", category: "Arrows" },
    { label: "chevron.down", category: "Arrows" },
    { label: "chevron.left", category: "Arrows" },
    { label: "chevron.right", category: "Arrows" },
    { label: "magnifyingglass", category: "Search" },
    { label: "search", category: "Search" },
    { label: "slider.horizontal.3", category: "Search" },
    { label: "line.3.horizontal", category: "Navigation" },
    { label: "line.3.horizontal.decrease", category: "Navigation" },
    { label: "list.bullet", category: "Navigation" },
    { label: "square.grid.2x2", category: "Navigation" },
    { label: "square.grid.3x2", category: "Navigation" },
    { label: "rectangle.grid.1x2", category: "Navigation" },
    { label: "house", category: "Navigation" },
    { label: "house.fill", category: "Navigation" },
    { label: "person", category: "People" },
    { label: "person.2", category: "People" },
    { label: "person.circle", category: "People" },
    { label: "person.fill", category: "People" },
    { label: "star", category: "Rating" },
    { label: "star.fill", category: "Rating" },
    { label: "star.leadinghalf.filled", category: "Rating" },
    { label: "heart", category: "Rating" },
    { label: "heart.fill", category: "Rating" },
    { label: "gear", category: "Settings" },
    { label: "gearshape", category: "Settings" },
    { label: "bell", category: "Notifications" },
    { label: "bell.fill", category: "Notifications" },
    { label: "bell.badge", category: "Notifications" },
    { label: "envelope", category: "Communication" },
    { label: "envelope.fill", category: "Communication" },
    { label: "phone", category: "Communication" },
    { label: "phone.fill", category: "Communication" },
    { label: "message", category: "Communication" },
    { label: "message.fill", category: "Communication" },
    { label: "bubble.left", category: "Communication" },
    { label: "bubble.left.fill", category: "Communication" },
    { label: "cloud", category: "Status" },
    { label: "cloud.fill", category: "Status" },
    { label: "sun.max", category: "Weather" },
    { label: "sun.min", category: "Weather" },
    { label: "moon", category: "Weather" },
    { label: "moon.fill", category: "Weather" },
    { label: "play", category: "Media" },
    { label: "play.fill", category: "Media" },
    { label: "pause", category: "Media" },
    { label: "pause.fill", category: "Media" },
    { label: "stop", category: "Media" },
    { label: "stop.fill", category: "Media" },
    { label: "forward", category: "Media" },
    { label: "backward", category: "Media" },
    { label: "doc", category: "Documents" },
    { label: "doc.fill", category: "Documents" },
    { label: "folder", category: "Documents" },
    { label: "folder.fill", category: "Documents" },
    { label: "tray", category: "Documents" },
    { label: "tray.full", category: "Documents" },
    { label: "map", category: "Maps" },
    { label: "map.fill", category: "Maps" },
    { label: "location", category: "Maps" },
    { label: "location.fill", category: "Maps" },
    { label: "camera", category: "Media" },
    { label: "camera.fill", category: "Media" },
    { label: "photo", category: "Media" },
    { label: "photo.fill", category: "Media" },
    { label: "video", category: "Media" },
    { label: "video.fill", category: "Media" },
    { label: "mic", category: "Media" },
    { label: "mic.fill", category: "Media" },
    { label: "ellipsis", category: "Actions" },
    { label: "ellipsis.circle", category: "Actions" },
    { label: "square.and.arrow.up", category: "Actions" },
    { label: "square.and.arrow.down", category: "Actions" },
    { label: "link", category: "Actions" },
    { label: "qrcode", category: "Actions" },
    { label: "barcode", category: "Actions" },
    { label: "lock", category: "Security" },
    { label: "lock.fill", category: "Security" },
    { label: "lock.open", category: "Security" },
    { label: "eye", category: "Actions" },
    { label: "eye.slash", category: "Actions" },
    { label: "tag", category: "Actions" },
    { label: "tag.fill", category: "Actions" },
    { label: "bookmark", category: "Actions" },
    { label: "bookmark.fill", category: "Actions" },
    { label: "flag", category: "Actions" },
    { label: "flag.fill", category: "Actions" },
    { label: "clock", category: "Time" },
    { label: "clock.fill", category: "Time" },
    { label: "calendar", category: "Time" },
    { label: "calendar.badge.plus", category: "Time" },
    { label: "gift", category: "Objects" },
    { label: "gift.fill", category: "Objects" },
    { label: "cart", category: "Commerce" },
    { label: "cart.fill", category: "Commerce" },
    { label: "bag", category: "Commerce" },
    { label: "bag.fill", category: "Commerce" },
    { label: "creditcard", category: "Commerce" },
    { label: "creditcard.fill", category: "Commerce" },
    { label: "wifi", category: "Status" },
    { label: "antenna.radiowaves.left.and.right", category: "Status" },
    { label: "battery.100", category: "Status" },
    { label: "battery.25", category: "Status" },
];
const CSS_COLORS = {
    aliceblue: "#F0F8FF",
    antiquewhite: "#FAEBD7",
    aqua: "#00FFFF",
    aquamarine: "#7FFFD4",
    azure: "#F0FFFF",
    beige: "#F5F5DC",
    bisque: "#FFE4C4",
    black: "#000000",
    blanchedalmond: "#FFEBCD",
    blue: "#0000FF",
    blueviolet: "#8A2BE2",
    brown: "#A52A2A",
    burlywood: "#DEB887",
    cadetblue: "#5F9EA0",
    chartreuse: "#7FFF00",
    chocolate: "#D2691E",
    coral: "#FF7F50",
    cornflowerblue: "#6495ED",
    cornsilk: "#FFF8DC",
    crimson: "#DC143C",
    cyan: "#00FFFF",
    darkblue: "#00008B",
    darkcyan: "#008B8B",
    darkgoldenrod: "#B8860B",
    darkgray: "#A9A9A9",
    darkgreen: "#006400",
    darkgrey: "#A9A9A9",
    darkkhaki: "#BDB76B",
    darkmagenta: "#8B008B",
    darkolivegreen: "#556B2F",
    darkorange: "#FF8C00",
    darkorchid: "#9932CC",
    darkred: "#8B0000",
    darksalmon: "#E9967A",
    darkseagreen: "#8FBC8F",
    darkslateblue: "#483D8B",
    darkslategray: "#2F4F4F",
    darkturquoise: "#00CED1",
    darkviolet: "#9400D3",
    deeppink: "#FF1493",
    deepskyblue: "#00BFFF",
    dimgray: "#696969",
    dodgerblue: "#1E90FF",
    firebrick: "#B22222",
    floralwhite: "#FFFAF0",
    forestgreen: "#228B22",
    fuchsia: "#FF00FF",
    gainsboro: "#DCDCDC",
    ghostwhite: "#F8F8FF",
    gold: "#FFD700",
    goldenrod: "#DAA520",
    gray: "#808080",
    green: "#008000",
    greenyellow: "#ADFF2F",
    grey: "#808080",
    honeydew: "#F0FFF0",
    hotpink: "#FF69B4",
    indianred: "#CD5C5C",
    indigo: "#4B0082",
    ivory: "#FFFFF0",
    khaki: "#F0E68C",
    lavender: "#E6E6FA",
    lavenderblush: "#FFF0F5",
    lawngreen: "#7CFC00",
    lemonchiffon: "#FFFACD",
    lightblue: "#ADD8E6",
    lightcoral: "#F08080",
    lightcyan: "#E0FFFF",
    lightgoldenrodyellow: "#FAFAD2",
    lightgray: "#D3D3D3",
    lightgreen: "#90EE90",
    lightgrey: "#D3D3D3",
    lightpink: "#FFB6C1",
    lightsalmon: "#FFA07A",
    lightseagreen: "#20B2AA",
    lightskyblue: "#87CEFA",
    lightslategray: "#778899",
    lightsteelblue: "#B0C4DE",
    lightyellow: "#FFFFE0",
    lime: "#00FF00",
    limegreen: "#32CD32",
    linen: "#FAF0E6",
    magenta: "#FF00FF",
    maroon: "#800000",
    mediumaquamarine: "#66CDAA",
    mediumblue: "#0000CD",
    mediumorchid: "#BA55D3",
    mediumpurple: "#9370DB",
    mediumseagreen: "#3CB371",
    mediumslateblue: "#7B68EE",
    mediumspringgreen: "#00FA9A",
    mediumturquoise: "#48D1CC",
    mediumvioletred: "#C71585",
    midnightblue: "#191970",
    mintcream: "#F5FFFA",
    mistyrose: "#FFE4E1",
    moccasin: "#FFE4B5",
    navajowhite: "#FFDEAD",
    navy: "#000080",
    oldlace: "#FDF5E6",
    olive: "#808000",
    olivedrab: "#6B8E23",
    orange: "#FFA500",
    orangered: "#FF4500",
    orchid: "#DA70D6",
    palegoldenrod: "#EEE8AA",
    palegreen: "#98FB98",
    paleturquoise: "#AFEEEE",
    palevioletred: "#DB7093",
    papayawhip: "#FFEFD5",
    peachpuff: "#FFDAB9",
    peru: "#CD853F",
    pink: "#FFC0CB",
    plum: "#DDA0DD",
    powderblue: "#B0E0E6",
    purple: "#800080",
    rebeccapurple: "#663399",
    red: "#FF0000",
    rosybrown: "#BC8F8F",
    royalblue: "#4169E1",
    saddlebrown: "#8B4513",
    salmon: "#FA8072",
    sandybrown: "#F4A460",
    seagreen: "#2E8B57",
    seashell: "#FFF5EE",
    sienna: "#A0522D",
    silver: "#C0C0C0",
    skyblue: "#87CEEB",
    slateblue: "#6A5ACD",
    slategray: "#708090",
    snow: "#FFFAFA",
    springgreen: "#00FF7F",
    steelblue: "#4682B4",
    tan: "#D2B48C",
    teal: "#008080",
    thistle: "#D8BFD8",
    tomato: "#FF6347",
    turquoise: "#40E0D0",
    violet: "#EE82EE",
    wheat: "#F5DEB3",
    white: "#FFFFFF",
    whitesmoke: "#F5F5F5",
    yellow: "#FFFF00",
    yellowgreen: "#9ACD32",
};
class IconCompletionProvider {
    async provideInlineCompletionItems(document, position, _context, _token) {
        const line = document.lineAt(position.line).text;
        const lineBefore = line.substring(0, position.character);
        const iconMatch = lineBefore.match(/name:\s*"([^"]*)$/);
        if (!iconMatch)
            return undefined;
        const partial = iconMatch[1].toLowerCase();
        if (partial.length === 0)
            return undefined;
        const results = [];
        for (const icon of COMMON_ICONS) {
            if (icon.label.toLowerCase().startsWith(partial)) {
                const range = new vscode.Range(position.line, position.character - partial.length, position.line, position.character);
                results.push({
                    insertText: icon.label,
                    filterText: icon.label,
                    range,
                });
                if (results.length >= 20)
                    break;
            }
        }
        return results;
    }
}
class ColorCompletionProvider {
    async provideInlineCompletionItems(document, position, _context, _token) {
        const line = document.lineAt(position.line).text;
        const lineBefore = line.substring(0, position.character);
        const colorMatch = lineBefore.match(/(?:color|background|border_color):\s*"([^"]*)$/);
        if (!colorMatch)
            return undefined;
        const partial = colorMatch[1].toLowerCase();
        if (partial.length === 0)
            return undefined;
        const results = [];
        for (const [name, hex] of Object.entries(CSS_COLORS)) {
            if (name.startsWith(partial)) {
                const range = new vscode.Range(position.line, position.character - partial.length, position.line, position.character);
                results.push({
                    insertText: name,
                    filterText: name,
                    range,
                });
                if (results.length >= 30)
                    break;
            }
        }
        return results;
    }
}
class FilePathCompletionProvider {
    async provideInlineCompletionItems(document, position, _context, _token) {
        const line = document.lineAt(position.line).text;
        const lineBefore = line.substring(0, position.character);
        const importMatch = lineBefore.match(/import\s*\{[^}]*\}\s*"([^"]*)$/);
        if (!importMatch)
            return undefined;
        const partial = importMatch[1];
        const dir = path.dirname(document.uri.fsPath);
        const searchDir = partial.includes("/")
            ? path.resolve(dir, partial.substring(0, partial.lastIndexOf("/")))
            : dir;
        let searchPrefix = partial.includes("/")
            ? partial.substring(partial.lastIndexOf("/") + 1)
            : partial;
        if (!fs.existsSync(searchDir))
            return undefined;
        const results = [];
        try {
            const entries = fs.readdirSync(searchDir);
            for (const entry of entries) {
                if (entry.endsWith(".fr") &&
                    entry.toLowerCase().startsWith(searchPrefix.toLowerCase())) {
                    const insertName = partial.includes("/")
                        ? `${partial.substring(0, partial.lastIndexOf("/") + 1)}${entry.replace(/\.fr$/, "")}`
                        : entry.replace(/\.fr$/, "");
                    const range = new vscode.Range(position.line, position.character - partial.length, position.line, position.character);
                    results.push({
                        insertText: insertName,
                        filterText: insertName,
                        range,
                    });
                }
            }
        }
        catch { }
        return results.length > 0 ? results : undefined;
    }
}
class RouteCompletionProvider {
    async provideInlineCompletionItems(document, position, _context, _token) {
        const line = document.lineAt(position.line).text;
        const lineBefore = line.substring(0, position.character);
        const routeMatch = lineBefore.match(/navigate\(\s*"([^"]*)$/);
        if (!routeMatch)
            return undefined;
        const partial = routeMatch[1].toLowerCase();
        const pageRouteRegex = /page:\s*\{[^}]*route:\s*"([^"]+)"/g;
        const routes = [];
        const files = await vscode.workspace.findFiles("**/*.fr");
        for (const file of files) {
            try {
                const content = fs.readFileSync(file.fsPath, "utf-8");
                let match;
                while ((match = pageRouteRegex.exec(content)) !== null) {
                    routes.push(match[1]);
                }
            }
            catch { }
        }
        const uniqueRoutes = [...new Set(routes)];
        const results = [];
        for (const route of uniqueRoutes) {
            if (route.toLowerCase().startsWith(partial)) {
                const range = new vscode.Range(position.line, position.character - partial.length, position.line, position.character);
                results.push({
                    insertText: route,
                    filterText: route,
                    range,
                });
            }
        }
        return results.length > 0 ? results : undefined;
    }
}
class AppIconCompletionProvider {
    async provideInlineCompletionItems(document, position, _context, _token) {
        // Only provide completions for frame.config.json
        if (!document.fileName.endsWith("frame.config.json")) {
            return undefined;
        }
        const line = document.lineAt(position.line).text;
        const lineBefore = line.substring(0, position.character);
        // Match: "icon": "..."
        const iconMatch = lineBefore.match(/"icon"\s*:\s*"([^"]*)$/);
        if (!iconMatch)
            return undefined;
        const partial = iconMatch[1];
        const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
        const completionItems = (0, app_icon_manager_1.createIconCompletionItems)(workspaceRoot);
        const results = [];
        for (const item of completionItems) {
            const itemLabel = typeof item.label === "string" ? item.label : item.label.label;
            const itemInsertText = typeof item.insertText === "string" ? item.insertText : undefined;
            if (itemLabel.toLowerCase().includes(partial.toLowerCase()) ||
                (itemInsertText &&
                    itemInsertText.toLowerCase().includes(partial.toLowerCase()))) {
                const insertText = itemInsertText || itemLabel;
                const range = new vscode.Range(position.line, position.character - partial.length, position.line, position.character);
                results.push({
                    insertText,
                    filterText: itemLabel,
                    range,
                });
            }
        }
        return results.length > 0 ? results : undefined;
    }
}
function registerCompletionProviders(context) {
    const disposables = [];
    disposables.push(vscode.languages.registerInlineCompletionItemProvider("frame", new IconCompletionProvider()));
    disposables.push(vscode.languages.registerInlineCompletionItemProvider("frame", new ColorCompletionProvider()));
    disposables.push(vscode.languages.registerInlineCompletionItemProvider("frame", new FilePathCompletionProvider()));
    disposables.push(vscode.languages.registerInlineCompletionItemProvider("frame", new RouteCompletionProvider()));
    disposables.push(vscode.languages.registerInlineCompletionItemProvider(["frame", "json"], new AppIconCompletionProvider()));
    return disposables;
}
//# sourceMappingURL=completion-provider.js.map