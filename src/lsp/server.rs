use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use lsp_types::*;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use crate::parser::{self, AST, FrameError};
use crate::cli::lint::{self, LintConfig, LintDiagnostic};
use crate::compiler::registry::{registry, BuiltInComponentDef};

use super::capabilities::server_capabilities;
use super::diagnostics::publish_diagnostics_for_file;

/// The Frame Language Server.
pub struct FrameLanguageServer {
    client: Client,
    workspace_root: Arc<Mutex<Option<PathBuf>>>,
    ast: Arc<Mutex<Option<AST>>>,
    open_files: Arc<Mutex<HashMap<Url, String>>>,
}

impl FrameLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            workspace_root: Arc::new(Mutex::new(None)),
            ast: Arc::new(Mutex::new(None)),
            open_files: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn find_project_root(start: &Path) -> Option<PathBuf> {
        let mut current = Some(start.to_path_buf());
        while let Some(ref dir) = current {
            if dir.join("frame.config.json").exists()
                || dir.join("src").join("project.fr").exists()
            {
                return Some(dir.clone());
            }
            current = dir.parent().map(|p| p.to_path_buf());
        }
        None
    }

    async fn rebuild_project(&self) {
        let root = {
            let guard = self.workspace_root.lock().await;
            guard.clone()
        };

        let root_clone = root.clone();
        let result = tokio::task::spawn_blocking(move || {
            let root = root_clone?;
            let root_str = root.to_string_lossy().to_string();

            let (ast, parse_errors) = match parser::parse_project(&root_str) {
                Ok(ast) => (ast, vec![]),
                Err(errs) => (AST::default(), errs),
            };

            let lint_config = LintConfig::default();
            let mut lint_diags: Vec<LintDiagnostic> = Vec::new();
            lint::lint_naming(&ast, &lint_config, &mut lint_diags);
            lint::lint_style(&ast, &lint_config, &mut lint_diags);
            lint::lint_complexity(&ast, &lint_config, &mut lint_diags);
            lint::lint_performance(&ast, &lint_config, &mut lint_diags);
            lint::lint_best_practice(&ast, &lint_config, &mut lint_diags);

            Some((ast, parse_errors, lint_diags))
        })
        .await
        .ok()
        .flatten();

        if let Some((ast, parse_errors, lint_diags)) = result {
            {
                let mut cached = self.ast.lock().await;
                *cached = Some(ast);
            }

            let open_files = {
                let guard = self.open_files.lock().await;
                guard.keys().cloned().collect::<Vec<_>>()
            };

            for file_url in open_files {
                let file_errors: Vec<FrameError> = parse_errors
                    .iter()
                    .filter(|e| {
                        if let Ok(p) = file_url.to_file_path() {
                            e.file == p.to_string_lossy().to_string()
                        } else {
                            false
                        }
                    })
                    .cloned()
                    .collect();

                let file_lint: Vec<LintDiagnostic> = lint_diags
                    .iter()
                    .filter(|d| {
                        if let Ok(p) = file_url.to_file_path() {
                            d.context.contains(&p.to_string_lossy().to_string())
                        } else {
                            false
                        }
                    })
                    .cloned()
                    .collect();

                let diags = publish_diagnostics_for_file(&file_url, &file_errors, &file_lint);
                self.client.publish_diagnostics(file_url, diags, None).await;
            }
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for FrameLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let root = params
            .root_uri
            .and_then(|uri| uri.to_file_path().ok())
            .or_else(|| {
                params
                    .workspace_folders
                    .and_then(|f| f.first().cloned())
                    .and_then(|f| f.uri.to_file_path().ok())
            })
            .or_else(|| std::env::current_dir().ok());

        if let Some(ref root_path) = root {
            let resolved =
                Self::find_project_root(root_path).unwrap_or_else(|| root_path.clone());
            {
                let mut ws = self.workspace_root.lock().await;
                *ws = Some(resolved);
            }
        }

        Ok(InitializeResult {
            capabilities: server_capabilities(),
            server_info: Some(ServerInfo {
                name: "frame-lsp".into(),
                version: Some("0.1.0".into()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Frame LSP initialized")
            .await;
        self.rebuild_project().await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(MessageType::INFO, "Frame LSP shutting down")
            .await;
        Ok(())
    }

    // ── Text Document Sync ──────────────────────────────────────────────

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        {
            let mut files = self.open_files.lock().await;
            files.insert(uri, params.text_document.text);
        }
        self.rebuild_project().await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().last() {
            {
                let mut files = self.open_files.lock().await;
                files.insert(uri.clone(), change.text);
            }
            self.rebuild_project().await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.rebuild_project().await;
        if let Ok(path) = params.text_document.uri.to_file_path() {
            self.client
                .log_message(MessageType::INFO, format!("Saved: {}", path.display()))
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        {
            let mut files = self.open_files.lock().await;
            files.remove(&uri);
        }
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    // ── Completion ─────────────────────────────────────────────────────

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        let position = params.text_document_position.position;
        let uri = params.text_document_position.text_document.uri;

        let text = {
            let files = self.open_files.lock().await;
            files.get(&uri).cloned()
        };
        let ast = {
            let cached = self.ast.lock().await;
            cached.clone()
        };

        if let (Some(text), Some(ast)) = (text, ast) {
            let items = compute_completions(&text, position, &ast);
            return Ok(Some(CompletionResponse::Array(items)));
        }
        Ok(Some(CompletionResponse::Array(vec![])))
    }

    // ── Hover ──────────────────────────────────────────────────────────

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let pos = params.text_document_position_params;
        let uri = pos.text_document.uri;
        let position = pos.position;

        let text = {
            let files = self.open_files.lock().await;
            files.get(&uri).cloned()
        };
        let ast = {
            let cached = self.ast.lock().await;
            cached.clone()
        };

        if let (Some(text), Some(ast)) = (text, ast) {
            return Ok(compute_hover(&text, position, &ast));
        }
        Ok(None)
    }

    // ── Go to Definition ───────────────────────────────────────────────

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let pos = params.text_document_position_params;
        let uri = pos.text_document.uri;
        let position = pos.position;

        let text = {
            let files = self.open_files.lock().await;
            files.get(&uri).cloned()
        };
        let ast = {
            let cached = self.ast.lock().await;
            cached.clone()
        };

        if let (Some(text), Some(ast)) = (text, ast) {
            return Ok(compute_definition(&text, position, &ast, &uri));
        }
        Ok(None)
    }

    // ── References ─────────────────────────────────────────────────────

    async fn references(
        &self,
        params: ReferenceParams,
    ) -> Result<Option<Vec<Location>>> {
        let pos = params.text_document_position;
        let uri = pos.text_document.uri;
        let position = pos.position;

        let text = {
            let files = self.open_files.lock().await;
            files.get(&uri).cloned()
        };

        if let Some(text) = text {
            let highlights = compute_document_highlights(&text, position);
            let locations: Vec<Location> = highlights
                .into_iter()
                .map(|h| Location {
                    uri: uri.clone(),
                    range: h.range,
                })
                .collect();
            return Ok(Some(locations));
        }

        Ok(Some(vec![]))
    }

    // ── Document Symbols ───────────────────────────────────────────────

    async fn document_symbol(
        &self,
        _params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let ast = { self.ast.lock().await.clone() };
        if let Some(ast) = ast {
            let symbols = compute_document_symbols(&ast);
            return Ok(Some(DocumentSymbolResponse::Nested(symbols)));
        }
        Ok(None)
    }

    // ── Workspace Symbols ──────────────────────────────────────────────

    async fn symbol(
        &self,
        _params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let ast = { self.ast.lock().await.clone() };
        if let Some(ast) = ast {
            let symbols = compute_workspace_symbols(&ast);
            return Ok(Some(symbols));
        }
        Ok(None)
    }

    // ── Code Actions ───────────────────────────────────────────────────

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<CodeActionResponse>> {
        let ast = { self.ast.lock().await.clone() };
        let uri = params.text_document.uri;
        let text = {
            let files = self.open_files.lock().await;
            files.get(&uri).cloned()
        };

        if let (Some(text), Some(ast)) = (text, ast) {
            let actions = compute_code_actions(&text, params.range, &ast, &uri);
            return Ok(Some(actions));
        }
        Ok(None)
    }

    // ── Formatting ─────────────────────────────────────────────────────

    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let text = {
            let files = self.open_files.lock().await;
            files.get(&params.text_document.uri).cloned()
        };
        if let Some(text) = text {
            let edits = compute_formatting(&text);
            return Ok(Some(edits));
        }
        Ok(None)
    }

    // ── Folding Ranges ─────────────────────────────────────────────────

    async fn folding_range(
        &self,
        params: FoldingRangeParams,
    ) -> Result<Option<Vec<FoldingRange>>> {
        let text = {
            let files = self.open_files.lock().await;
            files.get(&params.text_document.uri).cloned()
        };
        if let Some(text) = text {
            let ranges = compute_folding_ranges(&text);
            return Ok(Some(ranges));
        }
        Ok(None)
    }

    // ── Semantic Tokens ────────────────────────────────────────────────

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let text = {
            let files = self.open_files.lock().await;
            files.get(&params.text_document.uri).cloned()
        };
        let ast = { self.ast.lock().await.clone() };

        if let (Some(text), Some(ast)) = (text, ast) {
            let data = compute_semantic_tokens(&text, &ast);
            return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data,
            })));
        }
        Ok(None)
    }

    // ── Document Highlight ─────────────────────────────────────────────

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        let pos = params.text_document_position_params;
        let uri = pos.text_document.uri;
        let position = pos.position;

        let text = {
            let files = self.open_files.lock().await;
            files.get(&uri).cloned()
        };
        if let Some(text) = text {
            let highlights = compute_document_highlights(&text, position);
            return Ok(Some(highlights));
        }
        Ok(None)
    }

    // ── Rename ─────────────────────────────────────────────────────────

    async fn rename(
        &self,
        params: RenameParams,
    ) -> Result<Option<WorkspaceEdit>> {
        let pos = params.text_document_position;
        let uri = pos.text_document.uri;
        let position = pos.position;
        let new_name = params.new_name;

        let text = {
            let files = self.open_files.lock().await;
            files.get(&uri).cloned()
        };
        let ast = { self.ast.lock().await.clone() };

        if let (Some(text), Some(ast)) = (text, ast) {
            return Ok(Some(compute_rename(&text, position, &new_name, &ast, &uri)));
        }
        Ok(None)
    }

    // ── Document Color ─────────────────────────────────────────────────

    async fn document_color(
        &self,
        params: DocumentColorParams,
    ) -> Result<Vec<ColorInformation>> {
        let text = {
            let files = self.open_files.lock().await;
            files.get(&params.text_document.uri).cloned()
        };
        if let Some(text) = text {
            return Ok(compute_document_colors(&text));
        }
        Ok(vec![])
    }

    async fn color_presentation(
        &self,
        params: ColorPresentationParams,
    ) -> Result<Vec<ColorPresentation>> {
        let c = params.color;
        Ok(vec![ColorPresentation {
            label: format!(
                "#{:02X}{:02X}{:02X}",
                (c.red * 255.0) as u8,
                (c.green * 255.0) as u8,
                (c.blue * 255.0) as u8
            ),
            ..Default::default()
        }])
    }
}

