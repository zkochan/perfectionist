//! End-to-end safety and machine-fix checks for struct field ordering.
pub mod _utils;

use _utils::{
    TempDir, build_project_with_config, cargo_manifest_dir, run_dylint, run_dylint_fix,
    shared_target_dir,
};
use std::fs;
use std::process::Command;

const CONFIG: &str = "[perfectionist]\nenable = [\"unordered_struct_fields\"]\n";
const POSITIVE: &str = include_str!("fixtures/struct_field_order/positive.rs");
const EXCLUDED: &str = include_str!("fixtures/struct_field_order/excluded.rs");

#[test]
fn safe_fields_are_fixed_and_the_result_is_idempotent() {
    let temp = project("struct_order_positive", POSITIVE, CONFIG);
    let target = shared_target_dir();
    let (stderr, success) = run_dylint_fix(temp.path(), &target);
    assert!(success, "{stderr}");
    let source = fs::read_to_string(temp.path().join("src/lib.rs")).unwrap();
    assert!(
        source.contains("struct Strings { alpha: String, zebra: String }"),
        "{source}\n{stderr}",
    );
    assert!(
        source.contains(
            "struct Containers { alpha: Box<Result<String, u32>>, zebra: Vec<Option<String>> }"
        ),
        "{source}",
    );
    assert!(
        source.contains("struct Printable { alpha: String, zebra: String }"),
        "{source}",
    );
    assert!(
        source.contains("struct Copied { alpha: u32, zebra: u32 }"),
        "{source}",
    );
    assert!(
        source.contains("struct Borrowed<'view, Value> { alpha: u32, zebra: &'view Value }"),
        "{source}",
    );
    assert!(source.contains("/// First in the alphabet.\n    alpha: String,\n    /// Last in the alphabet.\n    zebra: String"), "{source}");
    assert!(
        source.contains("struct Raw { r#type: u32, zebra: u32 }"),
        "{source}",
    );
    let (stderr, success) = run_dylint_fix(temp.path(), &target);
    assert!(success, "{stderr}");
    assert_eq!(
        source,
        fs::read_to_string(temp.path().join("src/lib.rs")).unwrap(),
    );
}

#[test]
fn order_sensitive_declarations_are_untouched() {
    let temp = project("struct_order_excluded", EXCLUDED, CONFIG);
    let (stderr, success) = run_dylint(temp.path(), &shared_target_dir());
    assert!(success, "{stderr}");
    assert!(
        !stderr.contains("named struct fields are not in alphabetical order"),
        "{stderr}",
    );
}

#[test]
fn opt_out_preserves_unsorted_fields() {
    let temp = project("struct_order_disabled", POSITIVE, "");
    let (stderr, success) = run_dylint(temp.path(), &shared_target_dir());
    assert!(success, "{stderr}");
    assert!(
        !stderr.contains("named struct fields are not in alphabetical order"),
        "{stderr}",
    );
}

fn project(name: &str, source: &str, config: &str) -> TempDir {
    let temp = TempDir::new().unwrap();
    build_project_with_config(
        temp.path(),
        name,
        cargo_manifest_dir(),
        &[("src/lib.rs", source)],
        config,
    );
    temp
}

#[test]
fn procedural_attributes_and_disguised_derives_are_excluded() {
    let temp = TempDir::new().unwrap();
    build_project_with_config(
        temp.path(),
        "struct_order_attributes",
        cargo_manifest_dir(),
        &[
            (
                "src/lib.rs",
                include_str!("fixtures/struct_field_order/attributes.rs"),
            ),
            (
                "macros/src/lib.rs",
                include_str!("fixtures/struct_field_order/macros.rs"),
            ),
            (
                "macros/Cargo.toml",
                "[package]\nname = \"macros\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[lib]\nproc-macro = true\n",
            ),
        ],
        CONFIG,
    );
    let manifest = temp.path().join("Cargo.toml");
    let contents = fs::read_to_string(&manifest).unwrap();
    fs::write(
        manifest,
        format!("{contents}\n[dependencies]\nmacros = {{ path = \"macros\" }}\n"),
    )
    .unwrap();
    let (stderr, success) = run_dylint(temp.path(), &shared_target_dir());
    assert!(success, "{stderr}");
    assert!(
        !stderr.contains("named struct fields are not in alphabetical order"),
        "{stderr}",
    );
}

#[test]
fn ambiguous_comments_are_diagnosed_without_automatic_edits() {
    let source = "#![allow(dead_code, reason = \"lint fixture\")]\nstruct Commented {\n    zebra: u32,\n    // A grouping boundary whose ownership is ambiguous.\n    alpha: u32,\n}\n";
    let temp = project("struct_order_comments", source, CONFIG);
    let (stderr, success) = run_dylint(temp.path(), &shared_target_dir());
    assert!(success, "{stderr}");
    assert!(
        stderr.contains("named struct fields are not in alphabetical order"),
        "{stderr}",
    );
    let (stderr, success) = run_dylint_fix(temp.path(), &shared_target_dir());
    assert!(success, "{stderr}");
    assert_eq!(
        source,
        fs::read_to_string(temp.path().join("src/lib.rs")).unwrap(),
    );
}

#[test]
fn clippy_fixes_shorthand_initializers_after_declaration_ordering() {
    let source = "#![allow(dead_code)]\n#![warn(clippy::inconsistent_struct_constructor)]\nstruct Pair { zebra: String, alpha: String }\nfn construct(alpha: String, zebra: String) -> Pair { Pair { zebra, alpha } }\nfn effects() -> Pair { Pair { zebra: String::from(\"zebra\"), alpha: String::from(\"alpha\") } }\n";
    let temp = project("struct_order_clippy", source, CONFIG);
    let (stderr, success) = run_dylint_fix(temp.path(), &shared_target_dir());
    assert!(success, "{stderr}");
    let sorted = fs::read_to_string(temp.path().join("src/lib.rs")).unwrap();
    assert!(
        sorted.contains("struct Pair { alpha: String, zebra: String }"),
        "{sorted}",
    );
    assert!(sorted.contains("Pair { zebra, alpha }"), "{sorted}");
    let output = Command::new("cargo")
        .args([
            "clippy",
            "--fix",
            "--allow-no-vcs",
            "--allow-dirty",
            "--lib",
        ])
        .current_dir(temp.path())
        .env("CARGO_TARGET_DIR", shared_target_dir())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr),
    );
    let fixed = fs::read_to_string(temp.path().join("src/lib.rs")).unwrap();
    assert!(fixed.contains("Pair { alpha, zebra }"), "{fixed}");
    assert!(
        fixed.contains(r#"Pair { zebra: String::from("zebra"), alpha: String::from("alpha") }"#),
        "{fixed}",
    );
}
