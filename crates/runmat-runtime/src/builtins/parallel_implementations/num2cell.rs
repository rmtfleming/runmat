// Parallel clean-room implementation of `num2cell` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/cells/core/num2cell.rs
//! MATLAB-compatible `num2cell` builtin for RunMat.
//!
//! Clean-room provenance: specs/005-struct-cell-access (spec `num2cell`
//! 0.2.0, claims num2cell.signature-primary, num2cell.output-class,
//! num2cell.cell-grouping). By default every element of the input becomes a
//! 1x1 cell and the input shape is preserved; an optional dimension argument
//! groups elements along the given dimension(s).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    CharArray, ComplexTensor, LogicalArray, StringArray, Tensor, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::builtins::common::tensor;
use crate::{
    build_runtime_error, gather_if_needed_async, make_cell_with_shape, BuiltinResult, RuntimeError,
};

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "num2cell",
    op_kind: GpuOpKind::Custom("container"),
    supported_precisions: &[],
    broadcast: BroadcastSemantics::None,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "num2cell gathers gpuArray inputs to the host; cell construction is host-only.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "num2cell",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Conversion into cells terminates fusion; cell contents are produced on the host.",
};

const BUILTIN_NAME: &str = "num2cell";

const NUM2CELL_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "C",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Cell array holding the elements (or grouped slices) of the input.",
}];

const NUM2CELL_SIG_DEFAULT_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "A",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Array to convert.",
}];

const NUM2CELL_SIG_DIM_INPUTS: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "A",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Array to convert.",
    },
    BuiltinParamDescriptor {
        name: "dim",
        ty: BuiltinParamType::SizeArg,
        arity: BuiltinParamArity::Optional,
        default: None,
        description: "Dimension(s) to group along.",
    },
];

const NUM2CELL_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "C = num2cell(A)",
        inputs: &NUM2CELL_SIG_DEFAULT_INPUTS,
        outputs: &NUM2CELL_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "C = num2cell(A, dim)",
        inputs: &NUM2CELL_SIG_DIM_INPUTS,
        outputs: &NUM2CELL_OUTPUT,
    },
];

const NUM2CELL_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.NUM2CELL.INVALID_INPUT",
    identifier: Some("RunMat:num2cell:InvalidInput"),
    when: "Input array type or argument count is invalid.",
    message: "num2cell: invalid input arguments",
};

const NUM2CELL_ERROR_INVALID_DIM: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.NUM2CELL.INVALID_DIM",
    identifier: Some("RunMat:num2cell:InvalidDim"),
    when: "The dimension argument is not a vector of positive integers.",
    message: "num2cell: dimension argument must be positive integers",
};

const NUM2CELL_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.NUM2CELL.INTERNAL",
    identifier: None,
    when: "Internal slicing, indexing, or allocation failed.",
    message: "num2cell: internal error",
};

const NUM2CELL_ERRORS: [BuiltinErrorDescriptor; 3] = [
    NUM2CELL_ERROR_INVALID_INPUT,
    NUM2CELL_ERROR_INVALID_DIM,
    NUM2CELL_ERROR_INTERNAL,
];

pub const NUM2CELL_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &NUM2CELL_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &NUM2CELL_ERRORS,
};

