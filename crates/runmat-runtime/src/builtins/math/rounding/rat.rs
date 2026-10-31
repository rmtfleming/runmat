//! Clean-room `rat` (rational approximation by continued fractions).
//!
//! Clean-room feature `specs/063-rat/`. Normative behaviour comes only from
//! the approved export `rat` 0.2.0 (see that feature's `provenance.md`).
//! Observed `MATLAB:*` identifiers are raised as `RunMat:*` (D-063-1).
//!
//! Algorithm (textbook continued fractions with nearest-integer terms,
//! U-063-1): `a_k = round(y_k)`, `y_{k+1} = 1 / (y_k - a_k)`, convergents
//! `N_k = a_k N_{k-1} + N_{k-2}`, `D_k = a_k D_{k-1} + D_{k-2}`; stop at the
//! first convergent with `|x - N_k / D_k| <= tol`, when the remainder is
//! zero, or after `MAX_TERMS` terms. The default tolerance is the
//! documented choice `1e-6 * |x|` per element (U-063-2).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{CharArray, ComplexTensor, NumericDType, Tensor, Value};

use crate::builtins::common::map_control_flow_with_builtin;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "rat";

/// U-063-1: upper bound on the number of continued-fraction terms.
const MAX_TERMS: usize = 64;

/// U-063-2: default tolerance relative to each element's magnitude.
const DEFAULT_RELATIVE_TOL: f64 = 1e-6;

// ---------------------------------------------------------------------------
// Descriptor
// ---------------------------------------------------------------------------

const fn param(
    name: &'static str,
    arity: BuiltinParamArity,
    description: &'static str,
) -> BuiltinParamDescriptor {
    BuiltinParamDescriptor {
        name,
        ty: BuiltinParamType::Any,
        arity,
        default: None,
        description,
    }
}

const P_X: BuiltinParamDescriptor =
    param("X", BuiltinParamArity::Required, "Double or single array.");
const P_TOL: BuiltinParamDescriptor = param(
    "tol",
    BuiltinParamArity::Optional,
    "Absolute tolerance per element (default 1e-6*abs(x)).",
);
const RAT_INPUTS: [BuiltinParamDescriptor; 2] = [P_X, P_TOL];
const P_N: BuiltinParamDescriptor = param(
    "N",
    BuiltinParamArity::Required,
    "Numerators, the size and class of X.",
);
const P_D: BuiltinParamDescriptor = param(
    "D",
    BuiltinParamArity::Required,
    "Denominators, the size and class of X.",
);
const P_S: BuiltinParamDescriptor = param(
    "S",
    BuiltinParamArity::Required,
    "Char matrix of continued-fraction expansions, one row per element.",
);
const RAT_OUTPUTS_ND: [BuiltinParamDescriptor; 2] = [P_N, P_D];
const RAT_OUTPUTS_S: [BuiltinParamDescriptor; 1] = [P_S];

const RAT_SIGNATURES: [BuiltinSignatureDescriptor; 3] = [
    BuiltinSignatureDescriptor {
        label: "[N, D] = rat(X)",
        inputs: &RAT_INPUTS,
        outputs: &RAT_OUTPUTS_ND,
    },
    BuiltinSignatureDescriptor {
        label: "[N, D] = rat(X, tol)",
        inputs: &RAT_INPUTS,
        outputs: &RAT_OUTPUTS_ND,
    },
    BuiltinSignatureDescriptor {
        label: "S = rat(X)",
        inputs: &RAT_INPUTS,
        outputs: &RAT_OUTPUTS_S,
    },
];

const INPUT_TYPE: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.RAT.INPUT_TYPE",
    identifier: Some("RunMat:norm:inputType"),
    when: "X is char or of an integer class.",
    message: "rat: X must be double or single",
};

const INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.RAT.INVALID_INPUT",
    identifier: Some("RunMat:rat:InvalidInput"),
    when: "X is missing, or tol is not a nonnegative real scalar.",
    message: "rat: invalid input",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.RAT.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:rat:UnsupportedOption"),
    when:
        "Unobserved forms: more than two inputs; logical, string, cell, struct, sparse and other X.",
    message: "rat: this form is not yet specified",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.RAT.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:rat:TooManyOutputs"),
    when: "More than two outputs were requested.",
    message: "rat: too many output arguments",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.RAT.INTERNAL",
    identifier: Some("RunMat:rat:Internal"),
    when: "The runtime cannot assemble the result.",
    message: "rat: internal runtime failure",
};

