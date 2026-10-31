//! `validatecolor` builtin: validate colours and return them as RGB rows
//! (spec 065).
//!
//! Normative behaviour comes from the approved `validatecolor` 0.2.0 export
//! (claims `validatecolor.signature-primary`, `validatecolor.cobra-form`,
//! `validatecolor.names`, `validatecolor.triplets`, `validatecolor.hex`,
//! `validatecolor.multiple` and the five `validatecolor.error-*` claims).
//! Colour names and hex codes are recognised by RunMat's plotting colour
//! table (`style::color_from_name_or_token`); components are recovered
//! exactly as `k/255` doubles. The single-colour parser is shared with
//! `rectangle`. See `specs/065-graphics-n5/`.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{NumericDType, Tensor, Value};

use crate::builtins::plotting::style::color_from_name_or_token;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const BUILTIN_NAME: &str = "validatecolor";

const VC_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "RGB",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Colours as double RGB rows with values in [0, 1].",
}];
const VC_INPUT_C: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "c",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description:
        "Colour name, hexadecimal code, RGB triplet, or (with 'multiple') several colours.",
};
const VC_INPUT_OPTION: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "option",
    ty: BuiltinParamType::StringScalar,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "'one' (default) or 'multiple'.",
};
const VC_INPUTS_ONE: [BuiltinParamDescriptor; 1] = [VC_INPUT_C];
const VC_INPUTS_OPTION: [BuiltinParamDescriptor; 2] = [VC_INPUT_C, VC_INPUT_OPTION];
const VC_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "RGB = validatecolor(c)",
        inputs: &VC_INPUTS_ONE,
        outputs: &VC_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "RGB = validatecolor(c, option)",
        inputs: &VC_INPUTS_OPTION,
        outputs: &VC_OUTPUT,
    },
];

pub(crate) const VC_ERROR_MULTIPLE: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.VALIDATECOLOR.MULTIPLE_COLORS",
    identifier: Some("RunMat:graphics:validatecolor:MultipleColors"),
    when: "Several colours are given without the 'multiple' option.",
    message: "validatecolor: several colours require the 'multiple' option",
};
pub(crate) const VC_ERROR_COLOR_STRING: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.VALIDATECOLOR.INVALID_COLOR_STRING",
    identifier: Some("RunMat:graphics:validatecolor:InvalidColorString"),
    when: "Text that is not a colour name or hexadecimal colour code.",
    message: "validatecolor: invalid colour name",
};
pub(crate) const VC_ERROR_OUT_OF_RANGE: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.VALIDATECOLOR.OUT_OF_RANGE",
    identifier: Some("RunMat:graphics:validatecolor:OutOfRange"),
    when: "A floating-point colour component is outside [0, 1] or NaN.",
    message: "validatecolor: colour components must be in the range [0, 1]",
};
pub(crate) const VC_ERROR_TRIPLET_SHAPE: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.VALIDATECOLOR.INVALID_TRIPLET_SHAPE",
    identifier: Some("RunMat:graphics:validatecolor:InvalidTripletShape"),
    when: "A numeric colour is not a three-element triplet (or an m-by-3 matrix with 'multiple').",
    message: "validatecolor: an RGB triplet must have three elements",
};
const VC_ERROR_BAD_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.VALIDATECOLOR.UNRECOGNIZED_STRING_CHOICE",
    identifier: Some("RunMat:unrecognizedStringChoice"),
    when: "The second argument is neither 'one' nor 'multiple'.",
    message: "validatecolor: the option must be 'one' or 'multiple'",
};
pub(crate) const VC_ERROR_UNSUPPORTED: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.VALIDATECOLOR.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:validatecolor:UnsupportedOption"),
    when: "An input class the approved specification does not cover (logical, complex, integer classes other than uint8/uint16, multi-row char, other values).",
    message: "validatecolor: unsupported input",
};
const VC_ERROR_MINRHS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.VALIDATECOLOR.MINRHS",
    identifier: Some("RunMat:minrhs"),
    when: "No argument is given.",
    message: "validatecolor: not enough input arguments",
};
const VC_ERROR_MAXRHS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.VALIDATECOLOR.MAXRHS",
    identifier: Some("RunMat:maxrhs"),
    when: "More than two arguments are given.",
    message: "validatecolor: too many input arguments",
};
const VC_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.VALIDATECOLOR.INTERNAL",
    identifier: Some("RunMat:validatecolor:Internal"),
    when: "An internal operation fails.",
    message: "validatecolor: internal operation failed",
};
const VC_ERRORS: [BuiltinErrorDescriptor; 9] = [
    VC_ERROR_MULTIPLE,
    VC_ERROR_COLOR_STRING,
    VC_ERROR_OUT_OF_RANGE,
    VC_ERROR_TRIPLET_SHAPE,
    VC_ERROR_BAD_OPTION,
    VC_ERROR_UNSUPPORTED,
    VC_ERROR_MINRHS,
    VC_ERROR_MAXRHS,
    VC_ERROR_INTERNAL,
];

