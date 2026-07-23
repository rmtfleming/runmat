// Parallel clean-room implementation of `str2num` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/strings/core/str2num.rs
//! MATLAB-compatible `str2num` builtin for RunMat.
//!
//! Clean-room provenance: specs/007-numeric-string-conversion (spec `str2num`
//! 0.2.0, claims str2num.signature-primary, str2num.output-class,
//! str2num.parse-eval).
//!
//! Observed behaviour (normative): parses text to double values —
//! `str2num('42')` => 42 (1x1), `str2num('[1 2 3]')` => 1x3,
//! `str2num('[1 2; 3 4]')` => 2x2 — and evaluates arithmetic expressions
//! (`str2num('2+3')` => 5). Per the approved summary, unparsable input yields
//! an empty result (independently chosen here as a 0x0 double, tracked by
//! unresolved question str2num.q-eval).
//!
//! Evaluation strategy (independent implementation): a self-contained numeric
//! parser/evaluator covers numeric literals, bracketed row/matrix
//! concatenation, and scalar arithmetic. Text outside that grammar is
//! delegated to the registered `eval` builtin
//! (`builtins/introspection/dynamic_workspace.rs`); RunMat's `eval` executes
//! only inside a VM workspace frame, so outside one the delegation fails and
//! maps to the documented empty result.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    Tensor, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::map_control_flow_with_builtin;
use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::builtins::common::tensor;
use crate::builtins::strings::type_resolvers::unknown_type;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const BUILTIN_NAME: &str = "str2num";

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "str2num",
    op_kind: GpuOpKind::Custom("conversion"),
    supported_precisions: &[],
    broadcast: BroadcastSemantics::None,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "Parses text on the CPU; GPU-resident inputs are gathered before conversion.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "str2num",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: true,
    notes: "Conversion builtin; not eligible for fusion and materialises host-side doubles.",
};

const STR2NUM_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "X",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Numeric result, or a 0x0 empty double when evaluation fails.",
}];

const STR2NUM_INPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "chr",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description:
        "Character row vector or string scalar containing a numeric literal or expression.",
}];

const STR2NUM_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "X = str2num(chr)",
    inputs: &STR2NUM_INPUT,
    outputs: &STR2NUM_OUTPUT,
}];

const STR2NUM_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STR2NUM.INVALID_INPUT",
    identifier: Some("RunMat:str2num:InvalidInput"),
    when: "Input is not a character row vector or string scalar.",
    message: "str2num: input must be a character row vector or string scalar",
};

const STR2NUM_ERRORS: [BuiltinErrorDescriptor; 1] = [STR2NUM_ERROR_INVALID_INPUT];

pub const STR2NUM_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &STR2NUM_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &STR2NUM_ERRORS,
};