// ─── Run LSP Server ───────────────────────────────────────────────────

pub async fn run_lsp_server(workspace_root: PathBuf) {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| {
        let server = FrameLanguageServer::new(client);
        {
            let ws = server.workspace_root.clone();
            tokio::spawn(async move {
                let mut root = ws.lock().await;
                *root = Some(workspace_root);
            });
        }
        server
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}

// ============================================================================
// COMPLETIONS
// ============================================================================

fn compute_completions(text: &str, position: Position, ast: &AST) -> Vec<CompletionItem> {
    let line = position.line as usize;
    let char_pos = position.character as usize;
    let lines: Vec<&str> = text.lines().collect();
    if line >= lines.len() {
        return vec![];
    }

    let current_line = lines[line];
    let up_to_cursor = &current_line[..char_pos.min(current_line.len())];

    if is_top_level_context(up_to_cursor, &lines, line) {
        return top_level_completions();
    }
    if is_inside_children(up_to_cursor, &lines, line) {
        return builtin_component_completions(ast);
    }
    if let Some(kind) = component_kind_at_cursor(text, position) {
        return component_prop_completions(&kind);
    }
    if is_inside_styles(up_to_cursor, &lines, line) {
        return style_prop_completions();
    }
    if up_to_cursor.trim().starts_with("import {") || up_to_cursor.trim().starts_with("import{") {
        return import_name_completions(ast);
    }
    if up_to_cursor.ends_with('$') || up_to_cursor.ends_with(" $") {
        return vars_token_completions(ast);
    }
    if up_to_cursor.ends_with("wait:") || up_to_cursor.ends_with(" wait:") {
        return wait_completions(ast);
    }
    if let Some(store_name) = store_dot_prefix(up_to_cursor) {
        return store_field_completions(&store_name, ast);
    }
    if up_to_cursor.ends_with('@') {
        return breakpoint_completions(ast);
    }

    vec![]
}

fn is_top_level_context(up_to_cursor: &str, _lines: &[&str], _line: usize) -> bool {
    let trimmed = up_to_cursor.trim();
    if trimmed.is_empty() || trimmed == "{" || trimmed == "}" {
        return true;
    }
    let depth = brace_depth(up_to_cursor);
    depth == 0
        && !up_to_cursor.contains("children:")
        && !up_to_cursor.contains("styles:")
        && !up_to_cursor.contains("props:")
}

fn brace_depth(s: &str) -> i32 {
    let mut depth = 0i32;
    for c in s.chars() {
        match c {
            '{' => depth += 1,
            '}' if depth > 0 => depth -= 1,
            _ => {}
        }
    }
    depth
}

fn is_inside_children(up_to_cursor: &str, _lines: &[&str], _line: usize) -> bool {
    let before = match up_to_cursor.rfind("children:") {
        Some(pos) => &up_to_cursor[pos..],
        None => return false,
    };
    if let Some(bpos) = before.find('[') {
        brace_depth(&before[bpos..]) > 0
    } else {
        true
    }
}

fn is_inside_styles(up_to_cursor: &str, _lines: &[&str], _line: usize) -> bool {
    if let Some(pos) = up_to_cursor.rfind("styles:") {
        let after = &up_to_cursor[pos..];
        if let Some(bp) = after.find('{') {
            return brace_depth(&after[bp..]) > 0;
        }
    }
    false
}

fn component_kind_at_cursor(text: &str, position: Position) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    let line = position.line as usize;
    if line >= lines.len() {
        return None;
    }
    let mut brace_count = 0i32;
    for l in (0..=line).rev() {
        let t = lines[l];
        if let Some(colon) = t.find(':') {
            let kind = t[..colon].trim();
            if !kind.is_empty() && kind.chars().all(|c| c.is_alphanumeric() || c == '_') {
                let after_colon = &t[colon + 1..];
                if after_colon.contains('{') {
                    for fl in l..=line {
                        for c in lines[fl].chars() {
                            match c {
                                '{' => brace_count += 1,
                                '}' => brace_count -= 1,
                                _ => {}
                            }
                        }
                        if brace_count <= 0 && fl < line {
                            break;
                        }
                    }
                    if brace_count > 0 {
                        return Some(kind.to_string());
                    }
                }
            }
        }
    }
    None
}