pub const VALIDATECOLOR_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &VC_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &VC_ERRORS,
};

#[runtime_builtin(
    name = "validatecolor",
    category = "plotting",
    summary = "Validate colour names, hexadecimal codes and RGB triplets; return RGB rows.",
    keywords = "validatecolor,color,colour,rgb,hex,validate",
    descriptor(crate::builtins::plotting::ops::validatecolor::VALIDATECOLOR_DESCRIPTOR),
    builtin_path = "crate::builtins::plotting::ops::validatecolor"
)]
pub async fn validatecolor_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    let mut gathered = Vec::with_capacity(args.len());
    for value in &args {
        gathered.push(
            gather_if_needed_async(value)
                .await
                .map_err(|err| vc_error(&VC_ERROR_INTERNAL, err.message()))?,
        );
    }
    match gathered.len() {
        0 => return Err(vc_error(&VC_ERROR_MINRHS, "a colour is required")),
        1 | 2 => {}
        _ => return Err(vc_error(&VC_ERROR_MAXRHS, "at most two arguments")),
    }
    let multiple = match gathered.get(1) {
        None => false,
        Some(option) => match text_of(option).map(|t| t.trim().to_ascii_lowercase()) {
            Some(t) if t == "one" => false,
            Some(t) if t == "multiple" => true,
            _ => {
                return Err(vc_error(
                    &VC_ERROR_BAD_OPTION,
                    "expected 'one' or 'multiple'",
                ))
            }
        },
    };
    let rows = colour_rows(&gathered[0], multiple)?;
    let count = rows.len();
    let mut data = vec![0.0; count * 3];
    for (i, row) in rows.iter().enumerate() {
        for (j, component) in row.iter().enumerate() {
            data[j * count + i] = *component;
        }
    }
    Tensor::new(data, vec![count, 3])
        .map(Value::Tensor)
        .map_err(|e| vc_error(&VC_ERROR_INTERNAL, e))
}

/// All colours of `value` as RGB rows. Without `multiple`, exactly one.
fn colour_rows(value: &Value, multiple: bool) -> BuiltinResult<Vec<[f64; 3]>> {
    let rows = match value {
        Value::Cell(cell) => cell
            .data
            .iter()
            .map(|element| parse_single_color(element))
            .collect::<BuiltinResult<Vec<_>>>()?,
        Value::StringArray(sa) => sa
            .data
            .iter()
            .map(|s| text_color(s))
            .collect::<BuiltinResult<Vec<_>>>()?,
        Value::Num(_) | Value::Int(_) | Value::Tensor(_) => {
            let (t, scale) = numeric_colour_tensor(value)?;
            let (r, c) = (t.rows(), t.cols());
            let is_vector = t.shape.len() <= 2 && (r == 1 || c == 1);
            if multiple && !(is_vector && t.len() == 3) {
                if t.shape.iter().skip(2).any(|&d| d != 1) || c != 3 {
                    if t.is_empty() {
                        return Ok(Vec::new());
                    }
                    return Err(vc_error(
                        &VC_ERROR_TRIPLET_SHAPE,
                        "with 'multiple', numeric colours must be an m-by-3 matrix",
                    ));
                }
                let data = t.materialize_f64();
                (0..r)
                    .map(|i| scaled_triplet([data[i], data[r + i], data[2 * r + i]], scale))
                    .collect::<BuiltinResult<Vec<_>>>()?
            } else {
                if !multiple && c == 3 && r > 1 && t.shape.len() <= 2 {
                    return Err(vc_error(
                        &VC_ERROR_MULTIPLE,
                        format!("a {r}-by-3 matrix holds {r} colours"),
                    ));
                }
                vec![parse_single_color(value)?]
            }
        }
        _ => vec![parse_single_color(value)?],
    };
    if !multiple && rows.len() != 1 {
        return Err(vc_error(
            &VC_ERROR_MULTIPLE,
            format!("{} colours were given", rows.len()),
        ));
    }
    Ok(rows)
}

