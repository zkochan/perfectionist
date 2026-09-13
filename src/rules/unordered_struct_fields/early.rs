use crate::attr_tokens::attribute_calls_of;
use crate::common::span_is_macro_generated;
use crate::derive_list::derive_entries;
use rustc_ast::visit::{self, Visitor};
use rustc_ast::{Attribute, Item, ItemKind, MacCall, VariantData};
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_span::{Span, Symbol, sym};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub(super) struct SourceStruct {
    pub fields: Vec<Span>,
    pub derives: Vec<Symbol>,
}

pub(super) type Sources = Arc<Mutex<HashMap<(u32, u32), SourceStruct>>>;

pub(super) struct Collector(pub Sources);

impl EarlyLintPass for Collector {
    fn check_item(&mut self, _: &EarlyContext<'_>, item: &Item) {
        let ItemKind::Struct(ident, _, VariantData::Struct { fields, .. }) = &item.kind else {
            return;
        };
        if span_is_macro_generated(item.span)
            || !safe_attributes(&item.attrs, true)
            || fields.iter().any(|field| {
                field.is_placeholder
                    || field.default.is_some()
                    || contains_type_macro(&field.ty)
                    || span_is_macro_generated(field.span)
                    || !safe_attributes(&field.attrs, false)
            })
        {
            return;
        }
        let Some(derives) = safe_derives(&item.attrs) else {
            return;
        };
        let fields = fields
            .iter()
            .map(|field| {
                field.attrs.iter().fold(field.span, |span, attr| {
                    span.with_lo(span.lo().min(attr.span.lo()))
                })
            })
            .collect();
        self.0
            .lock()
            .unwrap()
            .insert(key(ident.span), SourceStruct { fields, derives });
    }
}

fn safe_attributes(attrs: &[Attribute], derives: bool) -> bool {
    attrs.iter().all(|attr| {
        attr.is_doc_comment()
            || [
                sym::doc,
                sym::allow,
                sym::warn,
                sym::deny,
                sym::forbid,
                sym::expect,
                sym::deprecated,
                sym::must_use,
            ]
            .iter()
            .any(|name| attr.has_name(*name))
            || derives && attr.has_name(sym::derive)
    })
}

pub(super) fn key(span: Span) -> (u32, u32) {
    (span.lo().0, span.hi().0)
}

fn contains_type_macro(ty: &rustc_ast::Ty) -> bool {
    struct Finder(bool);
    impl<'ast> Visitor<'ast> for Finder {
        fn visit_mac_call(&mut self, _: &'ast MacCall) {
            self.0 = true;
        }
    }
    let mut finder = Finder(false);
    visit::walk_ty(&mut finder, ty);
    finder.0
}

fn safe_derives(attrs: &[Attribute]) -> Option<Vec<Symbol>> {
    let mut names = HashSet::new();
    let mut derives = Vec::new();
    for call in attribute_calls_of(attrs) {
        if call.name != sym::derive {
            continue;
        }
        for entry in derive_entries(call.tokens)? {
            if !entry.is_unqualified
                || !matches!(entry.name, sym::Debug | sym::Copy | sym::Clone)
                || !names.insert(entry.name)
            {
                return None;
            }
            derives.push(entry.name);
        }
    }
    Some(derives)
}
