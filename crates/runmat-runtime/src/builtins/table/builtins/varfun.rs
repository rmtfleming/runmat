//! Clean-room `varfun` (apply a function to each table variable).
//!
//! Clean-room feature `specs/056-table-variables/`. Normative behaviour
//! comes only from the approved export `varfun` 0.2.0 (see that feature's
//! `provenance.md`). Only the two-argument form was observed; every other
//! form raises `RunMat:varfun:UnsupportedOption`.

use super::variable_support::*;
use super::*;
use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::mapped_callable::MappedCallable;

// ---------------------------------------------------------------------------
// Descriptors
// ---------------------------------------------------------------------------

const fn param(name: &'static str, description: &'static str) -> BuiltinParamDescriptor {
    BuiltinParamDescriptor {
        name,
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description,
    }
}

const VARFUN_INPUTS: [BuiltinParamDescriptor; 2] = [
    param("func", "Function handle applied to each variable."),
    param("T", "Input table."),
];
const VARFUN_OUTPUTS: [BuiltinParamDescriptor; 1] = [param(
    "R",
    "Table of results; variables named <function>_<variable>.",
)];
const VARFUN_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "R = varfun(func, T)",
    inputs: &VARFUN_INPUTS,
    outputs: &VARFUN_OUTPUTS,
}];

const VARFUN_ERRORS: [BuiltinErrorDescriptor; 7] = [
    BuiltinErrorDescriptor {
        code: "RM.VARFUN.INVALID_INPUT",
        identifier: Some("RunMat:varfun:InvalidInput"),
        when: "The second argument is not a table.",
        message: "varfun: expected a table",
    },
    BuiltinErrorDescriptor {
        code: "RM.VARFUN.INVALID_ARGUMENT",
        identifier: Some("RunMat:varfun:InvalidArgument"),
        when: "Too few arguments, or the first argument is not a function handle.",
        message: "varfun: invalid argument",
    },
    BuiltinErrorDescriptor {
        code: "RM.VARFUN.UNSUPPORTED_OPTION",
        identifier: Some("RunMat:varfun:UnsupportedOption"),
        when: "The call uses a form the approved specification leaves unresolved.",
        message: "varfun: form not yet specified",
    },
    BuiltinErrorDescriptor {
        code: "RM.VARFUN.TOO_MANY_OUTPUTS",
        identifier: Some("RunMat:varfun:TooManyOutputs"),
        when: "More than one output was requested.",
        message: "varfun: too many outputs",
    },
    BuiltinErrorDescriptor {
        code: "RM.VARFUN.INTERNAL",
        identifier: Some("RunMat:varfun:Internal"),
        when: "RunMat cannot build the result value.",
        message: "varfun: internal error",
    },
    BuiltinErrorDescriptor {
        code: "RM.VARFUN.INVALID_FUNCTION",
        identifier: Some("RunMat:table:varfun:InvalidFunction"),
        when: "The first argument is numeric or logical.",
        message: "varfun: first argument must be a function handle",
    },
    BuiltinErrorDescriptor {
        code: "RM.VARFUN.FUN_FAILED",
        identifier: Some("RunMat:table:varfun:FunFailed"),
        when: "The function failed on a variable.",
        message: "varfun: the function failed on a variable",
    },
];

const INVALID_FUNCTION: &str = "INVALID_FUNCTION";
const FUN_FAILED: &str = "FUN_FAILED";

pub const VARFUN_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &VARFUN_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &VARFUN_ERRORS,
};