fn num2cell_error_with_message(
    message: impl Into<String>,
    error: &'static BuiltinErrorDescriptor,
) -> RuntimeError {
    let mut builder = build_runtime_error(message).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

async fn num2cell_builtin(value: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
    if rest.len() > 1 {
        return Err(num2cell_error_with_message(
            "num2cell: expected at most one dimension argument",
            &NUM2CELL_ERROR_INVALID_INPUT,
        ));
    }

    let host_value = gather_if_needed_async(&value).await?;
    let dims = match rest.first() {
        Some(arg) => {
            let gathered = gather_if_needed_async(arg).await?;
            parse_group_dims(&gathered)?
        }
        None => Vec::new(),
    };

    let input = Num2CellInput::try_new(host_value)?;
    convert_to_cells(&input, &dims)
}

#[derive(Debug)]
enum Num2CellKind {
    Tensor(Tensor),
    Complex(ComplexTensor),
    Logical(LogicalArray),
    Strings(StringArray),
    Char(CharArray),
}

struct Num2CellInput {
    kind: Num2CellKind,
    base_shape: Vec<usize>,
}

impl Num2CellInput {
    fn try_new(value: Value) -> BuiltinResult<Self> {
        match value {
            Value::Tensor(t) => {
                let base_shape = normalize_shape(&t.shape);
                Ok(Self {
                    kind: Num2CellKind::Tensor(t),
                    base_shape,
                })
            }
            Value::ComplexTensor(t) => {
                let base_shape = normalize_shape(&t.shape);
                Ok(Self {
                    kind: Num2CellKind::Complex(t),
                    base_shape,
                })
            }
            Value::LogicalArray(l) => {
                let base_shape = normalize_shape(&l.shape);
                Ok(Self {
                    kind: Num2CellKind::Logical(l),
                    base_shape,
                })
            }
            Value::StringArray(sa) => {
                let base_shape = if sa.shape.is_empty() {
                    vec![1, sa.rows()]
                } else {
                    normalize_shape(&sa.shape)
                };
                Ok(Self {
                    kind: Num2CellKind::Strings(sa),
                    base_shape,
                })
            }
            Value::String(s) => {
                let array = StringArray::new(vec![s], vec![1, 1]).map_err(|e| {
                    num2cell_error_with_message(format!("num2cell: {e}"), &NUM2CELL_ERROR_INTERNAL)
                })?;
                Ok(Self {
                    kind: Num2CellKind::Strings(array),
                    base_shape: vec![1, 1],
                })
            }
            Value::CharArray(ca) => {
                let base_shape = vec![ca.rows, ca.cols];
                Ok(Self {
                    kind: Num2CellKind::Char(ca),
                    base_shape,
                })
            }
            Value::Complex(re, im) => {
                let tensor = ComplexTensor::new(vec![(re, im)], vec![1, 1]).map_err(|e| {
                    num2cell_error_with_message(format!("num2cell: {e}"), &NUM2CELL_ERROR_INTERNAL)
                })?;
                Self::try_new(Value::ComplexTensor(tensor))
            }
            Value::Num(n) => {
                let tensor = tensor::value_into_tensor_for(BUILTIN_NAME, Value::Num(n))
                    .map_err(|e| num2cell_error_with_message(e, &NUM2CELL_ERROR_INTERNAL))?;
                Self::try_new(Value::Tensor(tensor))
            }
            Value::Int(i) => {
                let tensor = tensor::value_into_tensor_for(BUILTIN_NAME, Value::Int(i.clone()))
                    .map_err(|e| num2cell_error_with_message(e, &NUM2CELL_ERROR_INTERNAL))?;
                Self::try_new(Value::Tensor(tensor))
            }
            Value::Bool(b) => {
                let logical = LogicalArray::new(vec![u8::from(b)], vec![1, 1]).map_err(|e| {
                    num2cell_error_with_message(format!("num2cell: {e}"), &NUM2CELL_ERROR_INTERNAL)
                })?;
                Self::try_new(Value::LogicalArray(logical))
            }
            other => Err(num2cell_error_with_message(
                format!(
                    "num2cell: unsupported input type {other:?}; expected numeric, logical, string, or char arrays"
                ),
                &NUM2CELL_ERROR_INVALID_INPUT,
            )),
        }
    }

    fn extract(&self, start: &[usize], sizes: &[usize]) -> BuiltinResult<Value> {
        match &self.kind {
            Num2CellKind::Tensor(t) => {
                let data = copy_block(&t.data, &self.base_shape, start, sizes)?;
                let shape = trim_shape(sizes.to_vec());
                let tensor = Tensor::new(data, shape).map_err(|e| {
                    num2cell_error_with_message(format!("num2cell: {e}"), &NUM2CELL_ERROR_INTERNAL)
                })?;
                Ok(tensor::tensor_into_value(tensor))
            }
            Num2CellKind::Complex(t) => {
                let data = copy_block(&t.data, &self.base_shape, start, sizes)?;
                if data.len() == 1 {
                    let (re, im) = data[0];
                    Ok(Value::Complex(re, im))
                } else {
                    let shape = trim_shape(sizes.to_vec());
                    let tensor = ComplexTensor::new(data, shape).map_err(|e| {
                        num2cell_error_with_message(
                            format!("num2cell: {e}"),
                            &NUM2CELL_ERROR_INTERNAL,
                        )
                    })?;
                    Ok(Value::ComplexTensor(tensor))
                }
            }
            Num2CellKind::Logical(arr) => {
                let data = copy_block(&arr.data, &self.base_shape, start, sizes)?;
                if data.len() == 1 {
                    Ok(Value::Bool(data[0] != 0))
                } else {
                    let shape = trim_shape(sizes.to_vec());
                    let logical = LogicalArray::new(data, shape).map_err(|e| {
                        num2cell_error_with_message(
                            format!("num2cell: {e}"),
                            &NUM2CELL_ERROR_INTERNAL,
                        )
                    })?;
                    Ok(Value::LogicalArray(logical))
                }
            }
            Num2CellKind::Strings(arr) => {
                let data = copy_block(&arr.data, &self.base_shape, start, sizes)?;
                if data.len() == 1 {
                    Ok(Value::String(data.into_iter().next().unwrap()))
                } else {
                    let shape = trim_shape(sizes.to_vec());
                    let strings = StringArray::new(data, shape).map_err(|e| {
                        num2cell_error_with_message(
                            format!("num2cell: {e}"),
                            &NUM2CELL_ERROR_INTERNAL,
                        )
                    })?;
                    Ok(Value::StringArray(strings))
                }
            }
            Num2CellKind::Char(ca) => slice_char_array(ca, start, sizes),
        }
    }
}

fn convert_to_cells(input: &Num2CellInput, dims: &[usize]) -> BuiltinResult<Value> {
    let rank = input
        .base_shape
        .len()
        .max(dims.iter().copied().max().unwrap_or(0))
        .max(2);
    let mut ext_shape = input.base_shape.clone();
    while ext_shape.len() < rank {
        ext_shape.push(1);
    }

    let mut grouped = vec![false; rank];
    for &dim in dims {
        grouped[dim - 1] = true;
    }

    let mut outer_shape = Vec::with_capacity(rank);
    let mut content_sizes = Vec::with_capacity(rank);
    for d in 0..rank {
        if grouped[d] {
            outer_shape.push(1);
            content_sizes.push(ext_shape[d]);
        } else {
            outer_shape.push(ext_shape[d]);
            content_sizes.push(1);
        }
    }

    let total_cells = outer_shape
        .iter()
        .try_fold(1usize, |acc, &v| acc.checked_mul(v))
        .ok_or_else(|| {
            num2cell_error_with_message(
                "num2cell: resulting cell array is too large to represent on this platform",
                &NUM2CELL_ERROR_INTERNAL,
            )
        })?;

    let cell_shape = trim_shape(outer_shape.clone());
    if total_cells == 0 {
        return make_cell_with_shape(Vec::new(), cell_shape).map_err(|e| {
            num2cell_error_with_message(format!("num2cell: {e}"), &NUM2CELL_ERROR_INTERNAL)
        });
    }

    // Cell arrays store their data row-major (last dimension fastest), while
    // tensor blocks are extracted column-major; iterate accordingly.
    let mut indices = vec![0usize; rank];
    let mut cells = Vec::with_capacity(total_cells);
    loop {
        let mut start = Vec::with_capacity(rank);
        for d in 0..rank {
            start.push(if grouped[d] { 0 } else { indices[d] });
        }
        cells.push(input.extract(&start, &content_sizes)?);

        let mut carry = true;
        for dim in (0..rank).rev() {
            indices[dim] += 1;
            if indices[dim] < outer_shape[dim] {
                carry = false;
                break;
            }
            indices[dim] = 0;
        }
        if carry {
            break;
        }
    }

    make_cell_with_shape(cells, cell_shape).map_err(|e| {
        num2cell_error_with_message(format!("num2cell: {e}"), &NUM2CELL_ERROR_INTERNAL)
    })
}

fn parse_group_dims(value: &Value) -> BuiltinResult<Vec<usize>> {
    let numbers = extract_numeric_vector(value).ok_or_else(|| {
        num2cell_error_with_message(
            "num2cell: dimension argument must be a numeric vector",
            &NUM2CELL_ERROR_INVALID_DIM,
        )
    })?;

    let mut dims = Vec::with_capacity(numbers.len());
    for n in numbers {
        if !n.is_finite() {
            return Err(num2cell_error_with_message(
                "num2cell: dimension entries must be finite",
                &NUM2CELL_ERROR_INVALID_DIM,
            ));
        }
        let rounded = n.round();
        if (rounded - n).abs() > f64::EPSILON || rounded < 1.0 {
            return Err(num2cell_error_with_message(
                NUM2CELL_ERROR_INVALID_DIM.message,
                &NUM2CELL_ERROR_INVALID_DIM,
            ));
        }
        dims.push(rounded as usize);
    }
    dims.sort_unstable();
    dims.dedup();
    Ok(dims)
}

fn extract_numeric_vector(value: &Value) -> Option<Vec<f64>> {
    match value {
        Value::Num(n) => Some(vec![*n]),
        Value::Int(i) => Some(vec![i.to_f64()]),
        Value::Tensor(t) => {
            if is_vector_shape(&t.shape) {
                Some(t.data.clone())
            } else {
                None
            }
        }
        _ => None,
    }
}

fn is_vector_shape(shape: &[usize]) -> bool {
    shape.iter().filter(|&&dim| dim > 1).count() <= 1
}

fn copy_block<T: Clone>(
    data: &[T],
    shape: &[usize],
    start: &[usize],
    sizes: &[usize],
) -> BuiltinResult<Vec<T>> {
    let rank = sizes.len();
    let extended_shape = extend_shape(shape, rank);
    let strides = column_major_strides(&extended_shape);

    for dim in 0..rank {
        if start[dim] + sizes[dim] > extended_shape[dim] {
            return Err(num2cell_error_with_message(
                format!("num2cell: slice exceeds dimension {} bounds", dim + 1),
                &NUM2CELL_ERROR_INTERNAL,
            ));
        }
    }

    let total = sizes.iter().product::<usize>();
    if total == 0 {
        return Ok(Vec::new());
    }

    let mut result = Vec::with_capacity(total);
    let mut indices = vec![0usize; rank];
    loop {
        let mut linear = 0usize;
        for dim in 0..rank {
            linear += (start[dim] + indices[dim]) * strides[dim];
        }
        result.push(
            data.get(linear)
                .ok_or_else(|| {
                    num2cell_error_with_message(
                        "num2cell: internal indexing error",
                        &NUM2CELL_ERROR_INTERNAL,
                    )
                })?
                .clone(),
        );

        let mut carry = true;
        for dim in 0..rank {
            indices[dim] += 1;
            if indices[dim] < sizes[dim] {
                carry = false;
                break;
            }
            indices[dim] = 0;
        }
        if carry {
            break;
        }
    }
    Ok(result)
}

fn slice_char_array(array: &CharArray, start: &[usize], sizes: &[usize]) -> BuiltinResult<Value> {
    for (dim, &count) in sizes.iter().enumerate().skip(2) {
        let offset = start.get(dim).copied().unwrap_or(0);
        if count != 1 || offset != 0 {
            return Err(num2cell_error_with_message(
                "num2cell: character arrays cannot be grouped along higher dimensions",
                &NUM2CELL_ERROR_INVALID_DIM,
            ));
        }
    }
    let row_start = start.first().copied().unwrap_or(0);
    let row_count = sizes.first().copied().unwrap_or(1);
    let col_start = start.get(1).copied().unwrap_or(0);
    let col_count = sizes.get(1).copied().unwrap_or(1);

    if row_start + row_count > array.rows || col_start + col_count > array.cols {
        return Err(num2cell_error_with_message(
            "num2cell: slice exceeds character array bounds",
            &NUM2CELL_ERROR_INTERNAL,
        ));
    }

    let mut data = Vec::with_capacity(row_count * col_count);
    for r in 0..row_count {
        for c in 0..col_count {
            // CharArray data is stored row-major.
            data.push(array.data[(row_start + r) * array.cols + (col_start + c)]);
        }
    }
    let slice = CharArray::new(data, row_count, col_count).map_err(|e| {
        num2cell_error_with_message(format!("num2cell: {e}"), &NUM2CELL_ERROR_INTERNAL)
    })?;
    Ok(Value::CharArray(slice))
}

fn extend_shape(shape: &[usize], rank: usize) -> Vec<usize> {
    let mut extended = if shape.is_empty() {
        vec![1, 1]
    } else {
        shape.to_vec()
    };
    while extended.len() < rank {
        extended.push(1);
    }
    extended
}

fn column_major_strides(shape: &[usize]) -> Vec<usize> {
    let mut strides = Vec::with_capacity(shape.len());
    let mut acc = 1usize;
    for &extent in shape {
        strides.push(acc);
        acc = acc.saturating_mul(extent.max(1));
    }
    strides
}

fn normalize_shape(shape: &[usize]) -> Vec<usize> {
    match shape.len() {
        0 => vec![1, 1],
        1 => vec![1, shape[0]],
        _ => shape.to_vec(),
    }
}

fn trim_shape(mut shape: Vec<usize>) -> Vec<usize> {
    while shape.len() > 2 && shape.last() == Some(&1) {
        shape.pop();
    }
    if shape.is_empty() {
        shape = vec![1, 1];
    } else if shape.len() == 1 {
        shape.push(1);
    }
    shape
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use crate::builtins::common::test_support;
    use futures::executor::block_on;
    use runmat_builtins::CellArray;

    fn run_num2cell(value: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
        block_on(super::num2cell_builtin(value, rest))
    }

    fn expect_cell(value: Value) -> CellArray {
        match value {
            Value::Cell(cell) => cell,
            other => panic!("expected cell array, got {other:?}"),
        }
    }

    // Normative [FR-005-04; num2cell.signature-primary, num2cell.output-class,
    // num2cell.cell-grouping] — observed case `vector`:
    // num2cell([10 20 30]) => 1x3 cell, one cell per element.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_row_vector_yields_1x3_cell() {
        let tensor = Tensor::new(vec![10.0, 20.0, 30.0], vec![1, 3]).unwrap();
        let cell = expect_cell(run_num2cell(Value::Tensor(tensor), vec![]).expect("num2cell"));
        assert_eq!(cell.shape, vec![1, 3]);
        assert_eq!(
            cell.data,
            vec![Value::Num(10.0), Value::Num(20.0), Value::Num(30.0)]
        );
    }

    // Normative [FR-005-04; num2cell.output-class, num2cell.cell-grouping] —
    // observed case `matrix`: num2cell([1 2; 3 4]) => 2x2 cell preserving the
    // input shape, one cell per element.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_matrix_yields_2x2_cell() {
        // Column-major data for [1 2; 3 4].
        let tensor = Tensor::new(vec![1.0, 3.0, 2.0, 4.0], vec![2, 2]).unwrap();
        let cell = expect_cell(run_num2cell(Value::Tensor(tensor), vec![]).expect("num2cell"));
        assert_eq!(cell.shape, vec![2, 2]);
        assert_eq!(cell.get(0, 0).unwrap(), Value::Num(1.0));
        assert_eq!(cell.get(0, 1).unwrap(), Value::Num(2.0));
        assert_eq!(cell.get(1, 0).unwrap(), Value::Num(3.0));
        assert_eq!(cell.get(1, 1).unwrap(), Value::Num(4.0));
    }

    // Normative [FR-005-05; num2cell.output-class, num2cell.cell-grouping] —
    // observed case `dim1`: num2cell([1 2; 3 4], 1) => 1x2 cell.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_dim1_yields_1x2_cell() {
        let tensor = Tensor::new(vec![1.0, 3.0, 2.0, 4.0], vec![2, 2]).unwrap();
        let cell = expect_cell(
            run_num2cell(Value::Tensor(tensor), vec![Value::Num(1.0)]).expect("num2cell"),
        );
        assert_eq!(cell.shape, vec![1, 2]);
    }

    // summary_derived [FR-005-05; num2cell.cell-grouping]: the approved claim
    // states the dimension argument groups elements along that dimension, so
    // each cell of num2cell([1 2; 3 4], 1) holds a 2x1 column; the cell
    // contents themselves are not recorded in the observation.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn summary_derived_dim1_cells_are_columns() {
        let tensor = Tensor::new(vec![1.0, 3.0, 2.0, 4.0], vec![2, 2]).unwrap();
        let cell = expect_cell(
            run_num2cell(Value::Tensor(tensor), vec![Value::Num(1.0)]).expect("num2cell"),
        );
        let expected_columns = [vec![1.0, 3.0], vec![2.0, 4.0]];
        for (value, expected) in cell.data.iter().zip(expected_columns.iter()) {
            let gathered = test_support::gather(value.clone()).expect("gather");
            assert_eq!(gathered.shape, vec![2, 1]);
            assert_eq!(&gathered.data, expected);
        }
    }

    // Normative [FR-005-04; num2cell.output-class, num2cell.cell-grouping] —
    // observed case `scalar`: num2cell(7) => 1x1 cell.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_scalar_yields_1x1_cell() {
        let cell = expect_cell(run_num2cell(Value::Num(7.0), vec![]).expect("num2cell"));
        assert_eq!(cell.shape, vec![1, 1]);
        assert_eq!(cell.data, vec![Value::Num(7.0)]);
    }

    // unresolved_choice [FR-005-06; num2cell.q-shape]: empty inputs are
    // unobserved; the empty shape is preserved (0x0 cell).
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_empty_matrix_yields_0x0_cell() {
        let tensor = Tensor::new(Vec::new(), vec![0, 0]).unwrap();
        let cell = expect_cell(run_num2cell(Value::Tensor(tensor), vec![]).expect("num2cell"));
        assert_eq!(cell.shape, vec![0, 0]);
        assert!(cell.data.is_empty());
    }

    // unresolved_choice [FR-005-06; num2cell.q-shape]: dimension vectors are
    // unobserved; grouping along [1 2] collapses both dimensions into a
    // single cell holding the whole matrix.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_dim_vector_groups_both_dimensions() {
        let tensor = Tensor::new(vec![1.0, 3.0, 2.0, 4.0], vec![2, 2]).unwrap();
        let dims = Value::Tensor(Tensor::new(vec![1.0, 2.0], vec![1, 2]).unwrap());
        let cell = expect_cell(run_num2cell(Value::Tensor(tensor), vec![dims]).expect("num2cell"));
        assert_eq!(cell.shape, vec![1, 1]);
        let gathered = test_support::gather(cell.data[0].clone()).expect("gather");
        assert_eq!(gathered.shape, vec![2, 2]);
        assert_eq!(gathered.data, vec![1.0, 3.0, 2.0, 4.0]);
    }

    // unresolved_choice [FR-005-06; num2cell.q-shape]: char inputs are
    // unobserved; each cell holds a 1x1 char array by default.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_char_row_yields_char_cells() {
        let chars = CharArray::new_row("ab");
        let cell = expect_cell(run_num2cell(Value::CharArray(chars), vec![]).expect("num2cell"));
        assert_eq!(cell.shape, vec![1, 2]);
        for (value, expected) in cell.data.iter().zip(['a', 'b']) {
            match value {
                Value::CharArray(ca) => {
                    assert_eq!(ca.rows, 1);
                    assert_eq!(ca.cols, 1);
                    assert_eq!(ca.data, vec![expected]);
                }
                other => panic!("expected char cell, got {other:?}"),
            }
        }
    }

    // unresolved_choice [FR-005-06; num2cell.q-shape]: error conditions are
    // unobserved; non-positive or fractional dimensions are rejected.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_invalid_dim_errors() {
        let tensor = Tensor::new(vec![1.0, 2.0], vec![1, 2]).unwrap();
        for bad in [Value::Num(0.0), Value::Num(1.5)] {
            let err = run_num2cell(Value::Tensor(tensor.clone()), vec![bad]).unwrap_err();
            assert!(
                err.to_string().contains("positive integers"),
                "unexpected error message: {err}"
            );
        }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn too_many_arguments_error() {
        let tensor = Tensor::new(vec![1.0], vec![1, 1]).unwrap();
        let err = run_num2cell(
            Value::Tensor(tensor),
            vec![Value::Num(1.0), Value::Num(2.0)],
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("at most one dimension argument"),
            "unexpected error message: {err}"
        );
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn gpu_input_gathers_to_host() {
        test_support::with_test_provider(|provider| {
            let tensor = Tensor::new(vec![10.0, 20.0, 30.0], vec![1, 3]).unwrap();
            let view = runmat_accelerate_api::HostTensorView {
                data: &tensor.data,
                shape: &tensor.shape,
            };
            let handle = provider.upload(&view).expect("upload");
            let cell =
                expect_cell(run_num2cell(Value::GpuTensor(handle), vec![]).expect("num2cell"));
            assert_eq!(cell.shape, vec![1, 3]);
            assert_eq!(
                cell.data,
                vec![Value::Num(10.0), Value::Num(20.0), Value::Num(30.0)]
            );
        });
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn descriptor_signatures_cover_num2cell_forms() {
        let labels: Vec<&str> = NUM2CELL_DESCRIPTOR
            .signatures
            .iter()
            .map(|sig| sig.label)
            .collect();
        assert!(labels.contains(&"C = num2cell(A)"));
        assert!(labels.contains(&"C = num2cell(A, dim)"));
    }
}