fn store_dot_prefix(up_to_cursor: &str) -> Option<String> {
    let trimmed = up_to_cursor.trim();
    if let Some(dot) = trimmed.rfind('.') {
        if dot > 0 {
            let before = trimmed[..dot].trim();
            if !before.is_empty()
                && before.chars().next().map_or(false, |c| c.is_ascii_uppercase())
            {
                let words: Vec<&str> = before.split_whitespace().collect();
                if let Some(last) = words.last() {
                    if last.chars().next().map_or(false, |c| c.is_ascii_uppercase()) {
                        return Some(last.to_string());
                    }
                }
            }
        }
    }
    None
}

// ─── Completion Builders ──────────────────────────────────────────────

fn completion(
    label: &str,
    insert_text: &str,
    detail: &str,
    kind: CompletionItemKind,
) -> CompletionItem {
    CompletionItem {
        label: label.into(),
        kind: Some(kind),
        detail: Some(detail.into()),
        insert_text: Some(insert_text.into()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    }
}

fn top_level_completions() -> Vec<CompletionItem> {
    vec![
        completion("page:", "page: {\n\tname: \"${1:Name}\"\n\troute: \"/${2:route}\"\n\tchildren: [\n\t\t$0\n\t]\n}", "Create a page with route and children", CompletionItemKind::KEYWORD),
        completion("component", "component ${1:Name}: {\n\tprops: {\n\t\t$0\n\t}\n}", "Define a reusable component", CompletionItemKind::KEYWORD),
        completion("fn", "fn ${1:name}: ${2:async }($3) => {\n\t$0\n}", "Define a function", CompletionItemKind::KEYWORD),
        completion("import", "import { ${1:Name} } \"${2:module}\"", "Import from a module", CompletionItemKind::KEYWORD),
        completion(":store", ":store ${1:Name} {\n\t${2:field}: ${3:string} = ${4:\"\"}\n\t$0\n}", "Create a state store", CompletionItemKind::KEYWORD),
        completion(":obj", ":obj ${1:Name} {\n\t${2:field}: ${3:string}\n}", "Define an object type", CompletionItemKind::KEYWORD),
        completion(":enum", ":enum ${1:Name} {\n\t${2:VARIANT}\n}", "Define an enum", CompletionItemKind::KEYWORD),
        completion(":type", ":type ${1:Alias} = ${2:string}", "Define a type alias", CompletionItemKind::KEYWORD),
        completion(":vars", ":vars {\n\t${1:\\$primary}: \"${2:#007BFF}\"\n}", "Design tokens", CompletionItemKind::KEYWORD),
        completion(":i18n", ":i18n {\n\t${1:key}: \"${2:value}\"\n}", "Internationalization", CompletionItemKind::KEYWORD),
        completion(":validation", ":validation ${1:Schema} {\n\t${2:field}: ${3:required}\n}", "Validation rules", CompletionItemKind::KEYWORD),
        completion(":breakpoints", ":breakpoints {\n\tsm: 360dp\n\tmd: 600dp\n\tlg: 900dp\n}", "Responsive breakpoints", CompletionItemKind::KEYWORD),
        completion(":typography", ":typography {\n\t${1:headline}: {\n\t\tfont_size: ${2:24}sp\n\t}\n}", "Typography styles", CompletionItemKind::KEYWORD),
        completion(":app", ":app {\n\ton_launch: ${1:fn}\n}", "App lifecycle hooks", CompletionItemKind::KEYWORD),
        completion("const", "const ${1:name} = ${2:value}", "Declare a constant", CompletionItemKind::KEYWORD),
        completion("describe:", "describe: \"${1:Suite}\" => {\n\tit: \"${2:test case}\" => {\n\t\t$0\n\t}\n}", "Test suite", CompletionItemKind::KEYWORD),
    ]
}

fn builtin_component_completions(ast: &AST) -> Vec<CompletionItem> {
    let mut items: Vec<CompletionItem> = registry()
        .names()
        .into_iter()
        .map(|name| {
            let detail = registry()
                .get(name)
                .map(|c| format!("{:?} component", c.category))
                .unwrap_or_default();
            completion(name, &format!("{}: {{\n\t$0\n}}", name), &detail, CompletionItemKind::KEYWORD)
        })
        .collect();

    for comp_name in ast.components.keys() {
        items.push(completion(
            comp_name,
            &format!("{}: {{\n\t$0\n}}", comp_name),
            "User-defined component",
            CompletionItemKind::CLASS,
        ));
    }

    items.sort_by(|a, b| a.label.cmp(&b.label));
    items
}

fn component_prop_completions(kind: &str) -> Vec<CompletionItem> {
    let reg = registry();
    if let Some(def) = reg.get(kind) {
        let mut items: Vec<CompletionItem> = def
            .props
            .iter()
            .map(|p| {
                let detail = if p.required {
                    format!("{} (required)", p.kind)
                } else {
                    format!("{} (optional, default: {})", p.kind, p.default.unwrap_or("-"))
                };
                completion(p.name, &format!("{}: ", p.name), &detail, CompletionItemKind::PROPERTY)
            })
            .collect();

        for event in &def.events {
            items.push(completion(
                event,
                &format!("{}: ", event),
                "Event handler",
                CompletionItemKind::EVENT,
            ));
        }

        if def.allows_children {
            items.push(completion(
                "children",
                "children: [\n\t$0\n]",
                "Child components",
                CompletionItemKind::KEYWORD,
            ));
        }

        items
    } else {
        vec![
            completion("props:", "props: {\n\t$0\n}", "Component props", CompletionItemKind::KEYWORD),
            completion("state:", "state: {\n\t$0\n}", "Component state", CompletionItemKind::KEYWORD),
            completion("styles:", "styles: {\n\t$0\n}", "Component styles", CompletionItemKind::KEYWORD),
            completion("children:", "children: [\n\t$0\n]", "Child components", CompletionItemKind::KEYWORD),
        ]
    }
}

fn style_prop_completions() -> Vec<CompletionItem> {
    let props = vec![
        ("safe_area", "bool", "Respect safe area insets"),
        ("width", "dimension", "Width"),
        ("height", "dimension", "Height"),
        ("min_width", "dimension", "Minimum width"),
        ("max_width", "dimension", "Maximum width"),
        ("min_height", "dimension", "Minimum height"),
        ("max_height", "dimension", "Maximum height"),
        ("padding", "dimension", "Padding on all sides"),
        ("padding_top", "dimension", "Top padding"),
        ("padding_bottom", "dimension", "Bottom padding"),
        ("padding_start", "dimension", "Start padding"),
        ("padding_end", "dimension", "End padding"),
        ("margin", "dimension", "Margin on all sides"),
        ("margin_top", "dimension", "Top margin"),
        ("margin_bottom", "dimension", "Bottom margin"),
        ("margin_start", "dimension", "Start margin"),
        ("margin_end", "dimension", "End margin"),
        ("background", "string", "Background color"),
        ("color", "string", "Text color"),
        ("border_radius", "dimension", "Corner radius"),
        ("border_width", "dimension", "Border width"),
        ("border_color", "string", "Border color"),
        ("font_size", "dimension(sp)", "Font size"),
        ("font_weight", "string", "Font weight (normal, bold, light)"),
        ("font_family", "string", "Font family name"),
        ("font_style", "string", "Font style (normal, italic)"),
        ("text_align", "string", "Text alignment"),
        ("line_height", "dimension", "Line height"),
        ("letter_spacing", "dimension", "Letter spacing"),
        ("opacity", "float (0-1)", "Opacity"),
        ("elevation", "float", "Shadow elevation"),
        ("shadow_color", "string", "Shadow color"),
        ("align", "string", "Child alignment"),
        ("justify", "string", "Justify content"),
        ("gap", "dimension", "Gap between children"),
        ("flex", "float", "Flex grow factor"),
        ("overflow", "string", "Overflow behavior"),
        ("text_overflow", "string", "Text overflow"),
        ("max_lines", "int", "Max text lines"),
        ("line_clamp", "int", "Line clamp"),
        ("scroll_indicator", "bool", "Show scroll indicator"),
        ("scroll_enabled", "bool", "Enable scrolling"),
        ("clip_behavior", "string", "Clip behavior"),
        ("fit", "string", "Image fit mode"),
        ("position", "string", "Position type"),
        ("top", "dimension", "Top position"),
        ("bottom", "dimension", "Bottom position"),
        ("left", "dimension", "Left position"),
        ("right", "dimension", "Right position"),
        ("z_index", "int", "Z-index"),
        ("rotate", "float", "Rotation degrees"),
        ("scale", "float", "Scale factor"),
        ("aspect_ratio", "float", "Aspect ratio"),
    ];

    props.into_iter()
        .map(|(n, k, d)| completion(n, &format!("{}: ", n), &format!("{} — {}", k, d), CompletionItemKind::PROPERTY))
        .collect()
}

fn import_name_completions(ast: &AST) -> Vec<CompletionItem> {
    let mut items: Vec<CompletionItem> = registry()
        .names()
        .iter()
        .filter_map(|name| {
            registry().get(name).map(|def| {
                completion(name, name, &format!("{:?} — frame-core", def.category), CompletionItemKind::CLASS)
            })
        })
        .collect();

    for name in ast.components.keys() {
        items.push(completion(name, name, "User component", CompletionItemKind::CLASS));
    }
    for name in ast.stores.keys() {
        items.push(completion(name, name, "Store", CompletionItemKind::CLASS));
    }
    for name in ast.objects.keys() {
        items.push(completion(name, name, "Object type", CompletionItemKind::STRUCT));
    }
    for name in ast.enums.keys() {
        items.push(completion(name, name, "Enum", CompletionItemKind::ENUM));
    }

    items.sort_by(|a, b| a.label.cmp(&b.label));
    items.dedup_by(|a, b| a.label == b.label);
    items
}

fn vars_token_completions(ast: &AST) -> Vec<CompletionItem> {
    ast.vars
        .keys()
        .map(|name| {
            let val = ast.vars.get(name).cloned().unwrap_or_default();
            completion(name, &format!("${}", name), &format!("Token: {}", val), CompletionItemKind::CONSTANT)
        })
        .collect()
}

fn wait_completions(ast: &AST) -> Vec<CompletionItem> {
    let mut items = vec![completion(
        "fetch(",
        "fetch(\"${1:url}\", {\n\tmethod: \"${2|GET,POST|}\"\n})",
        "HTTP fetch request",
        CompletionItemKind::FUNCTION,
    )];

    for (sname, store) in &ast.stores {
        for fname in store.actions.keys() {
            let label = format!("{}.{}", sname, fname);
            items.push(completion(
                &label,
                &format!("{}.{}($0)", sname, fname),
                "Store action",
                CompletionItemKind::METHOD,
            ));
        }
    }

    items
}

fn store_field_completions(store_name: &str, ast: &AST) -> Vec<CompletionItem> {
    let mut items = vec![];
    if let Some(store) = ast.stores.get(store_name) {
        for fname in store.fields.keys() {
            items.push(completion(fname, fname, "Field", CompletionItemKind::FIELD));
        }
        for fname in store.actions.keys() {
            items.push(completion(
                fname,
                &format!("{}(", fname),
                "Action",
                CompletionItemKind::METHOD,
            ));
        }
    }
    items
}

fn breakpoint_completions(ast: &AST) -> Vec<CompletionItem> {
    ast.breakpoints
        .iter()
        .map(|bp| {
            completion(
                &bp.name,
                &format!("@{} ", bp.name),
                &format!("Breakpoint: {}dp", bp.min_width_dp),
                CompletionItemKind::CONSTANT,
            )
        })
        .collect()
}

// ============================================================================
// HOVER
// ============================================================================

fn compute_hover(text: &str, position: Position, _ast: &AST) -> Option<Hover> {
    let lines: Vec<&str> = text.lines().collect();
    let line = position.line as usize;
    if line >= lines.len() {
        return None;
    }
    let cl = lines[line];
    let cp = position.character as usize;
    if cp >= cl.len() {
        return None;
    }

    let word = extract_word_at(cl, cp)?;
    let lower = word.to_lowercase();

    if let Some(def) = registry().get(&lower) {
        return Some(builtin_component_hover(def));
    }
    if let Some(docs) = get_style_docs(&word) {
        return Some(hover_text(&docs));
    }
    if word.starts_with("on_") {
        return Some(hover_text(&format!(
            "**{}**\n\nEvent handler. Assign a function or lambda.\n\n```\n{}: handlerName\n{}: () => {{ }}\n```",
            word, word, word
        )));
    }
    if let Some(docs) = get_keyword_docs(&word) {
        return Some(hover_text(&docs));
    }

    None
}

fn extract_word_at(line: &str, char_pos: usize) -> Option<String> {
    if char_pos >= line.len() {
        return None;
    }
    let bytes = line.as_bytes();
    let mut s = char_pos;
    let mut e = char_pos;
    while s > 0 {
        let c = bytes[s - 1] as char;
        if c.is_alphanumeric() || c == '_' || c == ':' || c == '$' {
            s -= 1;
        } else {
            break;
        }
    }
    while e < bytes.len() {
        let c = bytes[e] as char;
        if c.is_alphanumeric() || c == '_' || c == ':' || c == '$' {
            e += 1;
        } else {
            break;
        }
    }
    if s < e {
        Some(line[s..e].to_string())
    } else {
        None
    }
}

fn builtin_component_hover(def: &BuiltInComponentDef) -> Hover {
    let mut md = format!("**{}**  \n{:?} component\n\n", def.name, def.category);

    if !def.props.is_empty() {
        md.push_str("**Props:**  \n| Prop | Type | Required | Default |\n");
        md.push_str("|------|------|----------|---------|\n");
        for p in &def.props {
            md.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                p.name,
                p.kind,
                if p.required { "yes" } else { "no" },
                p.default.unwrap_or("-"),
            ));
        }
        md.push('\n');
    }

    if !def.events.is_empty() {
        md.push_str(&format!("**Events:** {}\n\n", def.events.join(", ")));
    }

    if def.allows_children {
        if let Some(kinds) = def.allowed_children_kinds {
            md.push_str(&format!("**Children:** only {}\n\n", kinds.join(", ")));
        } else {
            md.push_str("**Children:** any component\n\n");
        }
    } else {
        md.push_str("**Children:** none\n\n");
    }

    hover_text(&md)
}