const CTX: Ctx = Ctx {
    name: "varfun",
    errors: &VARFUN_ERRORS,
};

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "varfun",
    category = "table",
    summary = "Apply a function to each variable of a table.",
    keywords = "varfun,table,variables,function,apply",
    accel = "cpu",
    descriptor(crate::builtins::table::builtins::varfun::VARFUN_DESCRIPTOR),
    builtin_path = "crate::builtins::table::builtins"
)]
pub(crate) async fn varfun_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    CTX.check_outputs()?;
    match args.len() {
        0 | 1 => return Err(CTX.error(INVALID_ARGUMENT, "expected varfun(func, T)")),
        2 => {}
        // U-056-10 (varfun.q-more-args).
        _ => {
            return Err(CTX.error(
                UNSUPPORTED_OPTION,
                "only varfun(func, T) is specified; options are not yet supported",
            ))
        }
    }
    let func = gathered(&args[0]).await?;
    let prefix = function_prefix(&func)?;
    let object = CTX.table_input(&args[1]).await?;
    let callable = MappedCallable::parse(func)
        .map_err(|err| CTX.error(INVALID_ARGUMENT, format!("invalid function: {err:?}")))?;

    let (names, columns) = columns_of(&object)?;
    let mut out_names = Vec::with_capacity(names.len());
    let mut results = Vec::with_capacity(names.len());
    let mut height: Option<usize> = None;
    for (name, column) in names.iter().zip(columns) {
        // varfun.error-cellstr: any failure of func is FunFailed.
        let result = callable.invoke(&[column]).await.map_err(|err| {
            let detail = match err {
                crate::builtins::common::mapped_callable::CallableCallError::Runtime(err) => {
                    err.message().to_string()
                }
                _ => "the function could not be called".to_string(),
            };
            CTX.error(
                FUN_FAILED,
                format!("applying the function to variable '{name}' failed: {detail}"),
            )
        })?;
        let result = scalar_as_array(first_output(result).ok_or_else(|| {
            CTX.error(
                FUN_FAILED,
                format!("the function returned no value for variable '{name}'"),
            )
        })?);
        let rows = value_row_count(&result)?;
        match height {
            None => height = Some(rows),
            Some(expected) if expected != rows => {
                // U-056-12.
                return Err(CTX.error(
                    UNSUPPORTED_OPTION,
                    "function results with different numbers of rows are not yet specified",
                ));
            }
            Some(_) => {}
        }
        out_names.push(format!("{prefix}_{name}"));
        results.push(result);
    }
    table_from_columns(out_names, results)
        .map_err(|err| CTX.error(INTERNAL, err.message().to_string()))
}

/// The first value of a call result (builtins with several outputs return
/// an output list).
pub(super) fn first_output(value: Value) -> Option<Value> {
    match value {
        Value::OutputList(values) => values.into_iter().next(),
        other => Some(other),
    }
}

/// A scalar double or logical result as a 1-by-1 array, the storage the
/// other table builtins (e.g. `table2array`) expect of a variable.
fn scalar_as_array(value: Value) -> Value {
    match value {
        Value::Num(n) => Tensor::new(vec![n], vec![1, 1])
            .map(Value::Tensor)
            .unwrap_or(Value::Num(n)),
        Value::Bool(b) => LogicalArray::new(vec![u8::from(b)], vec![1, 1])
            .map(Value::LogicalArray)
            .unwrap_or(Value::Bool(b)),
        other => other,
    }
}

/// varfun.names / U-056-11: the name prefix for a function value; numeric
/// and logical values are InvalidFunction (varfun.error-not-function).
fn function_prefix(func: &Value) -> BuiltinResult<String> {
    let raw = match func {
        Value::FunctionHandle(name)
        | Value::ExternalFunctionHandle(name)
        | Value::MethodFunctionHandle(name)
        | Value::BoundFunctionHandle { name, .. } => name.clone(),
        Value::Closure(closure) => closure.function_name.clone(),
        Value::Num(_)
        | Value::Int(_)
        | Value::Bool(_)
        | Value::Tensor(_)
        | Value::ComplexTensor(_)
        | Value::Complex(..)
        | Value::LogicalArray(_) => {
            return Err(CTX.error(
                INVALID_FUNCTION,
                "the first argument must be a function handle",
            ))
        }
        Value::CharArray(_) | Value::String(_) | Value::StringArray(_) => {
            return Err(CTX.error(
                UNSUPPORTED_OPTION,
                "a function given by name is not yet specified; use a function handle",
            ))
        }
        other => {
            return Err(CTX.error(
                INVALID_ARGUMENT,
                format!("expected a function handle, got {}", class_label(other)),
            ))
        }
    };
    let trimmed = raw.trim();
    let trimmed = trimmed.strip_prefix('@').unwrap_or(trimmed);
    let last = trimmed.rsplit('.').next().unwrap_or(trimmed);
    let is_identifier = last.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        && last.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    Ok(if is_identifier {
        last.to_string()
    } else {
        "Fun".to_string()
    })
}

#[cfg(test)]
mod varfun_tests {
    use super::super::variable_support::test_helpers::*;
    use super::*;
    use futures::executor::block_on;
    use runmat_value::Closure;
    use std::sync::Arc;

    /// `table([1;2;3],[4;6;8],'VariableNames',{'x','y'})`.
    fn xy() -> Value {
        table(
            vec![column(&[1.0, 2.0, 3.0]), column(&[4.0, 6.0, 8.0])],
            &["x", "y"],
        )
    }

