pub use super::hist;

// Clean-room builtins (specs 040, 041). `plotting/mod.rs` is generated, so
// these hand-written modules are declared here instead.
#[path = "exportgraphics.rs"]
pub(crate) mod exportgraphics;
#[path = "spy.rs"]
pub(crate) mod spy;

// Clean-room builtins (spec 051).
#[path = "alpha.rs"]
pub(crate) mod alpha;
#[path = "tiledlayout.rs"]
pub(crate) mod tiledlayout;

// Clean-room builtins (spec 058).
#[path = "allchild.rs"]
pub(crate) mod allchild;
#[path = "findall.rs"]
pub(crate) mod findall;
#[path = "ishghandle.rs"]
pub(crate) mod ishghandle;
#[path = "pcolor.rs"]
pub(crate) mod pcolor;
#[path = "surfl.rs"]
pub(crate) mod surfl;

// Clean-room builtins (spec 065).
#[path = "fplot.rs"]
pub(crate) mod fplot;
#[path = "rectangle.rs"]
pub(crate) mod rectangle;
#[path = "validatecolor.rs"]
pub(crate) mod validatecolor;
