#![allow(dead_code, reason = "lint fixture")]
struct Strings { zebra: String, alpha: String }
struct Containers { zebra: Vec<Option<String>>, alpha: Box<Result<String, u32>> }
#[derive(Debug)]
struct Printable { zebra: String, alpha: String }
#[derive(Clone, Copy, Debug)]
struct Copied { zebra: u32, alpha: u32 }
struct Borrowed<'view, Value> { zebra: &'view Value, alpha: u32 }
struct Documented {
    /// Last in the alphabet.
    zebra: String,
    /// First in the alphabet.
    alpha: String,
}
struct Raw { zebra: u32, r#type: u32 }
