//! Parallel clean-room builtin implementations (differential cross-validation).
//!
//! Independently-developed implementations of builtins that RunMat 0-6-0 already provides as the
//! DEFAULT. Compiled but **not registered** (their runtime_builtin/gpu/fusion attributes are stripped),
//! so they never shadow default dispatch; retained so the two independent implementations of each
//! black-box spec can be compared. See matlab-interface-spec.
//!
//! Excluded (retained as files, not compiled): `saveas` (needs plotting figure state/print infra),
//! `seconds` (needs duration-subsystem internal helpers). Vendor those helpers to re-enable.
#![allow(warnings, clippy::all)]

mod support;
pub(crate) mod text_utils;

pub mod accumarray;
pub mod append;
pub mod bounds;
pub mod datestr;
pub mod dec2bin;
pub mod display;
pub mod eps;
pub mod etime;
pub mod fileparts;
pub mod full;
pub mod histc;
pub mod int2str;
pub mod iscell;
pub mod iscellstr;
pub mod iscolumn;
pub mod isfile;
pub mod isfolder;
pub mod isobject;
pub mod isrow;
pub mod issparse;
pub mod istable;
pub mod matches;
pub mod nonzeros;
pub mod normalize;
pub mod num2cell;
pub mod spdiags;
pub mod speye;
pub mod str2num;
pub mod strncmpi;
pub mod strtok;
pub mod system;
pub mod verLessThan;
pub mod version;
pub mod what;