    fn handle(name: &str) -> Value {
        Value::FunctionHandle(name.into())
    }

    fn varfun(func: Value, t: Value) -> Value {
        ok("varfun", &[func, t])
    }

    fn assert_column(out: &Value, name: &str, values: &[f64]) {
        let value = variable(out, name);
        assert_eq!(class_of(&value), "double");
        let (shape, data) = doubles(&value);
        assert_eq!(shape, vec![values.len(), 1]);
        assert_eq!(data, values.to_vec());
    }

    // ---- observed (normative) ---------------------------------------------

    /// varfun.signature-primary, varfun.names, varfun.rows (case `mean`).
    #[test]
    fn observed_mean() {
        let out = varfun(handle("mean"), xy());
        assert_eq!(names(&out), sv(&["mean_x", "mean_y"]));
        assert_eq!(size(&out), (1, 2));
        assert_column(&out, "mean_x", &[2.0]);
        assert_column(&out, "mean_y", &[6.0]);
    }

    /// varfun.names, varfun.rows (case `std`).
    #[test]
    fn observed_std() {
        let out = varfun(handle("std"), xy());
        assert_eq!(names(&out), sv(&["std_x", "std_y"]));
        assert_column(&out, "std_x", &[1.0]);
        assert_column(&out, "std_y", &[2.0]);
    }

    /// varfun.names, varfun.rows (case `elementwise`).
    #[test]
    fn observed_cumsum_column_results() {
        let out = varfun(handle("cumsum"), xy());
        assert_eq!(names(&out), sv(&["cumsum_x", "cumsum_y"]));
        assert_eq!(size(&out), (3, 2));
        assert_column(&out, "cumsum_x", &[1.0, 3.0, 6.0]);
        assert_column(&out, "cumsum_y", &[4.0, 10.0, 18.0]);
    }

    /// varfun.names (case `one-var`).
    #[test]
    fn observed_one_var_max() {
        let t = table(vec![column(&[5.0, 2.0])], &["v"]);
        let out = varfun(handle("max"), t);
        assert_eq!(names(&out), sv(&["max_v"]));
        assert_column(&out, "max_v", &[5.0]);
    }

    /// varfun.names, varfun.rows (case `anonymous`): `@(v) v * 2`, modelled
    /// as an anonymous closure served by a semantic invoker.
    #[test]
    fn observed_anonymous_function() {
        let _invoker = crate::user_functions::install_semantic_function_invoker(Some(Arc::new(
            |function, arguments, _outputs| {
                assert_eq!(function, 5601);
                let arguments = arguments.to_vec();
                Box::pin(async move {
                    let doubled = crate::call_builtin_async(
                        "times",
                        &[arguments[0].clone(), Value::Num(2.0)],
                    )
                    .await?;
                    crate::sequence::single_value_sequence(doubled)
                })
            },
        )));
        let anonymous = Value::Closure(Closure {
            function_name: "anonymous#3".into(),
            bound_function: Some(5601),
            captures: Vec::new(),
        });
        let out = varfun(anonymous, xy());
        assert_eq!(names(&out), sv(&["Fun_x", "Fun_y"]));
        assert_eq!(size(&out), (3, 2));
        assert_column(&out, "Fun_x", &[2.0, 4.0, 6.0]);
        assert_column(&out, "Fun_y", &[8.0, 12.0, 16.0]);
    }

    /// varfun.cobra-pattern (case `table2array`).
    #[test]
    fn observed_table2array_of_result() {
        let array = ok("table2array", &[varfun(handle("mean"), xy())]);
        let transposed = ok("transpose", &[array]);
        assert_eq!(doubles(&transposed), (vec![2, 1], vec![2.0, 6.0]));
    }

    /// varfun.cobra-pattern (case `subset`): `T([1 3], 2:end)` is the table
    /// `y = [4;8]`, built directly.
    #[test]
    fn observed_subset() {
        let t = table(vec![column(&[4.0, 8.0])], &["y"]);
        let out = varfun(handle("mean"), t);
        assert_eq!(names(&out), sv(&["mean_y"]));
        assert_column(&out, "mean_y", &[6.0]);
    }

    /// varfun.rownames (case `rownames`).
    #[test]
    fn observed_row_names_scalar_result() {
        let t = table_rows(vec![column(&[1.0, 3.0])], &["x"], &["r1", "r2"]);
        let out = varfun(handle("mean"), t);
        assert_eq!(names(&out), sv(&["mean_x"]));
        assert_eq!(size(&out), (1, 1));
        assert_column(&out, "mean_x", &[2.0]);
    }