const RAT_ERRORS: [BuiltinErrorDescriptor; 5] = [
    INPUT_TYPE,
    INVALID_INPUT,
    UNSUPPORTED_OPTION,
    TOO_MANY_OUTPUTS,
    INTERNAL,
];

pub const RAT_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &RAT_SIGNATURES,
    output_mode: BuiltinOutputMode::ByRequestedOutputCount,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &RAT_ERRORS,
};

fn rat_error(
    descriptor: &'static BuiltinErrorDescriptor,
    detail: impl Into<String>,
) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{NAME}: {}", detail.into())).with_builtin(NAME);
    if let Some(identifier) = descriptor.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "rat",
    category = "math/rounding",
    summary = "Rational approximation N./D of each element, or its continued-fraction text.",
    keywords = "rat,rational,fraction,continued fraction,numerator,denominator",
    accel = "cpu",
    descriptor(crate::builtins::math::rounding::rat::RAT_DESCRIPTOR),
    builtin_path = "crate::builtins::math::rounding::rat"
)]
pub(crate) async fn rat_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    // U-063-5
    let count = crate::output_count::current_output_count();
    if let Some(count) = count {
        if count > 2 {
            return Err(rat_error(
                &TOO_MANY_OUTPUTS,
                format!("rat returns at most two outputs, {count} were requested"),
            ));
        }
    }
    if args.is_empty() {
        return Err(rat_error(&INVALID_INPUT, "expected rat(X) or rat(X, tol)"));
    }
    if args.len() > 2 {
        return Err(rat_error(
            &UNSUPPORTED_OPTION,
            "only rat(X) and rat(X, tol) are specified",
        ));
    }
    let mut gathered = Vec::with_capacity(args.len());
    for arg in &args {
        gathered.push(
            gather_if_needed_async(arg)
                .await
                .map_err(|err| map_control_flow_with_builtin(err, NAME))?,
        );
    }
    let input = Input::parse(&gathered[0])?;
    let tol = match gathered.get(1) {
        Some(value) => Some(parse_tol(value)?),
        None => None,
    };
    match count {
        // rat.signature-primary, rat.tol (two outputs)
        Some(2) => {
            let (n, d) = input.fractions(tol)?;
            Ok(Value::OutputList(vec![n, d]))
        }
        Some(0) => Ok(Value::OutputList(Vec::new())),
        // rat.text (one output)
        Some(_) => Ok(Value::OutputList(vec![input.text(tol)?])),
        None => input.text(tol),
    }
}

// ---------------------------------------------------------------------------
// Input handling
// ---------------------------------------------------------------------------

/// Floating-point input, widened to f64.
struct Input {
    shape: Vec<usize>,
    single: bool,
    scalar: bool,
    data: Data,
}

enum Data {
    Real(Vec<f64>),
    Complex(Vec<(f64, f64)>),
}

fn not_float() -> RuntimeError {
    // rat.error-char (D-063-1)
    rat_error(
        &INPUT_TYPE,
        "X must be double or single, not char or an integer class",
    )
}

impl Input {
    fn parse(value: &Value) -> BuiltinResult<Self> {
        match value {
            Value::Num(x) => Ok(Input {
                shape: vec![1, 1],
                single: false,
                scalar: true,
                data: Data::Real(vec![*x]),
            }),
            // rat.complex (non-normative, supported: maintainer disposition)
            Value::Complex(re, im) => Ok(Input {
                shape: vec![1, 1],
                single: false,
                scalar: true,
                data: Data::Complex(vec![(*re, *im)]),
            }),
            Value::Tensor(t) => {
                if t.integer_storage().is_some() {
                    return Err(not_float());
                }
                let single = match t.numeric_dtype() {
                    NumericDType::F64 => false,
                    NumericDType::F32 => true,
                    _ => return Err(not_float()),
                };
                Ok(Input {
                    shape: t.shape.clone(),
                    single,
                    scalar: false,
                    data: Data::Real(t.materialize_f64()),
                })
            }
            Value::ComplexTensor(t) => {
                if t.integer_storage().is_some() {
                    return Err(not_float());
                }
                let single = match t.numeric_dtype() {
                    NumericDType::F64 => false,
                    NumericDType::F32 => true,
                    _ => return Err(not_float()),
                };
                Ok(Input {
                    shape: t.shape.clone(),
                    single,
                    scalar: false,
                    data: Data::Complex(t.materialize_f64()),
                })
            }
            // rat.error-char
            Value::Int(_) | Value::CharArray(_) => Err(not_float()),
            // U-063-6
            _ => Err(rat_error(
                &UNSUPPORTED_OPTION,
                "X must be a dense double or single array",
            )),
        }
    }

