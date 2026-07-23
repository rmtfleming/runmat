//! Diagnostics-oriented builtins (error handling, warnings, assertions, ...).

pub mod assert;
pub mod error;
pub mod rethrow;
pub mod runtests;
pub(crate) mod type_resolvers;
pub mod warning;
#[path = "getReport.rs"]
pub mod get_report;
