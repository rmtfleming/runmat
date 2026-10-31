#[derive(Debug, Clone, PartialEq)]
pub struct MException {
    pub identifier: String,
    pub message: String,
    pub stack: Vec<String>,
    /// Exceptions added with `addCause`, in the order they were added.
    pub causes: Vec<MException>,
}

impl MException {
    pub fn new(identifier: String, message: String) -> Self {
        Self {
            identifier,
            message,
            stack: Vec::new(),
            causes: Vec::new(),
        }
    }

    /// Return a copy of this exception with `cause` appended to its causes.
    pub fn with_cause(mut self, cause: MException) -> Self {
        self.causes.push(cause);
        self
    }
}

/// The causes of a thrown exception, carried as the `source` of the runtime
/// error that raises it so that `catch` can restore them. Defined here (not
/// in the runtime) so every crate that downcasts the source agrees on the
/// type.
#[derive(Debug, Clone, PartialEq)]
pub struct ExceptionCauses(pub Vec<MException>);

impl std::fmt::Display for ExceptionCauses {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ids: Vec<&str> = self.0.iter().map(|c| c.identifier.as_str()).collect();
        write!(f, "caused by: {}", ids.join(", "))
    }
}

impl std::error::Error for ExceptionCauses {}
