#![allow(dead_code, reason = "lint fixture")]
struct Resource;
impl Drop for Resource { fn drop(&mut self) {} }
struct Guarded { zebra: Resource, alpha: Resource }
struct Transitive { zebra: Option<Box<Resource>>, alpha: String }
struct Shared { zebra: Option<std::sync::Arc<Resource>>, alpha: String }
struct Owned<Value> { zebra: Value, alpha: u32 }
struct Dynamic { zebra: Box<dyn core::fmt::Debug>, alpha: u32 }
#[repr(C)]
struct Foreign { zebra: u32, alpha: u32 }
#[derive(Clone)]
struct Cloned { zebra: String, alpha: String }
#[derive(Default)]
struct Defaults { zebra: String, alpha: String }
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Compared { zebra: u32, alpha: u32 }
struct Conditional { zebra: u32, #[cfg(any())] hidden: u32, alpha: u32 }
struct DropItself { zebra: String, alpha: String }
impl Drop for DropItself { fn drop(&mut self) {} }
macro_rules! structure { () => { struct Generated { zebra: u32, alpha: u32 } } }
structure!();
struct LocationDependent { zebra: [u8; line!() as usize], alpha: u32 }