    fn real_output(&self, data: Vec<f64>) -> BuiltinResult<Value> {
        if self.scalar {
            return Ok(Value::Num(data[0]));
        }
        let result = if self.single {
            Tensor::from_f32(
                data.into_iter().map(|v| v as f32).collect(),
                self.shape.clone(),
            )
        } else {
            Tensor::new(data, self.shape.clone())
        };
        result
            .map(Value::Tensor)
            .map_err(|err| rat_error(&INTERNAL, err))
    }

    fn complex_output(&self, data: Vec<(f64, f64)>) -> BuiltinResult<Value> {
        if self.scalar {
            return Ok(Value::Complex(data[0].0, data[0].1));
        }
        let result = if self.single {
            ComplexTensor::from_f32(
                data.into_iter()
                    .map(|(re, im)| (re as f32, im as f32))
                    .collect(),
                self.shape.clone(),
            )
        } else {
            ComplexTensor::new(data, self.shape.clone())
        };
        result
            .map(Value::ComplexTensor)
            .map_err(|err| rat_error(&INTERNAL, err))
    }

    /// `[N, D]` (rat.simple, rat.elementwise, rat.class, rat.nonfinite).
    fn fractions(&self, tol: Option<f64>) -> BuiltinResult<(Value, Value)> {
        match &self.data {
            Data::Real(values) => {
                let (n, d): (Vec<f64>, Vec<f64>) = values
                    .iter()
                    .map(|&x| {
                        let r = expand(x, tol);
                        (r.numerator, r.denominator)
                    })
                    .unzip();
                Ok((self.real_output(n)?, self.real_output(d)?))
            }
            Data::Complex(values) => {
                let (n, d): (Vec<(f64, f64)>, Vec<f64>) = values
                    .iter()
                    .map(|&(re, im)| complex_fraction(re, im, tol))
                    .unzip();
                Ok((self.complex_output(n)?, self.real_output(d)?))
            }
        }
    }

    /// One-output char matrix (rat.text; complex layout U-063-4).
    fn text(&self, tol: Option<f64>) -> BuiltinResult<Value> {
        let rows: Vec<String> = match &self.data {
            Data::Real(values) => values.iter().map(|&x| expand(x, tol).text()).collect(),
            Data::Complex(values) => values
                .iter()
                .flat_map(|&(re, im)| {
                    [
                        expand(re, tol).text(),
                        " +i* ...".to_string(),
                        expand(im, tol).text(),
                    ]
                })
                .collect(),
        };
        char_matrix(&rows)
    }
}

/// U-063-3: tol is a real, non-negative, non-NaN numeric scalar.
fn parse_tol(value: &Value) -> BuiltinResult<f64> {
    let tol = match value {
        Value::Num(x) => Some(*x),
        Value::Int(i) => Some(i.to_f64()),
        Value::Tensor(t) => match t.materialize_f64()[..] {
            [x] => Some(x),
            _ => None,
        },
        _ => None,
    };
    match tol {
        Some(tol) if tol >= 0.0 => Ok(tol),
        _ => Err(rat_error(
            &INVALID_INPUT,
            "tol must be a nonnegative real scalar",
        )),
    }
}

/// Rows padded with blanks on the right to a common width (U-063-4).
fn char_matrix(rows: &[String]) -> BuiltinResult<Value> {
    if rows.is_empty() {
        return CharArray::new(Vec::new(), 0, 0)
            .map(Value::CharArray)
            .map_err(|err| rat_error(&INTERNAL, err));
    }
    let width = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0);
    let mut data = Vec::with_capacity(rows.len() * width);
    for row in rows {
        let len = row.chars().count();
        data.extend(row.chars());
        data.extend(std::iter::repeat_n(' ', width - len));
    }
    CharArray::new(data, rows.len(), width)
        .map(Value::CharArray)
        .map_err(|err| rat_error(&INTERNAL, err))
}