fn get_style_docs(name: &str) -> Option<String> {
    match name {
        "width" => Some("`width: dimension` — Width of element.\n\nAccepts: `100`, `50%`, `200dp`".into()),
        "height" => Some("`height: dimension` — Height of element.\n\nAccepts: `100`, `50%`, `200dp`".into()),
        "padding" => Some("`padding: dimension` — Padding on all sides.\n\nAccepts: `8`, `16dp`".into()),
        "margin" => Some("`margin: dimension` — Margin on all sides.\n\nAccepts: `8`, `16dp`".into()),
        "background" => Some("`background: string` — Background color.\n\nAccepts: `\"#FF0000\"`, `\"red\"`, `$primary`".into()),
        "color" => Some("`color: string` — Text color.\n\nAccepts: `\"#000000\"`, `\"white\"`, `$text_color`".into()),
        "font_size" => Some("`font_size: dimension(sp)` — Font size.\n\nAccepts: `14sp`, `16sp`".into()),
        "font_weight" => Some("`font_weight: string` — Font weight.\n\nAccepts: `\"normal\"`, `\"bold\"`".into()),
        "opacity" => Some("`opacity: float (0-1)` — Element opacity.".into()),
        "elevation" => Some("`elevation: float` — Shadow elevation (Android).".into()),
        "border_radius" => Some("`border_radius: dimension` — Corner radius.".into()),
        "align" => Some("`align: string` — Child alignment.\n\nAccepts: `\"start\"`, `\"center\"`, `\"end\"`, `\"stretch\"`".into()),
        "justify" => Some("`justify: string` — Justify content.\n\nAccepts: `\"start\"`, `\"center\"`, `\"end\"`, `\"space_between\"`".into()),
        "gap" => Some("`gap: dimension` — Gap between children.".into()),
        "flex" => Some("`flex: float` — Flex grow factor.".into()),
        "overflow" => Some("`overflow: string` — Overflow behavior.\n\nAccepts: `\"hidden\"`, `\"visible\"`, `\"scroll\"`, `\"auto\"`".into()),
        "fit" => Some("`fit: string` — Image fit mode.\n\nAccepts: `\"cover\"`, `\"contain\"`, `\"fill\"`, `\"scale_down\"`, `\"none\"`".into()),
        _ => None,
    }
}

