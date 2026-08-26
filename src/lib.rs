//! A small, fast expression compiler and interpreter in Rust.
//!
//! Expressions are compiled into a compact stack-based program (reverse Polish
//! notation) and evaluated on a small machine; see the crate README for design
//! notes.
//!
//! # Example
//!
//! ```
//! use xpr::{Blueprint, Callable, Interpreter, MethodInfo};
//!
//! #[derive(Debug)]
//! enum Id {
//!     Sum,
//! }
//!
//! struct Math;
//!
//! impl Blueprint<Id> for Math {
//!     fn find(&self, name: &str) -> Option<MethodInfo<Id>> {
//!         match name {
//!             "sum" => Some(MethodInfo::new(Id::Sum, 2)),
//!             _ => None,
//!         }
//!     }
//! }
//!
//! impl Callable<Id> for Math {
//!     fn call(&self, op: &Id, args: &[f64]) -> f64 {
//!         match (op, args) {
//!             (Id::Sum, &[l, r]) => l + r,
//!             _ => unreachable!(),
//!         }
//!     }
//! }
//!
//! let value = Interpreter::compile_and_run("sum(1.0, 2.0) * 3", Math).unwrap();
//! assert_eq!(value, 9.0);
//! ```

mod callable;
mod compiler;
mod reader;
mod scanner;
pub mod interpreter;

pub use callable::{Blueprint, Callable, MethodInfo};
pub use interpreter::Interpreter;
pub use scanner::{ExprError, Token};

#[cfg(test)]
mod math {
    use crate::callable::{Blueprint, Callable, MethodInfo};

    #[derive(Debug, PartialEq)]
    pub enum MethodId {
        Sum,
        Sub,
        Foo,
    }

    pub struct Math;

    impl Blueprint<MethodId> for Math {
        fn find(&self, name: &str) -> Option<MethodInfo<MethodId>> {
            match name {
                "sum" => Some(MethodInfo::new(MethodId::Sum, 2)),
                "sub" => Some(MethodInfo::new(MethodId::Sub, 2)),
                "foo" => Some(MethodInfo::new(MethodId::Foo, 1)),
                _ => None,
            }
        }
    }

    impl Callable<MethodId> for Math {
        fn call(&self, op: &MethodId, args: &[f64]) -> f64 {
            match (op, args) {
                (MethodId::Sum, &[l, r]) => l + r,
                (MethodId::Sub, &[l, r]) => l - r,
                (MethodId::Foo, &[l]) => l,
                _ => todo!(),
            }
        }
    }
}
