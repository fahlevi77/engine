use thiserror::Error;

/// Main error type for EventFlux operations
#[derive(Error, Debug)]
pub enum EventFluxError {
    /// Errors that occur during EventFlux app creation and parsing
    #[error("EventFlux app creation error: {message}")]
    EventFluxAppCreation {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Runtime errors during query execution
    #[error("EventFlux app runtime error: {message}")]
    EventFluxAppRuntime {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Errors in query parsing and compilation
    #[error("Query creation error: {message}")]
    QueryCreation {
        message: String,
        query: Option<String>,
    },

    /// Runtime errors in query execution
    #[error("Query runtime error: {message}")]
    QueryRuntime {
        message: String,
        query_name: Option<String>,
    },

    /// Errors in on-demand query creation
    #[error("On-demand query creation error: {message}")]
    OnDemandQueryCreation { message: String },

    /// Runtime errors in on-demand query execution
    #[error("On-demand query runtime error: {message}")]
    OnDemandQueryRuntime { message: String },

    /// Store query errors
    #[error("Store query error: {message}")]
    StoreQuery {
        message: String,
        store_name: Option<String>,
    },

    /// Definition not found errors
    #[error("{definition_type} definition '{name}' does not exist")]
    DefinitionNotExist {
        definition_type: String,
        name: String,
    },

    /// Query not found errors
    #[error("Query '{name}' does not exist in EventFlux app '{app_name}'")]
    QueryNotExist { name: String, app_name: String },

    /// Attribute not found errors
    #[error("Attribute '{attribute}' does not exist in {context}")]
    NoSuchAttribute { attribute: String, context: String },

    /// Extension not found errors
    #[error("{extension_type} extension '{name}' not found")]
    ExtensionNotFound {
        extension_type: String,
        name: String,
    },

    /// Connection unavailable errors
    #[error("Connection unavailable: {message}")]
    ConnectionUnavailable {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Database runtime errors
    #[error("Database runtime error: {message}")]
    DatabaseRuntime {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Table/Store errors
    #[error("Queryable record table error: {message}")]
    QueryableRecordTable {
        message: String,
        table_name: Option<String>,
    },

    /// Persistence errors
    #[error("Persistence store error: {message}")]
    PersistenceStore {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// State persistence errors
    #[error("Cannot persist EventFlux app state: {message}")]
    CannotPersistState {
        message: String,
        app_name: Option<String>,
    },

    /// State restoration errors
    #[error("Cannot restore EventFlux app state: {message}")]
    CannotRestoreState {
        message: String,
        app_name: Option<String>,
    },

    /// State clearing errors
    #[error("Cannot clear EventFlux app state: {message}")]
    CannotClearState {
        message: String,
        app_name: Option<String>,
    },

    /// Operation not supported errors
    #[error("Operation not supported: {message}")]
    OperationNotSupported {
        message: String,
        operation: Option<String>,
    },

    /// Data type errors
    #[error("Type error: {message}")]
    TypeError {
        message: String,
        expected: Option<String>,
        actual: Option<String>,
    },

    /// Invalid parameter errors
    #[error("Invalid parameter: {message}")]
    InvalidParameter {
        message: String,
        parameter: Option<String>,
        expected: Option<String>,
    },

    /// Mapping/Marshalling errors
    #[error("Mapping failed: {message}")]
    MappingFailed {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        config_key: Option<String>,
    },

    /// Class loading errors (for extensions)
    #[error("Cannot load class/extension: {message}")]
    CannotLoadClass {
        message: String,
        class_name: Option<String>,
    },

    /// Parser errors from LALRPOP
    #[error("Parse error: {message}")]
    ParseError {
        message: String,
        line: Option<usize>,
        column: Option<usize>,
    },

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    /// Database errors from rusqlite
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// Send errors for channels
    #[error("Send error: {message}")]
    SendError { message: String },

    /// Processor errors
    #[error("Processor error: {message}")]
    ProcessorError {
        message: String,
        processor: Option<String>,
    },

    /// Generic other errors (to be phased out)
    #[error("{0}")]
    Other(String),
}

/// Result type alias for EventFlux operations
pub type EventFluxResult<T> = Result<T, EventFluxError>;

impl EventFluxError {
    /// Create a new EventFluxAppCreation error
    pub fn app_creation(message: impl Into<String>) -> Self {
        EventFluxError::EventFluxAppCreation {
            message: message.into(),
            source: None,
        }
    }

    /// Create a new EventFluxAppRuntime error
    pub fn app_runtime(message: impl Into<String>) -> Self {
        EventFluxError::EventFluxAppRuntime {
            message: message.into(),
            source: None,
        }
    }

    /// Create a new TypeError
    pub fn type_error(
        message: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        EventFluxError::TypeError {
            message: message.into(),
            expected: Some(expected.into()),
            actual: Some(actual.into()),
        }
    }

    /// Create a new InvalidParameter error
    pub fn invalid_parameter(message: impl Into<String>, parameter: impl Into<String>) -> Self {
        EventFluxError::InvalidParameter {
            message: message.into(),
            parameter: Some(parameter.into()),
            expected: None,
        }
    }

    /// Create a new ExtensionNotFound error
    pub fn extension_not_found(extension_type: impl Into<String>, name: impl Into<String>) -> Self {
        EventFluxError::ExtensionNotFound {
            extension_type: extension_type.into(),
            name: name.into(),
        }
    }

    /// Create a new DefinitionNotExist error
    pub fn definition_not_exist(
        definition_type: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        EventFluxError::DefinitionNotExist {
            definition_type: definition_type.into(),
            name: name.into(),
        }
    }

    /// Add source error context
    pub fn with_source(mut self, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        match &mut self {
            EventFluxError::EventFluxAppCreation { source: src, .. }
            | EventFluxError::EventFluxAppRuntime { source: src, .. }
            | EventFluxError::ConnectionUnavailable { source: src, .. }
            | EventFluxError::DatabaseRuntime { source: src, .. }
            | EventFluxError::PersistenceStore { source: src, .. }
            | EventFluxError::MappingFailed { source: src, .. } => {
                *src = Some(Box::new(source));
            }
            _ => {}
        }
        self
    }
}

/// Convert String errors to EventFluxError::Other (for backward compatibility)
impl From<String> for EventFluxError {
    fn from(s: String) -> Self {
        EventFluxError::Other(s)
    }
}

/// Convert &str errors to EventFluxError::Other (for backward compatibility)
impl From<&str> for EventFluxError {
    fn from(s: &str) -> Self {
        EventFluxError::Other(s.to_string())
    }
}

/// Extension trait for converting Results with String errors to EventFluxError
pub trait IntoEventFluxResult<T> {
    fn into_eventflux_result(self) -> EventFluxResult<T>;
}

impl<T> IntoEventFluxResult<T> for Result<T, String> {
    fn into_eventflux_result(self) -> EventFluxResult<T> {
        self.map_err(EventFluxError::from)
    }
}