    /// varfun.empty-rows (case `empty-rows`).
    #[test]
    fn observed_empty_rows_sum() {
        let empty = Value::Tensor(runmat_value::Tensor::new(Vec::new(), vec![0, 1]).unwrap());
        let t = table(vec![empty], &["x"]);
        let out = varfun(handle("sum"), t);
        assert_eq!(names(&out), sv(&["sum_x"]));
        assert_eq!(size(&out), (1, 1));
        assert_column(&out, "sum_x", &[0.0]);
    }

    /// varfun.error-cellstr (case `err-cellstr`).
    #[test]
    fn observed_error_function_failed() {
        let t = table(vec![cellcol(&["a", "b"])], &["s"]);
        assert_eq!(
            err_id("varfun", &[handle("mean"), t]),
            "RunMat:table:varfun:FunFailed"
        );
    }

    /// varfun.error-not-function (case `err-not-function`).
    #[test]
    fn observed_error_not_function() {
        assert_eq!(
            err_id("varfun", &[Value::Num(5.0), xy()]),
            "RunMat:table:varfun:InvalidFunction"
        );
    }

    // ---- documented choices (FR-056-40, 49..51) ----------------------------

    /// U-056-1 (varfun.non-table is non-normative), arity.
    #[test]
    fn rejects_non_table_and_arity() {
        let m =
            Value::Tensor(runmat_value::Tensor::new(vec![1.0, 3.0, 2.0, 4.0], vec![2, 2]).unwrap());
        assert_eq!(
            err_id("varfun", &[handle("mean"), m]),
            "RunMat:varfun:InvalidInput"
        );
        assert_eq!(
            err_id("varfun", &[handle("mean")]),
            "RunMat:varfun:InvalidArgument"
        );
    }

    /// U-056-1: one output (called directly; the dispatcher retries).
    #[test]
    fn unresolved_too_many_outputs() {
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(varfun_builtin(vec![handle("mean"), xy()])).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:varfun:TooManyOutputs"));
    }

    /// U-056-10 (varfun.q-more-args): name-value options are not specified.
    #[test]
    fn unresolved_more_args_unsupported() {
        for option in ["InputVariables", "OutputFormat", "GroupingVariables"] {
            assert_eq!(
                err_id("varfun", &[handle("mean"), xy(), text(option), text("x")]),
                "RunMat:varfun:UnsupportedOption"
            );
        }
    }

    /// U-056-11: function argument forms and derived names.
    #[test]
    fn documented_choice_function_argument() {
        assert_eq!(
            err_id("varfun", &[Value::Bool(true), xy()]),
            "RunMat:table:varfun:InvalidFunction"
        );
        assert_eq!(
            err_id("varfun", &[text("mean"), xy()]),
            "RunMat:varfun:UnsupportedOption"
        );
        assert_eq!(
            err_id("varfun", &[cellstr(&["mean"]), xy()]),
            "RunMat:varfun:InvalidArgument"
        );
        let out = varfun(handle("@mean"), xy());
        assert_eq!(names(&out), sv(&["mean_x", "mean_y"]));
    }

    /// U-056-12: results of different heights; no row names on output;
    /// a table without variables.
    #[test]
    fn documented_choice_results() {
        let t = table(
            vec![column(&[1.0, 2.0, 3.0]), column(&[4.0, 6.0, 8.0])],
            &["x", "y"],
        );
        // nonzeros gives 3 rows for x and 3 for y: same heights, fine.
        let out = varfun(handle("nonzeros"), t);
        assert_eq!(size(&out), (3, 2));
        let uneven = table(
            vec![column(&[1.0, 0.0, 3.0]), column(&[4.0, 6.0, 8.0])],
            &["x", "y"],
        );
        assert_eq!(
            err_id("varfun", &[handle("nonzeros"), uneven]),
            "RunMat:varfun:UnsupportedOption"
        );
        let t = table_rows(vec![column(&[1.0, 3.0])], &["x"], &["r1", "r2"]);
        let out = varfun(handle("cumsum"), t);
        assert!(property_strings(&out, ROW_NAMES).is_empty());
        let empty = table_from_columns(Vec::new(), Vec::new()).unwrap();
        let out = varfun(handle("mean"), empty);
        assert_eq!(size(&out), (0, 0));
    }
}