// ---------------------------------------------------------------------------
// Continued fractions
// ---------------------------------------------------------------------------

/// A rational approximation and the terms that produced it.
struct Expansion {
    numerator: f64,
    denominator: f64,
    terms: Vec<f64>,
    special: Option<&'static str>,
}

impl Expansion {
    /// U-063-4: `a0 + 1/(a1 + 1/(a2 + ...))`; observed `1 + 1/(-4)`.
    fn text(&self) -> String {
        if let Some(special) = self.special {
            return special.to_string();
        }
        let mut out = String::new();
        for (k, term) in self.terms.iter().enumerate() {
            if k > 0 {
                out.push_str(" + 1/(");
            }
            out.push_str(&format!("{:.0}", term + 0.0));
        }
        out.push_str(&")".repeat(self.terms.len().saturating_sub(1)));
        out
    }
}

/// Expand one real value (U-063-1, U-063-2; rat.nonfinite).
fn expand(x: f64, tol: Option<f64>) -> Expansion {
    if x.is_nan() {
        // rat.nonfinite: NaN -> 0/0
        return Expansion {
            numerator: 0.0,
            denominator: 0.0,
            terms: Vec::new(),
            special: Some("NaN"),
        };
    }
    if x.is_infinite() {
        // rat.nonfinite: +-Inf -> +-1/0
        return Expansion {
            numerator: x.signum(),
            denominator: 0.0,
            terms: Vec::new(),
            special: Some(if x > 0.0 { "Inf" } else { "-Inf" }),
        };
    }
    let tol = tol.unwrap_or(DEFAULT_RELATIVE_TOL * x.abs());
    let (mut n1, mut n0) = (1.0_f64, 0.0_f64);
    let (mut d1, mut d0) = (0.0_f64, 1.0_f64);
    let mut terms = Vec::new();
    let mut y = x;
    for _ in 0..MAX_TERMS {
        let a = y.round();
        let (n, d) = (a * n1 + n0, a * d1 + d0);
        if !n.is_finite() || !d.is_finite() {
            break;
        }
        terms.push(a);
        (n0, n1) = (n1, n);
        (d0, d1) = (d1, d);
        let rest = y - a;
        if rest == 0.0 || (x - n1 / d1).abs() <= tol {
            break;
        }
        y = 1.0 / rest;
        if !y.is_finite() {
            break;
        }
    }
    // rat.simple: the sign is on N, D is positive.
    if d1 < 0.0 {
        n1 = -n1;
        d1 = -d1;
    }
    Expansion {
        numerator: n1 + 0.0,
        denominator: d1,
        terms,
        special: None,
    }
}

fn gcd(mut a: f64, mut b: f64) -> f64 {
    while b != 0.0 {
        (a, b) = (b, a % b);
    }
    a
}

