use crate::common::{DefaultState, hir_in_external_macro, span_is_macro_generated};
use crate::rule_index::{Register, rule};
use early::{Collector, Sources};
use rustc_hir::{Item, ItemKind, VariantData};
use rustc_lint::{LateContext, LateLintPass, LintStore};
use rustc_session::impl_lint_pass;
use std::sync::{Arc, Mutex};

mod early;
mod fixes;
mod safety;
use rustc_session::declare_tool_lint;

declare_tool_lint! {
    /// ### What it does
    ///
    /// Orders eligible named struct declarations by case-sensitive
    /// alphabetical identifier order.
    ///
    /// ### Safety exclusions
    ///
    /// Explicit representations, conditional fields, macro-generated structs,
    /// macros in field types, unknown attributes and derives, custom destructors
    /// (including owned nested values), and unresolved owned types are excluded. Ordinary
    /// `String` and references remain supported, as do `Vec`, `Box`, `Option`
    /// and `Result` with eligible contents. Custom allocators are excluded.
    /// Built-in `Debug` is permitted: its displayed field order
    /// intentionally follows the alphabetical declaration. Built-in `Copy`
    /// and `Clone` are permitted only for compiler-proven `Copy` structs.
    /// Other derives, including `Hash`, ordering, serialization, CLI parsing,
    /// non-Copy `Clone` and `Default`, are excluded. Qualified derive paths and
    /// duplicate derive names are conservatively excluded too.
    ///
    /// Automatic edits require unambiguous source spans and separators;
    /// documentation attributes move with their fields. Ambiguous comments
    /// receive diagnostics without edits. External structs are not checked.
    ///
    /// ### Interaction with Clippy
    ///
    /// Clippy's `inconsistent_struct_constructor` checks shorthand initializer
    /// order against the declaration. Its default configuration supplies safe
    /// initializer fixes after this rule has reordered the declaration. This
    /// rule does not reorder initializer evaluation.
    ///
    /// Reordering declarations can change Rust layout and standard allocation
    /// deallocation order. This rule does not promise binary layout stability
    /// or preserve allocator instrumentation. Explicit layout contracts and
    /// custom destruction are outside its scope.
    ///
    /// ### Why restrict this?
    ///
    /// A predictable order makes fields easier to find and reduces arbitrary
    /// differences between declarations and construction sites.
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// struct Coordinates { x: u32, y: u32 }
    /// let coordinates = Coordinates { x: 1, y: 2 };
    /// ```
    pub perfectionist::UNORDERED_STRUCT_FIELDS,
    Warn,
    "named struct fields are not in alphabetical order",
    report_in_external_macro: false
}

const CONFIG_KEY: &str = "perfectionist::unordered_struct_fields";

/// The rule has no configuration knobs. Not dead code: the read
/// below rejects a mistyped key in the rule's `dylint.toml` table,
/// and gen-docs needs the struct for `Configuration: none.`
#[derive(Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Config {}

impl Register for rule::UnorderedStructFields {
    const DEFAULT_STATE: DefaultState = DefaultState::Inactive;

    fn register_lint(lint_store: &mut LintStore) {
        lint_store.register_lints(&[UNORDERED_STRUCT_FIELDS]);
    }

    fn register_pass(lint_store: &mut LintStore) {
        let _: Config = dylint_linting::config_or_default(CONFIG_KEY);
        register(lint_store);
    }
}

fn register(store: &mut LintStore) {
    let sources = Arc::new(Mutex::new(Default::default()));
    let early_sources = Arc::clone(&sources);
    store.register_pre_expansion_lint_pass(Box::new(move || {
        Box::new(Collector(Arc::clone(&early_sources)))
    }));
    store.register_late_lint_pass(Box::new(move |_| {
        Box::new(Ordering {
            sources: Arc::clone(&sources),
            derives: Default::default(),
        })
    }));
}

struct Ordering {
    sources: Sources,
    derives: safety::BuiltinDerives,
}

impl_lint_pass!(Collector => [UNORDERED_STRUCT_FIELDS]);
impl_lint_pass!(Ordering => [UNORDERED_STRUCT_FIELDS]);

impl<'tcx> LateLintPass<'tcx> for Ordering {
    fn check_crate(&mut self, cx: &LateContext<'tcx>) {
        self.derives = safety::builtin_derives(cx);
    }

    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        let ItemKind::Struct(ident, _, VariantData::Struct { fields, .. }) = item.kind else {
            return;
        };
        if span_is_macro_generated(item.span) || hir_in_external_macro(cx, item.hir_id(), item.span)
        {
            return;
        }
        let Some(source) = self
            .sources
            .lock()
            .unwrap()
            .get(&early::key(ident.span))
            .cloned()
        else {
            return;
        };
        let ty = cx
            .tcx
            .type_of(item.owner_id)
            .instantiate_identity()
            .skip_normalization();
        if fields.len() != source.fields.len() || !safety::eligible(cx, ty, &source, &self.derives)
        {
            return;
        }
        let mut order: Vec<_> = (0..fields.len()).collect();
        order.sort_by_key(|index| fields[*index].ident.name.as_str());
        if order.iter().copied().ne(0..fields.len()) {
            fixes::emit(
                cx,
                UNORDERED_STRUCT_FIELDS,
                item.span,
                &source.fields,
                &order,
            );
        }
    }
}
