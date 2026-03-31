use chrono::ParseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AtlasError {
    /// Not found error.
    #[error("Not found error: {0}")]
    NotFoundErr(String),
    // Date parsing error.
    #[error("Date parsing error: {0}")]
    DateParsingErr(#[from] ParseError),
    // Period parsing error.
    #[error("Period parsing error: {0}")]
    PeriodParsingErr(String),
    // Period operation error
    #[error("Period operation error: {0}")]
    PeriodOperationErr(String),
    #[error("MakeSchedule error: {0}")]
    MakeScheduleErr(String),
    #[error("Evaluation error: {0}")]
    EvaluationErr(String),
    #[error("Serialization error: {0}")]
    SerializationErr(String),
    #[error("Deserialization error: {0}")]
    DeserializationErr(String),
    #[error("Value not set error: {0}")]
    ValueNotSetErr(String),
    #[error("Invalid value error: {0}")]
    MissingRequiredField(String),
    #[error("Invalid value: {0}")]
    InvalidValueErr(String),
    #[error("{0}")]
    NotImplementedErr(String),
    #[error("Invalid configuration error: {0}")]
    InvalidConfigurationErr(String),
    #[error("Indexing error: {0}")]
    IndexingErr(String),
    #[error("Bootstrapping error: {0}")]
    BootstrappingErr(String),
    #[error("Solver Error: {0}")]
    SolverError(#[from] crate::math::solver::traits::SolverError),
    #[error("Interpolation error: {0}")]
    InterpolationErr(String),
    #[error("Poisoned Cache error: {0}")]
    PoisonedCacheErr(String),
    /// A generic tape error.
    #[error("Tape error: {0}")]
    TapeError(String),
    /// A generic AD number error.
    #[error("AD Real error: {0}")]
    ADRealError(String),
    /// Attempted to access a node that is not recorded on the tape.
    #[error("Node not indexed in tape")]
    NodeNotIndexedInTapeErr,
}

pub type Result<T> = std::result::Result<T, AtlasError>;