/// U-063-4: parts expanded separately over the common denominator
/// `lcm(D_re, D_im)` (observed `1+2i` -> `N = 1+2i, D = 1`).
fn complex_fraction(re: f64, im: f64, tol: Option<f64>) -> ((f64, f64), f64) {
    let r = expand(re, tol);
    let i = expand(im, tol);
    let d = if r.denominator == 0.0 || i.denominator == 0.0 {
        0.0
    } else {
        r.denominator / gcd(r.denominator, i.denominator) * i.denominator
    };
    let scale = |e: &Expansion| {
        if d == 0.0 {
            if e.denominator == 0.0 {
                e.numerator
            } else {
                0.0
            }
        } else {
            e.numerator * (d / e.denominator)
        }
    };
    ((scale(&r), scale(&i)), d)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
// Observed values are copied verbatim from the imported observation record.
#[allow(clippy::excessive_precision, clippy::approx_constant)]
mod rat_tests {
    use super::rat_builtin;
    use crate::{BuiltinResult, RuntimeError};
    use futures::executor::block_on;
    use runmat_value::{CharArray, IntValue, NumericDType, Tensor, Value};
    use std::f64::consts::PI;

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("rat", args))
    }

    fn n(x: f64) -> Value {
        Value::Num(x)
    }

    fn mat(rows: usize, cols: usize, column_major: &[f64]) -> Value {
        Value::Tensor(Tensor::new(column_major.to_vec(), vec![rows, cols]).unwrap())
    }

    fn multi(args: &[Value], count: usize) -> BuiltinResult<Vec<Value>> {
        let _guard = crate::output_count::push_output_count(Some(count));
        match block_on(rat_builtin(args.to_vec()))? {
            Value::OutputList(values) => Ok(values),
            other => panic!("expected output list, got {other:?}"),
        }
    }

    /// `(dtype, shape, values)` of a real numeric result.
    fn real(value: &Value) -> (NumericDType, Vec<usize>, Vec<f64>) {
        match value {
            Value::Num(v) => (NumericDType::F64, vec![1, 1], vec![*v]),
            Value::Tensor(t) => (t.numeric_dtype(), t.shape.clone(), t.materialize_f64()),
            other => panic!("expected a real numeric result, got {other:?}"),
        }
    }

    /// [N, D] = rat(args...) as (dtype, shape, N, D).
    fn nd(args: &[Value]) -> (NumericDType, Vec<usize>, Vec<f64>, Vec<f64>) {
        let out = multi(args, 2).expect("rat");
        assert_eq!(out.len(), 2);
        let (tn, sn, vn) = real(&out[0]);
        let (td, sd, vd) = real(&out[1]);
        assert_eq!(tn, td, "N and D must have the same class");
        assert_eq!(sn, sd, "N and D must have the same size");
        (tn, sn, vn, vd)
    }

    fn scalar_nd(x: f64) -> (f64, f64) {
        let (dtype, shape, nv, dv) = nd(&[n(x)]);
        assert_eq!(dtype, NumericDType::F64);
        assert_eq!(shape, vec![1, 1]);
        (nv[0], dv[0])
    }

    fn char_rows(value: &Value) -> Vec<String> {
        let Value::CharArray(c) = value else {
            panic!("expected char, got {value:?}");
        };
        (0..c.rows)
            .map(|r| c.data[r * c.cols..(r + 1) * c.cols].iter().collect())
            .collect()
    }

    fn char_shape(value: &Value) -> (usize, usize) {
        let Value::CharArray(c) = value else {
            panic!("expected char, got {value:?}");
        };
        (c.rows, c.cols)
    }

    fn identifier(err: RuntimeError) -> String {
        err.identifier().unwrap_or("<none>").to_string()
    }

    // --- Observed (normative): one test per observed case ------------------

    /// cases `quarter`, `third`, `negative`, `integer`, `zero` —
    /// rat.signature-primary, rat.simple, rat.class
    #[test]
    fn observed_simple_values() {
        assert_eq!(scalar_nd(0.75), (3.0, 4.0));
        assert_eq!(scalar_nd(1.0 / 3.0), (1.0, 3.0));
        assert_eq!(scalar_nd(-0.5), (-1.0, 2.0));
        assert_eq!(scalar_nd(5.0), (5.0, 1.0));
        assert_eq!(scalar_nd(0.0), (0.0, 1.0));
    }

    /// cases `pi-default`, `coeff-like` — rat.default-tol
    #[test]
    fn observed_default_tolerance() {
        let (nv, dv) = scalar_nd(PI);
        assert_eq!((nv, dv), (355.0, 113.0));
        assert_eq!((nv / dv - PI).abs(), 2.667641894049666e-07);
        assert_eq!(scalar_nd(0.6667), (6667.0, 10000.0));
    }

    /// case `pi-tol` — rat.tol
    #[test]
    fn observed_explicit_tolerance() {
        let (_, _, nv, dv) = nd(&[n(PI), n(1e-2)]);
        assert_eq!((nv[0], dv[0]), (22.0, 7.0));
        assert_eq!((nv[0] / dv[0] - PI).abs(), 0.0012644892673496777);
    }

    /// cases `matrix`, `empty` — rat.elementwise
    #[test]
    fn observed_elementwise_and_empty() {
        // [0.5 0.25; 1.5 2] in column-major order.
        let (dtype, shape, nv, dv) = nd(&[mat(2, 2, &[0.5, 1.5, 0.25, 2.0])]);
        assert_eq!(dtype, NumericDType::F64);
        assert_eq!(shape, vec![2, 2]);
        assert_eq!(nv, vec![1.0, 3.0, 1.0, 2.0]);
        assert_eq!(dv, vec![2.0, 2.0, 4.0, 1.0]);
        let (dtype, shape, nv, dv) = nd(&[mat(0, 0, &[])]);
        assert_eq!(dtype, NumericDType::F64);
        assert_eq!(shape, vec![0, 0]);
        assert!(nv.is_empty() && dv.is_empty());
    }

    /// cases `inf`, `nan` — rat.nonfinite
    #[test]
    fn observed_nonfinite() {
        let (_, shape, nv, dv) = nd(&[mat(1, 2, &[f64::INFINITY, f64::NEG_INFINITY])]);
        assert_eq!(shape, vec![1, 2]);
        assert_eq!(nv, vec![1.0, -1.0]);
        assert_eq!(dv, vec![0.0, 0.0]);
        assert_eq!(scalar_nd(f64::NAN), (0.0, 0.0));
    }

    /// case `single` — rat.class
    #[test]
    fn observed_single_class() {
        let x = Value::Tensor(Tensor::from_f32(vec![0.75], vec![1, 1]).unwrap());
        let (dtype, shape, nv, dv) = nd(&[x]);
        assert_eq!(dtype, NumericDType::F32);
        assert_eq!(shape, vec![1, 1]);
        assert_eq!((nv[0], dv[0]), (3.0, 4.0));
    }

    /// cases `text-form`, `text-form-vector` — rat.text
    #[test]
    fn observed_text_form() {
        let s = call(&[n(0.75)]).unwrap();
        assert_eq!(char_shape(&s), (1, 10));
        assert_eq!(char_rows(&s), vec!["1 + 1/(-4)"]);
        let s = call(&[mat(1, 2, &[0.5, 1.5])]).unwrap();
        assert_eq!(char_shape(&s), (2, 10));
        assert_eq!(char_rows(&s), vec!["1 + 1/(-2)", "2 + 1/(-2)"]);
        // `s = rat(X)` with an explicit single output.
        let out = multi(&[n(0.75)], 1).unwrap();
        assert_eq!(char_rows(&out[0]), vec!["1 + 1/(-4)"]);
    }

    /// cases `err-char`, `err-int` — rat.error-char (D-063-1)
    #[test]
    fn rejects_char_and_integer() {
        let err = call(&[Value::CharArray(CharArray::new_row("a"))]).unwrap_err();
        assert_eq!(identifier(err), "RunMat:norm:inputType");
        let err = call(&[Value::Int(IntValue::I8(3))]).unwrap_err();
        assert_eq!(identifier(err), "RunMat:norm:inputType");
    }

    // --- Non-normative observations supported as observed -------------------

    /// cases `complex-two-out`, `complex-text-form` — rat.complex
    /// (non-normative; supported by maintainer disposition of rat.q-complex)
    #[test]
    fn supported_complex_as_observed() {
        let out = multi(&[Value::Complex(1.0, 2.0)], 2).unwrap();
        assert_eq!(out[0], Value::Complex(1.0, 2.0));
        assert_eq!(real(&out[1]), (NumericDType::F64, vec![1, 1], vec![1.0]));
        let s = call(&[Value::Complex(1.0, 2.0)]).unwrap();
        assert_eq!(char_shape(&s), (3, 8));
        assert_eq!(char_rows(&s), vec!["1       ", " +i* ...", "2       "]);
    }

    // --- Documented choices (non-normative) ---------------------------------

    /// U-063-1, U-063-2: deeper expansions, sign handling, magnitude range.
    #[test]
    fn unresolved_choice_expansion_and_default_tolerance() {
        let s = call(&[n(PI)]).unwrap();
        assert_eq!(char_rows(&s), vec!["3 + 1/(7 + 1/(16))"]);
        let s = call(&[n(-0.5)]).unwrap();
        assert_eq!(char_rows(&s), vec!["-1 + 1/(2)"]);
        assert_eq!(char_rows(&call(&[n(5.0)]).unwrap()), vec!["5"]);
        assert_eq!(char_rows(&call(&[n(-0.2)]).unwrap()), vec!["0 + 1/(-5)"]);
        // Relative default: every result is within 1e-6*|x|.
        for x in [1e-9, 2.0_f64.sqrt(), 1234.5678, -7.25e5, 1e300] {
            let (nv, dv) = scalar_nd(x);
            assert!(dv > 0.0, "{x}: D = {dv}");
            assert!((x - nv / dv).abs() <= 1e-6 * x.abs(), "{x}: {nv}/{dv}");
        }
        // Explicit tol 0 asks for the exact binary value where reachable.
        let (_, _, nv, dv) = nd(&[n(0.1), n(0.0)]);
        assert_eq!(nv[0] / dv[0], 0.1);
    }

    /// U-063-3, U-063-5, U-063-6: inputs, tolerance and output count.
    #[test]
    fn unresolved_choice_inputs_and_outputs() {
        let id = |args: &[Value]| identifier(call(args).unwrap_err());
        assert_eq!(
            identifier(block_on(rat_builtin(Vec::new())).unwrap_err()),
            "RunMat:rat:InvalidInput"
        );
        assert_eq!(id(&[n(0.5), n(-1.0)]), "RunMat:rat:InvalidInput");
        assert_eq!(id(&[n(0.5), n(f64::NAN)]), "RunMat:rat:InvalidInput");
        assert_eq!(
            id(&[n(0.5), mat(1, 2, &[1.0, 2.0])]),
            "RunMat:rat:InvalidInput"
        );
        assert_eq!(
            id(&[n(0.5), Value::CharArray(CharArray::new_row("x"))]),
            "RunMat:rat:InvalidInput"
        );
        assert_eq!(
            id(&[n(0.5), n(1e-2), n(1.0)]),
            "RunMat:rat:UnsupportedOption"
        );
        assert_eq!(id(&[Value::Bool(true)]), "RunMat:rat:UnsupportedOption");
        assert_eq!(
            id(&[Value::String("a".into())]),
            "RunMat:rat:UnsupportedOption"
        );
        let err = multi(&[n(0.5)], 3).unwrap_err();
        assert_eq!(identifier(err), "RunMat:rat:TooManyOutputs");
        assert!(multi(&[n(0.5)], 0).unwrap().is_empty());
        // Empty input, one output: 0x0 char.
        assert_eq!(char_shape(&call(&[mat(0, 0, &[])]).unwrap()), (0, 0));
        // Non-finite text rows.
        let s = call(&[mat(1, 3, &[f64::INFINITY, f64::NEG_INFINITY, f64::NAN])]).unwrap();
        assert_eq!(char_rows(&s), vec!["Inf ", "-Inf", "NaN "]);
    }

    /// U-063-4: complex arrays, common denominator, non-finite parts, single.
    #[test]
    fn unresolved_choice_complex_layout() {
        let z = Value::ComplexTensor(
            runmat_value::ComplexTensor::new(vec![(0.5, 0.25), (1.0, 0.0)], vec![1, 2]).unwrap(),
        );
        let out = multi(std::slice::from_ref(&z), 2).unwrap();
        let Value::ComplexTensor(nt) = &out[0] else {
            panic!("expected complex N, got {:?}", out[0]);
        };
        assert_eq!(nt.materialize_f64(), vec![(2.0, 1.0), (1.0, 0.0)]);
        assert_eq!(real(&out[1]).2, vec![4.0, 1.0]);
        let s = call(&[z]).unwrap();
        assert_eq!(char_shape(&s), (6, 10));
        assert_eq!(char_rows(&s)[0], "1 + 1/(-2)");
        assert_eq!(char_rows(&s)[2], "0 + 1/(4) ");
        let out = multi(&[Value::Complex(f64::INFINITY, 0.5)], 2).unwrap();
        assert_eq!(out[0], Value::Complex(1.0, 0.0));
        assert_eq!(real(&out[1]).2, vec![0.0]);
        let zs = Value::ComplexTensor(
            runmat_value::ComplexTensor::from_f32(vec![(0.75, 0.5)], vec![1, 1]).unwrap(),
        );
        let out = multi(&[zs], 2).unwrap();
        assert_eq!(real(&out[1]).0, NumericDType::F32);
    }
}