fn str2num_error_with_message(
    message: impl Into<String>,
    error: &'static BuiltinErrorDescriptor,
) -> RuntimeError {
    let mut builder = build_runtime_error(message).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn remap_str2num_flow(err: RuntimeError) -> RuntimeError {
    map_control_flow_with_builtin(err, BUILTIN_NAME)
}

async fn str2num_builtin(value: Value) -> BuiltinResult<Value> {
    let gathered = gather_if_needed_async(&value)
        .await
        .map_err(remap_str2num_flow)?;
    let text = extract_text(&gathered)?;

    if let Some(parsed) = parse_numeric_source(&text) {
        return Ok(parsed_to_value(parsed));
    }

    // Delegation path: hand the text to the registered `eval` builtin. This
    // succeeds only inside a VM workspace frame (the runtime-side `eval`
    // dispatch errors with RunMat:DynamicWorkspaceRequiresVm otherwise); any
    // failure or non-numeric result maps to the documented empty output.
    match crate::call_builtin_async_with_outputs("eval", &[Value::String(text)], 1).await {
        Ok(result) => Ok(coerce_eval_result(result)),
        Err(_) => Ok(empty_double()),
    }
}

/// Accept character row vectors (including 0x0 '') and string scalars.
/// Other input classes are unobserved in the approved export; the documented
/// independent choice rejects them with a stable identifier.
fn extract_text(value: &Value) -> BuiltinResult<String> {
    match value {
        Value::String(text) => Ok(text.clone()),
        Value::StringArray(array) if array.data.len() == 1 => Ok(array.data[0].clone()),
        Value::CharArray(chars) if chars.rows <= 1 => Ok(chars.data.iter().collect()),
        other => Err(str2num_error_with_message(
            format!("{} (got {:?})", STR2NUM_ERROR_INVALID_INPUT.message, other),
            &STR2NUM_ERROR_INVALID_INPUT,
        )),
    }
}

fn empty_double() -> Value {
    Value::Tensor(Tensor::new(Vec::new(), vec![0, 0]).expect("0x0 tensor is always valid"))
}

/// Map a delegated evaluation result onto the observed double output class.
/// Non-numeric results become empty (documented choice under
/// str2num.q-eval).
fn coerce_eval_result(value: Value) -> Value {
    match value {
        Value::Num(_) | Value::Tensor(_) | Value::Complex(..) | Value::ComplexTensor(_) => value,
        Value::Int(i) => Value::Num(i.to_f64()),
        Value::Bool(b) => Value::Num(if b { 1.0 } else { 0.0 }),
        Value::LogicalArray(la) => match tensor::logical_to_tensor(&la) {
            Ok(t) => Value::Tensor(t),
            Err(_) => empty_double(),
        },
        Value::OutputList(mut values) => {
            if values.is_empty() {
                empty_double()
            } else {
                coerce_eval_result(values.remove(0))
            }
        }
        _ => empty_double(),
    }
}

enum ParsedNumeric {
    Scalar(f64),
    Matrix {
        data: Vec<f64>,
        rows: usize,
        cols: usize,
    },
}

fn parsed_to_value(parsed: ParsedNumeric) -> Value {
    match parsed {
        ParsedNumeric::Scalar(value) => Value::Num(value),
        ParsedNumeric::Matrix { data, rows, cols } => {
            if rows == 1 && cols == 1 {
                Value::Num(data[0])
            } else {
                match Tensor::new(data, vec![rows, cols]) {
                    Ok(tensor) => Value::Tensor(tensor),
                    Err(_) => empty_double(),
                }
            }
        }
    }
}

/// Self-contained numeric grammar: a bracketed concatenation of rows of
/// scalar expressions, or a single scalar expression. Anything outside the
/// grammar returns `None` so the caller can fall back to `eval` delegation.
fn parse_numeric_source(text: &str) -> Option<ParsedNumeric> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(after_open) = trimmed.strip_prefix('[') {
        let body = after_open.strip_suffix(']')?;
        // Nested concatenation is outside this grammar.
        if body.contains('[') || body.contains(']') {
            return None;
        }
        parse_matrix_body(body)
    } else {
        parse_full_expression(trimmed).map(ParsedNumeric::Scalar)
    }
}

fn parse_matrix_body(body: &str) -> Option<ParsedNumeric> {
    let mut rows_data: Vec<Vec<f64>> = Vec::new();
    for row_text in split_top_level(body, &[';', '\n']) {
        let row_text = row_text.trim();
        if row_text.is_empty() {
            continue;
        }
        let mut row = Vec::new();
        for element in split_row_elements(row_text)? {
            row.push(parse_full_expression(element.trim())?);
        }
        if !row.is_empty() {
            rows_data.push(row);
        }
    }
    if rows_data.is_empty() {
        return Some(ParsedNumeric::Matrix {
            data: Vec::new(),
            rows: 0,
            cols: 0,
        });
    }
    let cols = rows_data[0].len();
    if rows_data.iter().any(|row| row.len() != cols) {
        return None;
    }
    let rows = rows_data.len();
    let mut data = vec![0.0; rows * cols];
    for (r, row) in rows_data.iter().enumerate() {
        for (c, value) in row.iter().enumerate() {
            data[r + c * rows] = *value;
        }
    }
    Some(ParsedNumeric::Matrix { data, rows, cols })
}

/// Split on any separator at parenthesis depth zero.
fn split_top_level<'a>(text: &'a str, separators: &[char]) -> Vec<&'a str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (idx, ch) in text.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ if depth == 0 && separators.contains(&ch) => {
                parts.push(&text[start..idx]);
                start = idx + ch.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&text[start..]);
    parts
}

