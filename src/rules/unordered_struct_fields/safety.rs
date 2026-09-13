use super::early::SourceStruct;
use rustc_hir::def_id::DefId;
use rustc_lint::LateContext;
use rustc_middle::ty::{self, Ty};
use rustc_span::{Symbol, sym};
use std::collections::{HashMap, HashSet};

pub(super) type BuiltinDerives = HashMap<DefId, HashSet<Symbol>>;

pub(super) fn builtin_derives(cx: &LateContext<'_>) -> BuiltinDerives {
    let mut result = HashMap::<_, HashSet<_>>::new();
    for (trait_id, impls) in cx.tcx.all_local_trait_impls(()) {
        for impl_id in impls {
            if cx.tcx.is_builtin_derived(impl_id.to_def_id())
                && let ty::Adt(adt, _) = cx
                    .tcx
                    .type_of(*impl_id)
                    .instantiate_identity()
                    .skip_normalization()
                    .kind()
            {
                result
                    .entry(adt.did())
                    .or_default()
                    .insert(cx.tcx.item_name(*trait_id));
            }
        }
    }
    result
}

pub(super) fn eligible<'tcx>(
    cx: &LateContext<'tcx>,
    ty: Ty<'tcx>,
    source: &SourceStruct,
    derives: &BuiltinDerives,
) -> bool {
    let ty::Adt(adt, _) = ty.kind() else {
        return false;
    };
    source.derives.iter().all(|name| {
        derives
            .get(&adt.did())
            .is_some_and(|known| known.contains(name))
    }) && (!source
        .derives
        .iter()
        .any(|name| matches!(*name, sym::Copy | sym::Clone))
        || cx.tcx.type_is_copy_modulo_regions(cx.typing_env(), ty))
        && safe_drop(cx, ty, &mut HashSet::new())
}

fn safe_drop<'tcx>(cx: &LateContext<'tcx>, ty: Ty<'tcx>, visiting: &mut HashSet<Ty<'tcx>>) -> bool {
    if visiting.len() >= 64 {
        return false;
    }
    if !visiting.insert(ty) {
        return true;
    }
    let safe = match ty.kind() {
        ty::Bool
        | ty::Char
        | ty::Int(_)
        | ty::Uint(_)
        | ty::Float(_)
        | ty::Never
        | ty::Ref(..)
        | ty::RawPtr(..)
        | ty::FnDef(..)
        | ty::FnPtr(..) => true,
        ty::Tuple(types) => types.iter().all(|ty| safe_drop(cx, ty, visiting)),
        ty::Array(element, _) => safe_drop(cx, *element, visiting),
        ty::Adt(adt, args) => {
            if cx.tcx.is_diagnostic_item(sym::String, adt.did()) {
                true
            } else if adt.is_box() || cx.tcx.is_diagnostic_item(sym::Vec, adt.did()) {
                let mut types = args.types();
                types.next().is_some_and(|element| safe_drop(cx, element, visiting))
                    && types.all(|allocator| matches!(allocator.kind(), ty::Adt(allocator, _) if cx.tcx.lang_items().global_alloc_ty() == Some(allocator.did())))
            } else {
                !adt.has_dtor(cx.tcx)
                    && !adt.is_union()
                    && adt.all_fields().all(|field| {
                        safe_drop(cx, field.ty(cx.tcx, args).skip_normalization(), visiting)
                    })
            }
        }
        _ => false,
    };
    visiting.remove(&ty);
    safe
}