/// Parse one colour (name, hex code or triplet) as an exact double RGB
/// triple. Shared with `rectangle` (spec 065, FR-065-24).
pub(crate) fn parse_single_color(value: &Value) -> BuiltinResult<[f64; 3]> {
    if let Value::CharArray(ca) = value {
        if ca.rows > 1 {
            return Err(vc_error(
                &VC_ERROR_UNSUPPORTED,
                "multi-row char arrays are not supported",
            ));
        }
    }
    if let Some(text) = text_of(value) {
        return text_color(&text);
    }
    match value {
        Value::Num(_) | Value::Int(_) | Value::Tensor(_) => {
            let (t, scale) = numeric_colour_tensor(value)?;
            if t.len() != 3 || !(t.shape.len() <= 2 && (t.rows() == 1 || t.cols() == 1)) {
                return Err(vc_error(
                    &VC_ERROR_TRIPLET_SHAPE,
                    format!("got {} element(s)", t.len()),
                ));
            }
            let d = t.materialize_f64();
            scaled_triplet([d[0], d[1], d[2]], scale)
        }
        Value::Cell(cell) if cell.data.len() == 1 => parse_single_color(&cell.data[0]),
        Value::Cell(cell) => Err(vc_error(
            &VC_ERROR_MULTIPLE,
            format!("{} colours were given", cell.data.len()),
        )),
        Value::StringArray(sa) => Err(vc_error(
            &VC_ERROR_MULTIPLE,
            format!("{} colours were given", sa.data.len()),
        )),
        _ => Err(vc_error(
            &VC_ERROR_UNSUPPORTED,
            "expected a colour name, hexadecimal code or RGB triplet",
        )),
    }
}

/// A colour name or hex code. RunMat's plotting colour table holds every
/// component as `f32`; all table entries are `k/255`, so `k` is recovered
/// exactly and the double `k/255` returned.
fn text_color(text: &str) -> BuiltinResult<[f64; 3]> {
    let trimmed = text.trim();
    let color = color_from_name_or_token(trimmed).ok_or_else(|| {
        vc_error(
            &VC_ERROR_COLOR_STRING,
            format!("'{trimmed}' is not a colour name or hexadecimal code"),
        )
    })?;
    let exact = |c: f32| (c as f64 * 255.0).round() / 255.0;
    Ok([exact(color.x), exact(color.y), exact(color.z)])
}

/// Numeric input as a tensor plus the integer scale (None for floating
/// point). uint8 divides by 255 (observed), uint16 by 65535 (choice).
fn numeric_colour_tensor(value: &Value) -> BuiltinResult<(Tensor, Option<f64>)> {
    let scalar =
        |v: f64| Tensor::new(vec![v], vec![1, 1]).map_err(|e| vc_error(&VC_ERROR_INTERNAL, e));
    let t = match value {
        Value::Num(v) => return Ok((scalar(*v)?, None)),
        Value::Int(i) => {
            let scale = match i {
                runmat_value::IntValue::U8(_) => Some(255.0),
                runmat_value::IntValue::U16(_) => Some(65535.0),
                _ => {
                    return Err(vc_error(
                        &VC_ERROR_UNSUPPORTED,
                        "only uint8 and uint16 integer colours are supported",
                    ))
                }
            };
            return Ok((scalar(i.to_f64())?, scale));
        }
        _ => Tensor::try_from(value).map_err(|e| vc_error(&VC_ERROR_UNSUPPORTED, e))?,
    };
    let scale = match t.numeric_dtype() {
        NumericDType::F64 | NumericDType::F32 => None,
        NumericDType::U8 => Some(255.0),
        NumericDType::U16 => Some(65535.0),
        _ => {
            return Err(vc_error(
                &VC_ERROR_UNSUPPORTED,
                "only uint8 and uint16 integer colours are supported",
            ))
        }
    };
    Ok((t, scale))
}

