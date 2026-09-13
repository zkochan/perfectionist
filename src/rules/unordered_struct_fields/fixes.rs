use clippy_utils::diagnostics::span_lint_and_then;
use rustc_errors::Applicability;
use rustc_lint::{LateContext, LintContext};
use rustc_session::lint::Lint;
use rustc_span::Span;

pub(super) fn emit(
    cx: &LateContext<'_>,
    lint: &'static Lint,
    span: Span,
    fields: &[Span],
    order: &[usize],
) {
    span_lint_and_then(cx, lint, span, lint.desc, |diag| {
        if let Some(parts) = replacements(cx, span, fields, order) {
            diag.multipart_suggestion(
                "reorder the fields",
                parts,
                Applicability::MachineApplicable,
            );
        } else {
            diag.help("reorder the fields while preserving their associated comments");
        }
    });
}

fn replacements(
    cx: &LateContext<'_>,
    span: Span,
    fields: &[Span],
    order: &[usize],
) -> Option<Vec<(Span, String)>> {
    let sm = cx.sess().source_map();
    let source = sm.span_to_snippet(span).ok()?;
    if rustc_lexer::tokenize(&source, rustc_lexer::FrontmatterAllowed::No).any(|token| {
        matches!(
            token.kind,
            rustc_lexer::TokenKind::LineComment { doc_style: None }
                | rustc_lexer::TokenKind::BlockComment {
                    doc_style: None,
                    ..
                },
        )
    }) {
        return None;
    }
    let snippets: Vec<_> = fields
        .iter()
        .map(|span| sm.span_to_snippet(*span).ok())
        .collect::<Option<_>>()?;
    for adjacent in fields.windows(2) {
        let gap = sm
            .span_to_snippet(
                adjacent[0]
                    .with_lo(adjacent[0].hi())
                    .with_hi(adjacent[1].lo()),
            )
            .ok()?;
        if gap.trim() != "," {
            return None;
        }
    }
    Some(
        fields
            .iter()
            .zip(order)
            .map(|(span, index)| (*span, snippets[*index].clone()))
            .collect(),
    )
}