fn is_binary_operator(ch: char) -> bool {
    matches!(ch, '+' | '-' | '*' | '/' | '^')
}

/// Split one bracketed row into element expressions. Elements separate on
/// commas, or on whitespace unless that whitespace continues a binary
/// operation (`1 + 2` stays one element); a sign directly attached to the
/// following term (`1 -2`) starts a new element. Independent choice on
/// layouts beyond the space-separated observed cases.
fn split_row_elements(row: &str) -> Option<Vec<String>> {
    let chars: Vec<char> = row.chars().collect();
    let mut elements = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    let mut i = 0usize;
    while i < chars.len() {
        let ch = chars[i];
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
                i += 1;
            }
            ')' => {
                if depth == 0 {
                    return None;
                }
                depth -= 1;
                current.push(ch);
                i += 1;
            }
            ',' if depth == 0 => {
                if current.trim().is_empty() {
                    return None;
                }
                elements.push(std::mem::take(&mut current));
                i += 1;
            }
            c if c.is_whitespace() && depth == 0 => {
                let mut j = i;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if j >= chars.len() {
                    break;
                }
                let prev = current.trim_end().chars().last();
                let next = chars[j];
                let boundary = match prev {
                    None => false,
                    Some(p) if is_binary_operator(p) || p == '(' => false,
                    _ => match next {
                        '*' | '/' | '^' | ')' => false,
                        '+' | '-' => matches!(
                            chars.get(j + 1),
                            Some(after) if !after.is_whitespace() && !is_binary_operator(*after)
                        ),
                        _ => true,
                    },
                };
                if boundary {
                    if current.trim().is_empty() {
                        return None;
                    }
                    elements.push(std::mem::take(&mut current));
                } else if !current.is_empty() {
                    current.push(' ');
                }
                i = j;
            }
            _ => {
                current.push(ch);
                i += 1;
            }
        }
    }
    if depth != 0 {
        return None;
    }
    if !current.trim().is_empty() {
        elements.push(current);
    }
    Some(elements)
}

/// Evaluate one scalar arithmetic expression: `+ - * /` with standard
/// precedence, right-associative `^`, unary signs, parentheses, numeric
/// literals (decimal and scientific), and `Inf`/`NaN` words.
fn parse_full_expression(text: &str) -> Option<f64> {
    let chars: Vec<char> = text.chars().collect();
    let mut parser = ExprParser {
        chars: &chars,
        pos: 0,
    };
    let value = parser.parse_expr()?;
    parser.skip_whitespace();
    if parser.pos == chars.len() {
        Some(value)
    } else {
        None
    }
}

struct ExprParser<'a> {
    chars: &'a [char],
    pos: usize,
}

