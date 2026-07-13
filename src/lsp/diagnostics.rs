use std::path::Path;

use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, Url};

use crate::parser::{FrameError, ErrorCategory};
use crate::cli::lint::{LintDiagnostic as FrameLintDiag, Severity as LintSeverity};

/// Convert a Vec of FrameErrors to LSP Diagnostics with proper positions.
pub fn frame_errors_to_diagnostics(errors: &[FrameError]) -> Vec<Diagnostic> {
    errors.iter().map(frame_error_to_diagnostic).collect()
}

/// Convert a single FrameError to an LSP Diagnostic.
pub fn frame_error_to_diagnostic(err: &FrameError) -> Diagnostic {
    let severity = match err.category {
        ErrorCategory::ParseError => Some(DiagnosticSeverity::ERROR),
        ErrorCategory::TypeMismatchError => Some(DiagnosticSeverity::ERROR),
        ErrorCategory::UnresolvedImportError => Some(DiagnosticSeverity::ERROR),
        ErrorCategory::MissingPropError => Some(DiagnosticSeverity::ERROR),
        ErrorCategory::MissingAssetError => Some(DiagnosticSeverity::WARNING),
        ErrorCategory::CircularDependencyError => Some(DiagnosticSeverity::ERROR),
        ErrorCategory::UnsupportedPlatformError => Some(DiagnosticSeverity::WARNING),
    };

    let line = (err.line.max(1) - 1) as u32;
    let col = (err.column.max(1) - 1) as u32;

    Diagnostic {
        range: Range {
            start: Position {
                line,
                character: col,
            },
            end: Position {
                line,
                character: col + 1,
            },
        },
        severity,
        code: Some(lsp_types::NumberOrString::String(format!("{:?}", err.category))),
        source: Some("frame".into()),
        message: err.message.clone(),
        ..Default::default()
    }
}

/// Convert Frame lint diagnostics to LSP Diagnostics, grouped by file URL.
pub fn lint_diagnostics_to_lsp(
    lint_diags: &[FrameLintDiag],
    workspace_root: &Path,
) -> Vec<(Url, Vec<Diagnostic>)> {
    let mut grouped: std::collections::HashMap<String, Vec<Diagnostic>> =
        std::collections::HashMap::new();

    for ld in lint_diags {
        let severity = match ld.severity {
            LintSeverity::Error => DiagnosticSeverity::ERROR,
            LintSeverity::Warning => DiagnosticSeverity::WARNING,
            LintSeverity::Info => DiagnosticSeverity::INFORMATION,
        };

        let diagnostic = Diagnostic {
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 0 },
            },
            severity: Some(severity),
            code: Some(lsp_types::NumberOrString::String(ld.rule.to_string())),
            source: Some("frame-lint".into()),
            message: if let Some(hint) = ld.hint {
                format!("{} — Hint: {}", ld.message, hint)
            } else {
                ld.message.clone()
            },
            ..Default::default()
        };

        let ctx = if ld.context.is_empty() {
            workspace_root.to_string_lossy().to_string()
        } else {
            ld.context.clone()
        };
        grouped.entry(ctx).or_default().push(diagnostic);
    }

    grouped
        .into_iter()
        .filter_map(|(file_path, diagnostics)| {
            let url = Url::from_file_path(&file_path).ok()?;
            Some((url, diagnostics))
        })
        .collect()
}

/// Publish all diagnostics (parse errors + lint) for a single file.
pub fn publish_diagnostics_for_file(
    file_url: &Url,
    errors: &[FrameError],
    lint_diags: &[FrameLintDiag],
) -> Vec<Diagnostic> {
    let mut all_diags = frame_errors_to_diagnostics(errors);

    for ld in lint_diags {
        let severity = match ld.severity {
            LintSeverity::Error => DiagnosticSeverity::ERROR,
            LintSeverity::Warning => DiagnosticSeverity::WARNING,
            LintSeverity::Info => DiagnosticSeverity::INFORMATION,
        };

        all_diags.push(Diagnostic {
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 1 },
            },
            severity: Some(severity),
            code: Some(lsp_types::NumberOrString::String(ld.rule.to_string())),
            source: Some("frame".into()),
            message: if let Some(hint) = ld.hint {
                format!("{} — Hint: {}", ld.message, hint)
            } else {
                ld.message.clone()
            },
            ..Default::default()
        });
    }

    all_diags
}