fn scaled_triplet(c: [f64; 3], scale: Option<f64>) -> BuiltinResult<[f64; 3]> {
    if let Some(scale) = scale {
        return Ok([c[0] / scale, c[1] / scale, c[2] / scale]);
    }
    if c.iter().any(|v| !(0.0..=1.0).contains(v)) {
        return Err(vc_error(
            &VC_ERROR_OUT_OF_RANGE,
            format!("got [{} {} {}]", c[0], c[1], c[2]),
        ));
    }
    Ok(c)
}

fn text_of(value: &Value) -> Option<String> {
    match value {
        Value::CharArray(ca) if ca.rows <= 1 => Some(ca.data.iter().collect()),
        Value::String(s) => Some(s.clone()),
        Value::StringArray(sa) if sa.data.len() == 1 => Some(sa.data[0].clone()),
        _ => None,
    }
}

pub(crate) fn vc_error(
    error: &'static BuiltinErrorDescriptor,
    detail: impl AsRef<str>,
) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{}: {}", error.message, detail.as_ref()))
        .with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

/// Spec 065 (`validatecolor` 0.2.0). `observed_*` tests reproduce confirmed
/// observations; `unresolved_choice_*` tests pin RunMat's documented
/// choices and are not claims about MATLAB.
#[cfg(test)]
mod validatecolor_tests {
    use super::validatecolor_builtin;
    use crate::{BuiltinResult, RuntimeError};
    use futures::executor::block_on;
    use runmat_value::{
        CellArray, CharArray, ComplexTensor, IntegerStorage, StringArray, Tensor, Value,
    };

    fn direct(args: &[Value]) -> BuiltinResult<Value> {
        block_on(validatecolor_builtin(args.to_vec()))
    }

    fn text(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
    }

    fn row(data: &[f64]) -> Value {
        Value::Tensor(Tensor::new(data.to_vec(), vec![1, data.len()]).expect("row"))
    }

    fn matrix(rows: &[&[f64]]) -> Value {
        let (m, n) = (rows.len(), rows[0].len());
        let mut data = Vec::with_capacity(m * n);
        for j in 0..n {
            for r in rows {
                data.push(r[j]);
            }
        }
        Value::Tensor(Tensor::new(data, vec![m, n]).expect("matrix"))
    }

    fn cell(values: Vec<Value>) -> Value {
        let n = values.len();
        Value::Cell(CellArray::new(values, 1, n).expect("cell"))
    }

    /// (shape, column-major data, is double)
    fn shaped(value: &Value) -> (Vec<usize>, Vec<f64>) {
        match value {
            Value::Tensor(t) => {
                assert_eq!(t.numeric_dtype(), runmat_value::NumericDType::F64, "double");
                (vec![t.rows(), t.cols()], t.materialize_f64().to_vec())
            }
            other => panic!("expected a double matrix, got {other:?}"),
        }
    }

    fn rgb(value: Value) -> Vec<f64> {
        let (shape, data) = shaped(&value);
        assert_eq!(shape, vec![1, 3]);
        data
    }

    fn identifier(err: &RuntimeError) -> String {
        err.identifier().unwrap_or_default().to_string()
    }

    // ---- observed (normative) ----

    /// [validatecolor.signature-primary, validatecolor.cobra-form] cases
    /// `short-r`, `cobra-any`.
    #[test]
    fn observed_short_name_and_cobra_any() {
        assert_eq!(rgb(direct(&[text("r")]).unwrap()), vec![1.0, 0.0, 0.0]);
        let any = block_on(crate::call_builtin_async(
            "any",
            &[direct(&[text("r")]).unwrap()],
        ))
        .expect("any");
        assert!(
            matches!(any, Value::Bool(true)),
            "any(...) is true: {any:?}"
        );
    }