fn get_keyword_docs(word: &str) -> Option<String> {
    match word {
        "page:" => Some("**page:** — Define a page.\n```\npage: {\n  name: \"Home\"\n  route: \"/\"\n  children: [ ]\n}\n```".into()),
        "component" => Some("**component** — Define a reusable component.\n```\ncomponent MyCard: {\n  props: { title: string }\n  children: [ ]\n}\n```".into()),
        "fn" => Some("**fn** — Define a function.\n```\nfn handleClick: () => {\n  // body\n}\n```".into()),
        "import" => Some("**import** — Import from a module.\n```\nimport { ComponentName } \"module\"\n```".into()),
        ":store" => Some("**:store** — Define a state store.\n```\n:store UserStore {\n  name: string = \"\"\n  fn load: async () => { }\n}\n```".into()),
        ":obj" => Some("**:obj** — Define an object type.\n```\n:obj User {\n  name: string\n}\n```".into()),
        ":enum" => Some("**:enum** — Define an enum.\n```\n:enum Status { Active, Inactive }\n```".into()),
        ":vars" => Some("**:vars** — Design tokens.\n```\n:vars {\n  \\$primary: \"#007BFF\"\n}\n```".into()),
        ":i18n" => Some("**:i18n** — Internationalization strings.".into()),
        "wait:" => Some("**wait:** — Await an async operation.\n```\nwait:fetch(\"url\", { })\nwait:StoreName.action()\n```".into()),
        "navigate" => Some("**navigate()** — Navigate to a route.\n```\nnavigate(\"/path\")\nnavigate(\"/path\", replace: true)\n```".into()),
        "if" => Some("**if** — Conditional execution.\n```\nif condition { } else { }\n```".into()),
        "for" => Some("**for** — Iteration.\n```\nfor item in items { }\n```".into()),
        "return" => Some("**return** — Return a value.".into()),
        "try" => Some("**try/catch** — Error handling.\n```\ntry { } catch (err) { }\n```".into()),
        "switch" => Some("**switch** — Pattern matching.\n```\nswitch expr { case val => { } }\n```".into()),
        "show_if" => Some("**show_if:** — Conditionally display a component.".into()),
        "animate" => Some("**animate:** — Define an animation.\n```\nanimate: { property: \"opacity\"  from: 0  to: 1  duration: 300ms }\n```".into()),
        _ => None,
    }
}

fn hover_text(md: &str) -> Hover {
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: md.to_string(),
        }),
        range: None,
    }
}

// ============================================================================
// DEFINITION (scaffold)
// ============================================================================

