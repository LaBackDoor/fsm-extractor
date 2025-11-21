use thiserror::Error;

#[derive(Error, Debug)]
pub enum FsmError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("XML parsing error: {0}")]
    XmlParse(String),

    #[error("No function blocks found in XML")]
    NoFunctionBlocks,

    #[error("Function block '{0}' not found")]
    FunctionBlockNotFound(String),

    #[error("No case statement found in function block '{0}'")]
    NoCaseStatement(String),

    #[error("Invalid state reference: {0}")]
    InvalidStateReference(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}