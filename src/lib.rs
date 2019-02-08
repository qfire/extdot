use proc_macro_hack::proc_macro_hack;

/// Allows the use of the extended dot notation in expressions.
///
/// # Examples
/// ```
/// fn main() {
///     extdot::expr!{
///         let v: i32 = -5;
///
///         let v_abs = v.[it.abs()];
///#        assert_eq!(v_abs, 5);
///         let v_pow = v.[it.pow(2)];
///#        assert_eq!(v_pow, 25);
///
///     }
/// }
/// ```
///
/// ```
/// use serde::Deserialize;
/// use std::{ fs::File, path::Path };
///
/// #[derive(Debug, Deserialize)]
/// struct Point {
///   x: i32,
///   y: i32
/// }
///
/// fn main() {
///     extdot::expr!{
///         let point: Point =
///           Path::new("tests/data/point.json")
///           .[File::open].expect("file not found")
///           .[serde_json::from_reader].expect("error while reading json");
///
///#        assert_eq!(point.x, 4);
///#        assert_eq!(point.y, 6);
///     }
/// }
/// ```
#[proc_macro_hack]
pub use extdot_impl::expr;

#[doc(inline)]
pub use extdot_impl::item;