fn compute_definition(
    text: &str,
    position: Position,
    ast: &AST,
    current_uri: &Url,
) -> Option<GotoDefinitionResponse> {
    let lines: Vec<&str> = text.lines().collect();
    let line = position.line as usize;
    if line >= lines.len() {
        return None;
    }
    let cl = lines[line];
    let cp = position.character as usize;
    if cp >= cl.len() {
        return None;
    }
    let word = extract_word_at(cl, cp)?;

    let is_known = ast.components.contains_key(&word)
        || ast.stores.contains_key(&word)
        || ast.functions.contains_key(&word)
        || ast.objects.contains_key(&word)
        || ast.enums.contains_key(&word)
        || ast.pages.iter().any(|p| p.name == word);

    if !is_known {
        return None;
    }

    for (i, l) in lines.iter().enumerate() {
        let trimmed = l.trim();

        if trimmed.starts_with("component ") && trimmed.contains(&format!("{}:", word)) {
            return Some(GotoDefinitionResponse::Scalar(Location {
                uri: current_uri.clone(),
                range: Range {
                    start: Position {
                        line: i as u32,
                        character: 0,
                    },
                    end: Position {
                        line: i as u32,
                        character: l.len() as u32,
                    },
                },
            }));
        }

        if trimmed.starts_with(":store ") && trimmed.contains(&word) {
            return Some(GotoDefinitionResponse::Scalar(Location {
                uri: current_uri.clone(),
                range: Range {
                    start: Position {
                        line: i as u32,
                        character: 0,
                    },
                    end: Position {
                        line: i as u32,
                        character: l.len() as u32,
                    },
                },
            }));
        }

        if trimmed.starts_with(":obj ") && trimmed.contains(&word) {
            return Some(GotoDefinitionResponse::Scalar(Location {
                uri: current_uri.clone(),
                range: Range {
                    start: Position {
                        line: i as u32,
                        character: 0,
                    },
                    end: Position {
                        line: i as u32,
                        character: l.len() as u32,
                    },
                },
            }));
        }

        if trimmed.starts_with(":enum ") && trimmed.contains(&word) {
            return Some(GotoDefinitionResponse::Scalar(Location {
                uri: current_uri.clone(),
                range: Range {
                    start: Position {
                        line: i as u32,
                        character: 0,
                    },
                    end: Position {
                        line: i as u32,
                        character: l.len() as u32,
                    },
                },
            }));
        }

        if trimmed.starts_with("fn ") && trimmed.contains(&format!("{}:", word)) {
            return Some(GotoDefinitionResponse::Scalar(Location {
                uri: current_uri.clone(),
                range: Range {
                    start: Position {
                        line: i as u32,
                        character: 0,
                    },
                    end: Position {
                        line: i as u32,
                        character: l.len() as u32,
                    },
                },
            }));
        }

        if trimmed == "page:" || trimmed.starts_with("page:") {
            let mut j = i + 1;
            while j < lines.len() && lines[j].trim() != "}" {
                if lines[j].trim().starts_with("name:") {
                    let name_part = lines[j].trim().strip_prefix("name:").unwrap_or("").trim();
                    if name_part == &format!("\"{}\"", word) || name_part.trim_matches('"') == word {
                        return Some(GotoDefinitionResponse::Scalar(Location {
                            uri: current_uri.clone(),
                            range: Range {
                                start: Position {
                                    line: i as u32,
                                    character: 0,
                                },
                                end: Position {
                                    line: j as u32,
                                    character: lines[j].len() as u32,
                                },
                            },
                        }));
                    }
                }
                j += 1;
            }
        }
    }

    None
}

// ============================================================================
// DOCUMENT SYMBOLS
// ============================================================================

fn compute_document_symbols(ast: &AST) -> Vec<DocumentSymbol> {
    let mut symbols = vec![];
    let empty_range = || Range {
        start: Position { line: 0, character: 0 },
        end: Position { line: 0, character: 0 },
    };
    let child_doc = |name: String, kind: SymbolKind, detail: Option<String>| DocumentSymbol {
        name,
        detail,
        kind,
        tags: None,
        deprecated: None,
        range: empty_range(),
        selection_range: empty_range(),
        children: None,
    };
    let parent_doc = |name: String, detail: Option<String>, kind: SymbolKind, children: Vec<DocumentSymbol>| DocumentSymbol {
        name,
        detail,
        kind,
        tags: None,
        deprecated: None,
        range: empty_range(),
        selection_range: empty_range(),
        children: Some(children),
    };

    for page in &ast.pages {
        let mut children: Vec<DocumentSymbol> = page
            .state
            .iter()
            .map(|(name, ftype)| child_doc(name.clone(), SymbolKind::FIELD, Some(format!("state: {:?}", ftype))))
            .collect();
        for (fn_name, _) in &page.functions {
            children.push(child_doc(fn_name.clone(), SymbolKind::FUNCTION, Some("page function".into())));
        }
        symbols.push(parent_doc(page.name.clone(), Some(format!("route: {}", page.route)), SymbolKind::PACKAGE, children));
    }

    for (name, comp) in &ast.components {
        let mut children: Vec<DocumentSymbol> = comp
            .props
            .keys()
            .map(|pn| child_doc(pn.clone(), SymbolKind::PROPERTY, None))
            .collect();
        for (fn_name, _) in &comp.functions {
            children.push(child_doc(fn_name.clone(), SymbolKind::FUNCTION, Some("component function".into())));
        }
        symbols.push(parent_doc(name.clone(), None, SymbolKind::CLASS, children));
    }

    for (name, store) in &ast.stores {
        let mut children = vec![];
        for fname in store.fields.keys() {
            children.push(child_doc(fname.clone(), SymbolKind::FIELD, None));
        }
        for fname in store.actions.keys() {
            children.push(child_doc(fname.clone(), SymbolKind::FUNCTION, None));
        }
        symbols.push(parent_doc(name.clone(), None, SymbolKind::NAMESPACE, children));
    }

    for (name, obj) in &ast.objects {
        let children: Vec<DocumentSymbol> = obj
            .fields
            .iter()
            .map(|f| child_doc(f.name.clone(), SymbolKind::FIELD, None))
            .collect();
        symbols.push(parent_doc(name.clone(), None, SymbolKind::STRUCT, children));
    }

    for (name, enm) in &ast.enums {
        let children: Vec<DocumentSymbol> = enm
            .variants
            .iter()
            .map(|v| child_doc(v.name.clone(), SymbolKind::ENUM_MEMBER, None))
            .collect();
        symbols.push(parent_doc(name.clone(), None, SymbolKind::ENUM, children));
    }

    for (fn_name, fn_def) in &ast.functions {
        let params: Vec<String> = fn_def
            .params
            .iter()
            .map(|(n, t, _)| format!("{}: {:?}", n, t))
            .collect();
        symbols.push(child_doc(fn_name.clone(), SymbolKind::FUNCTION, Some(format!("fn({})", params.join(", ")))));
    }

    symbols
}

// ============================================================================
// WORKSPACE SYMBOLS
// ============================================================================

fn compute_workspace_symbols(ast: &AST) -> Vec<SymbolInformation> {
    let mut symbols = vec![];

    let fallback_uri = Url::parse("file:///unknown").unwrap();
    for page in &ast.pages {
        symbols.push(SymbolInformation {
            name: page.name.clone(),
            kind: SymbolKind::PACKAGE,
            tags: None,
            deprecated: None,
            container_name: Some("pages".into()),
            location: Location {
                uri: Url::from_file_path(&page.name).unwrap_or_else(|_| fallback_uri.clone()),
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: 0 },
                },
            },
        });
    }

    for name in ast.components.keys() {
        symbols.push(SymbolInformation {
            name: name.clone(),
            kind: SymbolKind::CLASS,
            tags: None,
            deprecated: None,
            container_name: Some("components".into()),
            location: Location {
                uri: Url::from_file_path(name).unwrap_or_else(|_| fallback_uri.clone()),
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: 0 },
                },
            },
        });
    }

    for name in ast.stores.keys() {
        symbols.push(SymbolInformation {
            name: name.clone(),
            kind: SymbolKind::NAMESPACE,
            tags: None,
            deprecated: None,
            container_name: Some("stores".into()),
            location: Location {
                uri: Url::from_file_path(name).unwrap_or_else(|_| fallback_uri.clone()),
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: 0 },
                },
            },
        });
    }

    symbols
}

