pub use super::hist;

// Clean-room builtins (specs 040, 041). `plotting/mod.rs` is generated, so
// these hand-written modules are declared here instead.
#[path = "exportgraphics.rs"]
pub(crate) mod exportgraphics;
#[path = "spy.rs"]
pub(crate) mod spy;
