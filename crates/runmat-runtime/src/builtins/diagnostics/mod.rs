//! Diagnostics-oriented builtins (error handling, warnings, assertions, ...).

pub mod assert;
pub mod error;
#[path = "getReport.rs"]
pub mod get_report;
pub mod rethrow;
pub(crate) mod type_resolvers;
pub mod warning;
