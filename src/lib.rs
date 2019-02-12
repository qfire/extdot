//! A set of macros for using the extended dot operator.
//!
//! # Examples
//! ## In areas where expression syntax is needed.
//! ```
//! fn main() {
//!     extdot::expr!{
//!         let v: i32 = -5;
//!
//!         let v_abs = v.[this.abs()];
//!#        assert_eq!(v_abs, 5);
//!         let v_pow = v.[this.pow(2)];
//!#        assert_eq!(v_pow, 25);
//!
//!     }
//! }
//! ```
//!
//! ```
//! use serde::Deserialize;
//! use std::{ fs::File, path::Path };
//!
//! #[derive(Debug, Deserialize)]
//! struct Point {
//!   x: i32,
//!   y: i32
//! }
//!
//! fn main() -> Result<(), Box<std::error::Error>>{
//!     extdot::expr!{
//!         let point: Point =
//!           Path::new("tests/data/point.json")
//!           .[File::open]?
//!           .[dbg!]
//!           .[serde_json::from_reader]?;
//!
//!#        assert_eq!(point.x, 4);
//!#        assert_eq!(point.y, 6);
//!
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ```
//! extdot::expr!{
//!     let map = std::collections::HashMap::new().[
//!         this.insert("key1", 13),
//!         this.insert("key2", 17),
//!     ];
//!
//!#    assert_eq!(map.get("key1"), Some(&13));
//!#    assert_eq!(map.get("key2"), Some(&17));
//! }
//! ```
//!
//! ## In areas where item syntax is needed.
//! ```rust
//! use std::fmt;
//!
//! struct Example {}
//!
//! extdot::item!{
//!   impl fmt::Display for Example {
//!     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//!        let v: i32 = -7;
//!
//!        let v_abs = v.[this.abs()];
//!#       assert_eq!(v_abs, 7);
//!        let v_pow = v.[this.pow(2)];
//!#       assert_eq!(v_pow, 49);
//!
//!        write!(f, "({}, {})", v_abs, v_pow)
//!     }
//!   }
//! }
//!
//! fn main() {
//!     println!("{}", Example {});
//! }
//! ```

use proc_macro_hack::proc_macro_hack;

/// Allows the use of the extended dot notation in expressions.
#[proc_macro_hack]
pub use extdot_impl::expr;

#[doc(inline)]
pub use extdot_impl::item;