impl ExprParser<'_> {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(|c| c.is_whitespace()) {
            self.pos += 1;
        }
    }

    fn parse_expr(&mut self) -> Option<f64> {
        let mut value = self.parse_term()?;
        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('+') => {
                    self.pos += 1;
                    value += self.parse_term()?;
                }
                Some('-') => {
                    self.pos += 1;
                    value -= self.parse_term()?;
                }
                _ => return Some(value),
            }
        }
    }

    fn parse_term(&mut self) -> Option<f64> {
        let mut value = self.parse_unary()?;
        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('*') => {
                    self.pos += 1;
                    value *= self.parse_unary()?;
                }
                Some('/') => {
                    self.pos += 1;
                    value /= self.parse_unary()?;
                }
                _ => return Some(value),
            }
        }
    }

    fn parse_unary(&mut self) -> Option<f64> {
        self.skip_whitespace();
        match self.peek() {
            Some('+') => {
                self.pos += 1;
                self.parse_unary()
            }
            Some('-') => {
                self.pos += 1;
                self.parse_unary().map(|value| -value)
            }
            _ => self.parse_power(),
        }
    }

    fn parse_power(&mut self) -> Option<f64> {
        let base = self.parse_atom()?;
        self.skip_whitespace();
        if self.peek() == Some('^') {
            self.pos += 1;
            let exponent = self.parse_unary()?;
            return Some(base.powf(exponent));
        }
        Some(base)
    }

    fn parse_atom(&mut self) -> Option<f64> {
        self.skip_whitespace();
        match self.peek()? {
            '(' => {
                self.pos += 1;
                let value = self.parse_expr()?;
                self.skip_whitespace();
                if self.peek() == Some(')') {
                    self.pos += 1;
                    Some(value)
                } else {
                    None
                }
            }
            c if c.is_ascii_digit() || c == '.' => self.parse_number(),
            c if c.is_ascii_alphabetic() => self.parse_word(),
            _ => None,
        }
    }

    fn parse_number(&mut self) -> Option<f64> {
        let start = self.pos;
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.pos += 1;
        }
        if self.peek() == Some('.') {
            self.pos += 1;
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        if matches!(self.peek(), Some('e') | Some('E')) {
            let mark = self.pos;
            self.pos += 1;
            if matches!(self.peek(), Some('+') | Some('-')) {
                self.pos += 1;
            }
            if self.peek().is_some_and(|c| c.is_ascii_digit()) {
                while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                    self.pos += 1;
                }
            } else {
                self.pos = mark;
            }
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        if text.is_empty() || text == "." {
            return None;
        }
        text.parse::<f64>().ok()
    }

    fn parse_word(&mut self) -> Option<f64> {
        let start = self.pos;
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            self.pos += 1;
        }
        let word: String = self.chars[start..self.pos].iter().collect();
        match word.to_ascii_lowercase().as_str() {
            "inf" => Some(f64::INFINITY),
            "nan" => Some(f64::NAN),
            _ => None,
        }
    }
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use runmat_builtins::{CharArray, ResolveContext, Type};

    fn run_str2num(value: Value) -> Value {
        futures::executor::block_on(super::str2num_builtin(value)).expect("str2num")
    }

    fn char_row(text: &str) -> Value {
        Value::CharArray(CharArray::new_row(text))
    }

    fn expect_empty(value: Value) {
        match value {
            Value::Tensor(t) => {
                assert_eq!(t.shape, vec![0, 0]);
                assert!(t.data.is_empty());
            }
            other => panic!("expected 0x0 empty double, got {other:?}"),
        }
    }

    // Normative [FR-007-04, FR-007-05; str2num.signature-primary,
    // str2num.output-class, str2num.parse-eval]
    // Observed case `scalar`: str2num('42') => 42 (double, 1x1).
    #[test]
    fn observed_scalar_literal() {
        assert_eq!(run_str2num(char_row("42")), Value::Num(42.0));
    }

    // Normative [FR-007-05; str2num.parse-eval, str2num.output-class]
    // Observed case `row-vector`: str2num('[1 2 3]') => [1 2 3] (double, 1x3).
    #[test]
    fn observed_bracketed_row_vector() {
        match run_str2num(char_row("[1 2 3]")) {
            Value::Tensor(t) => {
                assert_eq!(t.shape, vec![1, 3]);
                assert_eq!(t.data, vec![1.0, 2.0, 3.0]);
            }
            other => panic!("expected 1x3 double, got {other:?}"),
        }
    }

    // Normative [FR-007-05; str2num.parse-eval, str2num.output-class]
    // Observed case `matrix`: str2num('[1 2; 3 4]') => [1 2;3 4]
    // (double, 2x2).
    #[test]
    fn observed_bracketed_matrix() {
        match run_str2num(char_row("[1 2; 3 4]")) {
            Value::Tensor(t) => {
                assert_eq!(t.shape, vec![2, 2]);
                // Column-major storage of [1 2; 3 4].
                assert_eq!(t.data, vec![1.0, 3.0, 2.0, 4.0]);
            }
            other => panic!("expected 2x2 double, got {other:?}"),
        }
    }

    // Normative [FR-007-05; str2num.parse-eval]
    // Observed case `expression`: str2num('2+3') => 5 (double, 1x1).
    #[test]
    fn observed_arithmetic_expression() {
        assert_eq!(run_str2num(char_row("2+3")), Value::Num(5.0));
    }

    // summary_derived [FR-007-06]: the approved summary says str2num
    // "returns empty when the input cannot be parsed"; no observation case
    // backs the concrete shape, chosen here as 0x0 double.
    #[test]
    fn summary_derived_unparsable_text_returns_empty() {
        expect_empty(run_str2num(char_row("definitely not numeric")));
        expect_empty(run_str2num(char_row("")));
    }

    // summary_derived [FR-007-05]: str2num.parse-eval says arithmetic
    // expressions evaluate; standard precedence/grouping is derived, not
    // observed.
    #[test]
    fn summary_derived_arithmetic_precedence_and_grouping() {
        assert_eq!(run_str2num(char_row("2+3*4")), Value::Num(14.0));
        assert_eq!(run_str2num(char_row("(1+2)*3")), Value::Num(9.0));
        assert_eq!(run_str2num(char_row("2^3")), Value::Num(8.0));
        assert_eq!(run_str2num(char_row("-4/2")), Value::Num(-2.0));
    }

    // unresolved_choice: comma separators are unobserved; documented choice
    // treats them like the observed space separators.
    #[test]
    fn unresolved_choice_comma_separated_row() {
        match run_str2num(char_row("[1,2,3]")) {
            Value::Tensor(t) => {
                assert_eq!(t.shape, vec![1, 3]);
                assert_eq!(t.data, vec![1.0, 2.0, 3.0]);
            }
            other => panic!("expected 1x3 double, got {other:?}"),
        }
    }

    // unresolved_choice: sign/whitespace tokenisation inside brackets is
    // unobserved; documented choice: an attached sign starts a new element,
    // a spaced operator continues the expression.
    #[test]
    fn unresolved_choice_signed_whitespace_tokenisation() {
        match run_str2num(char_row("[1 -2]")) {
            Value::Tensor(t) => {
                assert_eq!(t.shape, vec![1, 2]);
                assert_eq!(t.data, vec![1.0, -2.0]);
            }
            other => panic!("expected 1x2 double, got {other:?}"),
        }
        assert_eq!(run_str2num(char_row("[1 - 2]")), Value::Num(-1.0));
    }

    // unresolved_choice: '[]' and a bracketed scalar are unobserved;
    // documented choices: 0x0 empty double and scalar double respectively.
    #[test]
    fn unresolved_choice_empty_brackets_and_bracketed_scalar() {
        expect_empty(run_str2num(char_row("[]")));
        assert_eq!(run_str2num(char_row("[5]")), Value::Num(5.0));
    }

    // unresolved_choice: inconsistent row lengths are unobserved; documented
    // choice maps the failed concatenation to the empty result.
    #[test]
    fn unresolved_choice_inconsistent_row_lengths_return_empty() {
        expect_empty(run_str2num(char_row("[1 2; 3]")));
    }

    // unresolved_choice: identifier expressions need workspace evaluation;
    // the `eval` delegation requires a VM workspace frame, so outside one
    // the result is the documented empty double.
    #[test]
    fn unresolved_choice_identifier_expression_returns_empty_without_vm() {
        expect_empty(run_str2num(char_row("x+1")));
    }

    // unresolved_choice: non-text input classes are unobserved; documented
    // choice rejects them with a stable identifier.
    #[test]
    fn unresolved_choice_non_text_input_errors() {
        let err = futures::executor::block_on(super::str2num_builtin(Value::Num(5.0)))
            .expect_err("str2num should reject numeric input");
        assert_eq!(err.identifier(), Some("RunMat:str2num:InvalidInput"));
    }

    // unresolved_choice: string-scalar input is unobserved (observed calls
    // used char rows); documented choice accepts it as text.
    #[test]
    fn unresolved_choice_string_scalar_input_accepted() {
        assert_eq!(run_str2num(Value::String("7".to_string())), Value::Num(7.0));
    }

    // Registration is discoverable through the builtin registry (SC-007-2).
    #[test]
    fn str2num_is_registered() {
        assert!(runmat_builtins::builtin_function_by_name("str2num").is_some());
    }

    #[test]
    fn str2num_type_resolver_is_unknown() {
        assert_eq!(
            unknown_type(&[Type::String], &ResolveContext::new(Vec::new())),
            Type::Unknown
        );
    }
}