    /// [validatecolor.names] cases `long-red`, `string-blue`, `mixed-case`,
    /// `short-k`.
    #[test]
    fn observed_names() {
        assert_eq!(rgb(direct(&[text("red")]).unwrap()), vec![1.0, 0.0, 0.0]);
        assert_eq!(
            rgb(direct(&[Value::String("blue".into())]).unwrap()),
            vec![0.0, 0.0, 1.0]
        );
        assert_eq!(rgb(direct(&[text("Green")]).unwrap()), vec![0.0, 1.0, 0.0]);
        assert_eq!(rgb(direct(&[text("k")]).unwrap()), vec![0.0, 0.0, 0.0]);
    }

    /// [validatecolor.triplets] cases `triplet-double`, `triplet-single`,
    /// `triplet-uint8`.
    #[test]
    fn observed_triplets() {
        assert_eq!(
            rgb(direct(&[row(&[0.2, 0.4, 0.6])]).unwrap()),
            vec![0.2, 0.4, 0.6]
        );
        let single = Value::Tensor(
            Tensor::new_with_dtype(
                vec![0.5, 0.25, 1.0],
                vec![1, 3],
                runmat_value::NumericDType::F32,
            )
            .expect("single"),
        );
        assert_eq!(rgb(direct(&[single]).unwrap()), vec![0.5, 0.25, 1.0]);
        let u8v = Value::Tensor(
            Tensor::new_integer(IntegerStorage::U8(vec![255, 128, 0]), vec![1, 3]).expect("uint8"),
        );
        assert_eq!(rgb(direct(&[u8v]).unwrap()), vec![1.0, 128.0 / 255.0, 0.0]);
    }

    /// [validatecolor.hex] cases `hex-6`, `hex-3`.
    #[test]
    fn observed_hex_codes() {
        assert_eq!(
            rgb(direct(&[text("#FF8000")]).unwrap()),
            vec![1.0, 128.0 / 255.0, 0.0]
        );
        assert_eq!(
            rgb(direct(&[text("#f80")]).unwrap()),
            vec![1.0, 136.0 / 255.0, 0.0]
        );
    }

