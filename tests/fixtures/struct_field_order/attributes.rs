#![allow(dead_code, reason = "lint fixture")]
use macros::Debug;
#[derive(Debug)]
struct Impostor { zebra: u32, alpha: u32 }
#[derive(macros::From)]
struct Positional { zebra: u32, alpha: u32 }
#[macros::object]
struct Object { zebra: u32, alpha: u32 }
#[derive(macros::Serialize)]
struct Serialized { zebra: u32, #[serde(default)] alpha: u32 }
#[derive(macros::Parser)]
struct Command { zebra: u32, #[arg(long)] alpha: u32 }

mod combined {
    #[derive(Debug, macros::Debug)]
    struct Composite { zebra: u32, alpha: u32 }
}