// ============================================================================
// CODE ACTIONS (scaffold)
// ============================================================================

fn compute_code_actions(
    text: &str,
    range: Range,
    ast: &AST,
    current_uri: &Url,
) -> Vec<CodeActionOrCommand> {
    let mut actions: Vec<CodeActionOrCommand> = vec![];
    let lines: Vec<&str> = text.lines().collect();

    let start_line = range.start.line as usize;
    let end_line = range.end.line as usize;

    // Check for image: without required src: prop
    for i in start_line..=end_line.min(lines.len().saturating_sub(1)) {
        let trimmed = lines[i].trim();
        if trimmed == "image:" || trimmed.starts_with("image:") {
            let mut brace_depth = 0i32;
            let mut has_src = false;
            let mut after_brace_line = lines.len();

            for j in i..lines.len() {
                for c in lines[j].chars() {
                    match c {
                        '{' => {
                            brace_depth += 1;
                            if brace_depth == 1 {
                                after_brace_line = j;
                            }
                        }
                        '}' => {
                            brace_depth -= 1;
                            if brace_depth <= 0 && j > i {
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                if lines[j].contains("src:") {
                    has_src = true;
                }

                if brace_depth <= 0 && j > i {
                    break;
                }
            }

            if !has_src {
                let insert_line = (after_brace_line + 1).min(lines.len().saturating_sub(1));
                let indent = lines[i]
                    .chars()
                    .take_while(|c| c.is_whitespace())
                    .collect::<String>();
                let edit = TextEdit {
                    range: Range {
                        start: Position {
                            line: insert_line as u32,
                            character: 0,
                        },
                        end: Position {
                            line: insert_line as u32,
                            character: 0,
                        },
                    },
                    new_text: format!("\t{}src: \"\"\n", indent),
                };
                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: "Add missing required prop 'src'".into(),
                    kind: Some(CodeActionKind::QUICKFIX),
                    edit: Some(WorkspaceEdit {
                        changes: Some(HashMap::from([(current_uri.clone(), vec![edit])])),
                        document_changes: None,
                        ..Default::default()
                    }),
                    ..Default::default()
                }));
            }
        }
    }

    // Check for non-PascalCase component names
    for i in start_line..=end_line.min(lines.len().saturating_sub(1)) {
        let trimmed = lines[i].trim();
        if let Some(rest) = trimmed.strip_prefix("component ") {
            if let Some(name_end) = rest.find(':') {
                let name = rest[..name_end].trim();
                if !name.is_empty()
                    && !name.starts_with(|c: char| c.is_uppercase())
                    && !ast.components.contains_key(name)
                {
                    let pascal = to_pascal_case(name);
                    let edit = TextEdit {
                        range: Range {
                            start: Position {
                                line: i as u32,
                                character: trimmed.find(name).unwrap_or(0) as u32,
                            },
                            end: Position {
                                line: i as u32,
                                character: (trimmed.find(name).unwrap_or(0) + name.len()) as u32,
                            },
                        },
                        new_text: pascal.clone(),
                    };
                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                        title: format!("Rename '{}' to '{}' (PascalCase)", name, pascal),
                        kind: Some(CodeActionKind::QUICKFIX),
                        edit: Some(WorkspaceEdit {
                            changes: Some(HashMap::from([(current_uri.clone(), vec![edit])])),
                            document_changes: None,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }));
                }
            }
        }
    }

    actions
}

// ============================================================================
// FORMATTING
// ============================================================================

fn compute_formatting(text: &str) -> Vec<TextEdit> {
    let lines: Vec<&str> = text.lines().collect();
    let mut edits = vec![];
    let mut expected_indent = 0i32;
    let mut last_content = 0usize;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('}') || trimmed.starts_with(']') {
            expected_indent = (expected_indent - 1).max(0);
        }
        let indent_str = "\t".repeat(expected_indent as usize);
        let formatted = format!("{}{}", indent_str, trimmed);
        if formatted != *line {
            edits.push(TextEdit {
                range: Range {
                    start: Position {
                        line: i as u32,
                        character: 0,
                    },
                    end: Position {
                        line: i as u32,
                        character: line.len() as u32,
                    },
                },
                new_text: formatted,
            });
        }
        let opens = trimmed.chars().filter(|c| *c == '{' || *c == '[').count() as i32;
        let closes = trimmed.chars().filter(|c| *c == '}' || *c == ']').count() as i32;
        expected_indent += opens - closes;
        last_content = i;
    }

    if last_content >= lines.len().saturating_sub(2) {
        if let Some(last) = lines.last() {
            if !last.is_empty() {
                edits.push(TextEdit {
                    range: Range {
                        start: Position {
                            line: lines.len() as u32,
                            character: 0,
                        },
                        end: Position {
                            line: lines.len() as u32,
                            character: 0,
                        },
                    },
                    new_text: "\n".into(),
                });
            }
        }
    }

    edits
}

// ============================================================================
// FOLDING RANGES
// ============================================================================