    /// [validatecolor.multiple] cases `multiple-names`, `multiple-matrix`,
    /// `one-option`.
    #[test]
    fn observed_multiple_and_one() {
        let out = direct(&[cell(vec![text("r"), text("b")]), text("multiple")]).unwrap();
        assert_eq!(
            shaped(&out),
            (vec![2, 3], vec![1.0, 0.0, 0.0, 0.0, 0.0, 1.0])
        );
        let m = matrix(&[&[1.0, 0.0, 0.0], &[0.0, 1.0, 0.0]]);
        let out = direct(&[m, text("multiple")]).unwrap();
        assert_eq!(
            shaped(&out),
            (vec![2, 3], vec![1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
        );
        assert_eq!(
            rgb(direct(&[text("r"), text("one")]).unwrap()),
            vec![1.0, 0.0, 0.0]
        );
    }

    /// [validatecolor.error-matrix-without-multiple] case
    /// `err-matrix-without-multiple`.
    #[test]
    fn observed_matrix_without_multiple_errors() {
        let m = matrix(&[&[1.0, 0.0, 0.0], &[0.0, 1.0, 0.0]]);
        let err = direct(&[m]).unwrap_err();
        assert_eq!(
            identifier(&err),
            "RunMat:graphics:validatecolor:MultipleColors"
        );
    }

    /// [validatecolor.error-unknown-name] cases `err-unknown-name`, `err-none`.
    #[test]
    fn observed_unknown_name_errors() {
        for name in ["notacolour", "none"] {
            let err = direct(&[text(name)]).unwrap_err();
            assert_eq!(
                identifier(&err),
                "RunMat:graphics:validatecolor:InvalidColorString",
                "{name}"
            );
        }
    }

    /// [validatecolor.error-out-of-range] case `err-out-of-range`.
    #[test]
    fn observed_out_of_range_errors() {
        let err = direct(&[row(&[1.5, 0.0, 0.0])]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:graphics:validatecolor:OutOfRange");
    }

    /// [validatecolor.error-two-components] cases `err-two-components`,
    /// `err-scalar`.
    #[test]
    fn observed_triplet_shape_errors() {
        for value in [row(&[1.0, 0.0]), Value::Num(1.0)] {
            let err = direct(&[value]).unwrap_err();
            assert_eq!(
                identifier(&err),
                "RunMat:graphics:validatecolor:InvalidTripletShape"
            );
        }
    }

    /// [validatecolor.error-bad-option] case `err-bad-option`
    /// (validatecolor.q-error-ids: generic identifier reproduced).
    #[test]
    fn observed_bad_option_errors() {
        let err = direct(&[text("r"), text("several")]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:unrecognizedStringChoice");
    }

    // ---- documented choices for unobserved forms ----

    /// (choice) Negative/NaN components, unsupported classes, argument
    /// counts, option case, cell/string-array forms and empty 'multiple'.
    #[test]
    fn unresolved_choice_unobserved_forms() {
        let id = |args: &[Value]| identifier(&direct(args).unwrap_err());
        assert_eq!(
            id(&[row(&[-0.1, 0.0, 0.0])]),
            "RunMat:graphics:validatecolor:OutOfRange"
        );
        assert_eq!(
            id(&[row(&[f64::NAN, 0.0, 0.0])]),
            "RunMat:graphics:validatecolor:OutOfRange"
        );
        assert_eq!(id(&[]), "RunMat:minrhs");
        assert_eq!(id(&[text("r"), text("one"), text("x")]), "RunMat:maxrhs");
        assert_eq!(
            id(&[text("r"), Value::Num(1.0)]),
            "RunMat:unrecognizedStringChoice"
        );
        assert_eq!(
            id(&[Value::Bool(true)]),
            "RunMat:validatecolor:UnsupportedOption"
        );
        let complex = Value::ComplexTensor(
            ComplexTensor::new(vec![(1.0, 0.0); 3], vec![1, 3]).expect("complex"),
        );
        assert_eq!(id(&[complex]), "RunMat:validatecolor:UnsupportedOption");
        let i8v = Value::Tensor(
            Tensor::new_integer(IntegerStorage::I8(vec![1, 0, 0]), vec![1, 3]).expect("int8"),
        );
        assert_eq!(id(&[i8v]), "RunMat:validatecolor:UnsupportedOption");
        assert_eq!(
            id(&[cell(vec![text("r"), text("b")])]),
            "RunMat:graphics:validatecolor:MultipleColors"
        );
        assert_eq!(
            id(&[matrix(&[&[1.0, 0.0], &[0.0, 1.0]]), text("multiple")]),
            "RunMat:graphics:validatecolor:InvalidTripletShape"
        );
        // Accepted forms.
        assert_eq!(
            rgb(direct(&[text(" R "), text("MULTIPLE")]).unwrap()),
            vec![1.0, 0.0, 0.0]
        );
        assert_eq!(
            rgb(direct(&[cell(vec![text("g")])]).unwrap()),
            vec![0.0, 1.0, 0.0]
        );
        let u16v = Value::Tensor(
            Tensor::new_integer(IntegerStorage::U16(vec![65535, 0, 0]), vec![1, 3])
                .expect("uint16"),
        );
        assert_eq!(rgb(direct(&[u16v]).unwrap()), vec![1.0, 0.0, 0.0]);
        let sa = Value::StringArray(
            StringArray::new(vec!["r".into(), "#00ff00".into()], vec![1, 2]).expect("strings"),
        );
        assert_eq!(
            shaped(&direct(&[sa, text("multiple")]).unwrap()),
            (vec![2, 3], vec![1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
        );
        let empty = Value::Tensor(Tensor::new(Vec::new(), vec![0, 3]).expect("empty"));
        assert_eq!(
            shaped(&direct(&[empty, text("multiple")]).unwrap()),
            (vec![0, 3], Vec::new())
        );
    }

    /// Dispatcher path: callable by name.
    #[test]
    fn registered_through_dispatcher() {
        let out = block_on(crate::call_builtin_async("validatecolor", &[text("b")])).expect("call");
        assert_eq!(rgb(out), vec![0.0, 0.0, 1.0]);
    }
}
