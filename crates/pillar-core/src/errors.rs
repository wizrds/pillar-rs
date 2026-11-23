/// The error type returned by all pillar operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A failure to establish or use a database connection.
    #[error("connection error: {message}")]
    Connection {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// A query was malformed or could not be constructed.
    #[error("invalid query: {message}")]
    InvalidQuery {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// An underlying I/O error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// A failure to serialize or deserialize data.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// An unexpected error with no more specific category.
    #[error("unexpected error: {message}")]
    Unexpected {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl Error {
    /// Creates an [`Error::Connection`](crate::errors::Error::Connection) with the given message.
    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
            source: None,
        }
    }

    /// Creates an [`Error::Connection`](crate::errors::Error::Connection) with the given source error.
    pub fn sourced_connection<E>(source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Connection {
            message: source.to_string(),
            source: Some(Box::new(source)),
        }
    }

    /// Creates an [`Error::InvalidQuery`](crate::errors::Error::InvalidQuery) with the given message.
    pub fn invalid_query(message: impl Into<String>) -> Self {
        Self::InvalidQuery {
            message: message.into(),
            source: None,
        }
    }

    /// Creates an [`Error::InvalidQuery`](crate::errors::Error::InvalidQuery) with the given source error.
    pub fn sourced_invalid_query<E>(source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::InvalidQuery {
            message: source.to_string(),
            source: Some(Box::new(source)),
        }
    }

    /// Creates an [`Error::Io`](crate::errors::Error::Io) from the given I/O error.
    pub fn io(error: std::io::Error) -> Self {
        Self::Io(error)
    }

    /// Creates an [`Error::Serialization`](crate::errors::Error::Serialization) with the given message.
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::Serialization(message.into())
    }

    /// Creates an [`Error::Unexpected`](crate::errors::Error::Unexpected) with the given message.
    pub fn unexpected(message: impl Into<String>) -> Self {
        Self::Unexpected {
            message: message.into(),
            source: None,
        }
    }

    /// Creates an [`Error::Unexpected`](crate::errors::Error::Unexpected) with the given source error.
    pub fn sourced_unexpected<E>(source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Unexpected {
            message: source.to_string(),
            source: Some(Box::new(source)),
        }
    }
}