fn compute_folding_ranges(text: &str) -> Vec<FoldingRange> {
    let lines: Vec<&str> = text.lines().collect();
    let mut ranges = vec![];
    let mut brace_stack: Vec<usize> = vec![];
    let mut bracket_stack: Vec<usize> = vec![];

    for (i, line) in lines.iter().enumerate() {
        for c in line.chars() {
            match c {
                '{' => brace_stack.push(i),
                '}' => {
                    if let Some(start) = brace_stack.pop() {
                        if i > start + 1 {
                            ranges.push(FoldingRange {
                                start_line: start as u32,
                                start_character: None,
                                end_line: i as u32,
                                end_character: None,
                                kind: Some(FoldingRangeKind::Region),
                                collapsed_text: None,
                            });
                        }
                    }
                }
                '[' => bracket_stack.push(i),
                ']' => {
                    if let Some(start) = bracket_stack.pop() {
                        if i > start + 1 {
                            ranges.push(FoldingRange {
                                start_line: start as u32,
                                start_character: None,
                                end_line: i as u32,
                                end_character: None,
                                kind: Some(FoldingRangeKind::Region),
                                collapsed_text: Some("[...]".into()),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }

    ranges
}

// ============================================================================
// SEMANTIC TOKENS (scaffold)
// ============================================================================

fn compute_semantic_tokens(text: &str, _ast: &AST) -> Vec<SemanticToken> {
    let lines: Vec<&str> = text.lines().collect();
    let mut tokens: Vec<SemanticToken> = vec![];
    let builtin_names: Vec<&str> = registry().names();

    for (line_num, line) in lines.iter().enumerate() {
        let lnum = line_num as u32;
        let bytes = line.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            if bytes[i].is_ascii_whitespace() {
                i += 1;
                continue;
            }

            // Comment
            if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
                tokens.push(SemanticToken {
                    delta_line: lnum,
                    delta_start: i as u32,
                    length: (bytes.len() - i) as u32,
                    token_type: 16,
                    token_modifiers_bitset: 0,
                });
                break;
            }

            // String literal
            if bytes[i] == b'"' {
                let start = i;
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
                tokens.push(SemanticToken {
                    delta_line: lnum,
                    delta_start: start as u32,
                    length: (i - start) as u32,
                    token_type: 17,
                    token_modifiers_bitset: 0,
                });
                continue;
            }

            // Number
            if bytes[i].is_ascii_digit() {
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                    i += 1;
                }
                tokens.push(SemanticToken {
                    delta_line: lnum,
                    delta_start: start as u32,
                    length: (i - start) as u32,
                    token_type: 18,
                    token_modifiers_bitset: 0,
                });
                continue;
            }

            // Identifier or keyword
            if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' || bytes[i] == b':' {
                let start = i;
                while i < bytes.len()
                    && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b':')
                {
                    i += 1;
                }
                let word = &line[start..i];
                let lower = word.to_lowercase();

                let tt = if is_keyword(&lower) {
                    14
                } else if is_type(&lower) {
                    1
                } else if builtin_names.contains(&lower.as_str()) {
                    2
                } else if lower == "true" || lower == "false" || lower == "null" {
                    14
                } else {
                    continue;
                };

                tokens.push(SemanticToken {
                    delta_line: lnum,
                    delta_start: start as u32,
                    length: (i - start) as u32,
                    token_type: tt,
                    token_modifiers_bitset: 0,
                });
                continue;
            }

            i += 1;
        }
    }

    // Delta-encode tokens
    let mut result = Vec::with_capacity(tokens.len());
    let mut prev_line = 0u32;
    let mut prev_start = 0u32;
    for token in &tokens {
        let delta_line = token.delta_line - prev_line;
        let delta_start = if delta_line == 0 {
            token.delta_start - prev_start
        } else {
            token.delta_start
        };
        result.push(SemanticToken {
            delta_line,
            delta_start,
            length: token.length,
            token_type: token.token_type,
            token_modifiers_bitset: token.token_modifiers_bitset,
        });
        prev_line = token.delta_line;
        prev_start = token.delta_start + token.length;
    }

    result
}

fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "page"
            | "component"
            | "fn"
            | "if"
            | "else"
            | "for"
            | "in"
            | "return"
            | "switch"
            | "case"
            | "try"
            | "catch"
            | "finally"
            | "import"
            | "wait"
            | "show_if"
            | "animate"
            | "const"
            | "describe"
            | "it"
            | "navigate"
            | "navigate_back"
            | "navigate_replace"
            | "navigate_back_to"
            | "navigate_modal"
            | "navigate_dismiss"
            | "props"
            | "state"
            | "styles"
            | "children"
            | ":store"
            | ":obj"
            | ":enum"
            | ":type"
            | ":vars"
            | ":i18n"
            | ":validation"
            | ":breakpoints"
            | ":typography"
            | ":app"
    )
}

fn is_type(word: &str) -> bool {
    matches!(word, "string" | "int" | "bool" | "float" | "object" | "list")
}

fn to_pascal_case(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    let mut capitalize_next = true;
    for c in name.chars() {
        if c == '_' || c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    if result.is_empty() {
        name.to_string()
    } else {
        result
    }
}

// ============================================================================
// DOCUMENT HIGHLIGHTS (scaffold)
// ============================================================================

fn compute_document_highlights(text: &str, position: Position) -> Vec<DocumentHighlight> {
    let lines: Vec<&str> = text.lines().collect();
    let line = position.line as usize;
    if line >= lines.len() {
        return vec![];
    }

    let word = match extract_word_at(lines[line], position.character as usize) {
        Some(w) if !w.is_empty() => w,
        _ => return vec![],
    };

    let mut highlights = vec![];
    for (i, l) in lines.iter().enumerate() {
        let l_bytes = l.as_bytes();
        let mut j = 0;
        while j < l_bytes.len() {
            if let Some(pos) = l[j..].find(&word) {
                let abs_pos = j + pos;
                let after_end = abs_pos + word.len();
                let boundary_before = abs_pos == 0
                    || !l_bytes[abs_pos - 1].is_ascii_alphanumeric()
                        && l_bytes[abs_pos - 1] != b'_';
                let boundary_after = after_end >= l_bytes.len()
                    || !l_bytes[after_end].is_ascii_alphanumeric()
                        && l_bytes[after_end] != b'_';
                if boundary_before && boundary_after {
                    highlights.push(DocumentHighlight {
                        range: Range {
                            start: Position {
                                line: i as u32,
                                character: abs_pos as u32,
                            },
                            end: Position {
                                line: i as u32,
                                character: after_end as u32,
                            },
                        },
                        kind: Some(DocumentHighlightKind::READ),
                    });
                }
                j = abs_pos + 1;
            } else {
                break;
            }
        }
    }

    highlights
}

// ============================================================================
// RENAME (scaffold)
// ============================================================================

fn compute_rename(
    text: &str,
    position: Position,
    new_name: &str,
    _ast: &AST,
    current_uri: &Url,
) -> WorkspaceEdit {
    let highlights = compute_document_highlights(text, position);
    if highlights.is_empty() {
        return WorkspaceEdit::default();
    }

    let edits: Vec<TextEdit> = highlights
        .into_iter()
        .map(|h| TextEdit {
            range: h.range,
            new_text: new_name.to_string(),
        })
        .collect();

    WorkspaceEdit {
        changes: Some(HashMap::from([(current_uri.clone(), edits)])),
        document_changes: None,
        ..Default::default()
    }
}

// ============================================================================
// DOCUMENT COLORS
// ============================================================================

fn compute_document_colors(text: &str) -> Vec<ColorInformation> {
    let mut colors = vec![];
    for (i, line) in text.lines().enumerate() {
        let bytes = line.as_bytes();
        let mut j = 0;
        while j < bytes.len() {
            if bytes[j] == b'#' {
                let start = j;
                let mut len = 0;
                for k in (j + 1)..bytes.len().min(j + 7) {
                    if (bytes[k] as char).is_ascii_hexdigit() {
                        len += 1;
                    } else {
                        break;
                    }
                }
                if len == 3 || len == 6 {
                    let hex = &line[start..=start + len];
                    let (r, g, b) = if len == 3 {
                        let rh = hex.as_bytes()[1] as char;
                        let gh = hex.as_bytes()[2] as char;
                        let bh = hex.as_bytes()[3] as char;
                        (
                            u8::from_str_radix(&format!("{}{}", rh, rh), 16).unwrap_or(0),
                            u8::from_str_radix(&format!("{}{}", gh, gh), 16).unwrap_or(0),
                            u8::from_str_radix(&format!("{}{}", bh, bh), 16).unwrap_or(0),
                        )
                    } else {
                        (
                            u8::from_str_radix(&hex[1..3], 16).unwrap_or(0),
                            u8::from_str_radix(&hex[3..5], 16).unwrap_or(0),
                            u8::from_str_radix(&hex[5..7], 16).unwrap_or(0),
                        )
                    };
                    colors.push(ColorInformation {
                        range: Range {
                            start: Position {
                                line: i as u32,
                                character: start as u32,
                            },
                            end: Position {
                                line: i as u32,
                                character: (start + len) as u32,
                            },
                        },
                        color: Color {
                            red: r as f32 / 255.0,
                            green: g as f32 / 255.0,
                            blue: b as f32 / 255.0,
                            alpha: 1.0,
                        },
                    });
                    j = start + len;
                    continue;
                }
            }
            j += 1;
        }
    }
    colors
}
