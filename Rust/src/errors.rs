use pyo3::prelude::*;
use pyo3::exceptions::PyException;
use pyo3::create_exception;

// Base Discord exception
create_exception!(discord, DiscordException, PyException, "Base exception class for discord.py");

// Client exceptions
create_exception!(discord, ClientException, DiscordException, "Exception that's raised when an operation in the Client fails.");
create_exception!(discord, GatewayNotFound, DiscordException, "An exception that is raised when the gateway for Discord could not be found");
create_exception!(discord, InvalidData, ClientException, "Exception that's raised when the library encounters unknown or invalid data from Discord.");
create_exception!(discord, LoginFailure, ClientException, "Exception that's raised when the Client.login function fails to log you in from improper credentials or some other misc. failure.");
create_exception!(discord, ConnectionClosed, ClientException, "Exception that's raised when the gateway connection is closed for reasons that could not be handled internally.");
create_exception!(discord, PrivilegedIntentsRequired, ClientException, "Exception that's raised when the gateway is requesting privileged intents but they're not ticked in the developer page yet.");
create_exception!(discord, InteractionResponded, ClientException, "Exception that's raised when sending another interaction response using InteractionResponse when one has already been done before.");
create_exception!(discord, MissingApplicationID, ClientException, "An exception raised when the client does not have an application ID set.");

// HTTP exceptions
create_exception!(discord, HTTPException, DiscordException, "Exception that's raised when an HTTP request operation fails.");
create_exception!(discord, RateLimited, DiscordException, "Exception that's raised for when status code 429 occurs and the timeout is greater than the configured maximum.");
create_exception!(discord, Forbidden, HTTPException, "Exception that's raised for when status code 403 occurs.");
create_exception!(discord, NotFound, HTTPException, "Exception that's raised for when status code 404 occurs.");
create_exception!(discord, DiscordServerError, HTTPException, "Exception that's raised for when a 500 range status code occurs.");

/// Custom error type used internally
#[derive(Debug)]
pub enum DiscordError {
    Http(reqwest::Error),
    WebSocket(tokio_tungstenite::tungstenite::Error),
    Json(serde_json::Error),
    Gateway(String),
    InvalidData(String),
    ConnectionClosed { code: i32, reason: String },
    RateLimited { retry_after: f64 },
    Forbidden,
    NotFound,
    ServerError,
}

impl std::fmt::Display for DiscordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscordError::Http(e) => write!(f, "HTTP error: {}", e),
            DiscordError::WebSocket(e) => write!(f, "WebSocket error: {}", e),
            DiscordError::Json(e) => write!(f, "JSON error: {}", e),
            DiscordError::Gateway(s) => write!(f, "Gateway error: {}", s),
            DiscordError::InvalidData(s) => write!(f, "Invalid data: {}", s),
            DiscordError::ConnectionClosed { code, reason } => {
                write!(f, "Connection closed with code {}: {}", code, reason)
            }
            DiscordError::RateLimited { retry_after } => {
                write!(f, "Rate limited, retry after {} seconds", retry_after)
            }
            DiscordError::Forbidden => write!(f, "Forbidden (403)"),
            DiscordError::NotFound => write!(f, "Not found (404)"),
            DiscordError::ServerError => write!(f, "Discord server error (5xx)"),
        }
    }
}

impl std::error::Error for DiscordError {}

impl From<reqwest::Error> for DiscordError {
    fn from(e: reqwest::Error) -> Self {
        DiscordError::Http(e)
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for DiscordError {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
        DiscordError::WebSocket(e)
    }
}

impl From<serde_json::Error> for DiscordError {
    fn from(e: serde_json::Error) -> Self {
        DiscordError::Json(e)
    }
}

impl From<DiscordError> for PyErr {
    fn from(err: DiscordError) -> PyErr {
        match err {
            DiscordError::Http(e) => HTTPException::new_err(format!("HTTP error: {}", e)),
            DiscordError::WebSocket(e) => ConnectionClosed::new_err(format!("WebSocket error: {}", e)),
            DiscordError::Json(e) => InvalidData::new_err(format!("JSON error: {}", e)),
            DiscordError::Gateway(s) => GatewayNotFound::new_err(s),
            DiscordError::InvalidData(s) => InvalidData::new_err(s),
            DiscordError::ConnectionClosed { code, reason } => {
                ConnectionClosed::new_err(format!("Connection closed with code {}: {}", code, reason))
            }
            DiscordError::RateLimited { retry_after } => {
                RateLimited::new_err(format!("Rate limited, retry after {} seconds", retry_after))
            }
            DiscordError::Forbidden => Forbidden::new_err("Forbidden (403)"),
            DiscordError::NotFound => NotFound::new_err("Not found (404)"),
            DiscordError::ServerError => DiscordServerError::new_err("Discord server error (5xx)"),
        }
    }
}

pub fn register_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    m.add("DiscordException", py.get_type_bound::<DiscordException>())?;
    m.add("ClientException", py.get_type_bound::<ClientException>())?;
    m.add("GatewayNotFound", py.get_type_bound::<GatewayNotFound>())?;
    m.add("HTTPException", py.get_type_bound::<HTTPException>())?;
    m.add("RateLimited", py.get_type_bound::<RateLimited>())?;
    m.add("Forbidden", py.get_type_bound::<Forbidden>())?;
    m.add("NotFound", py.get_type_bound::<NotFound>())?;
    m.add("DiscordServerError", py.get_type_bound::<DiscordServerError>())?;
    m.add("InvalidData", py.get_type_bound::<InvalidData>())?;
    m.add("LoginFailure", py.get_type_bound::<LoginFailure>())?;
    m.add("ConnectionClosed", py.get_type_bound::<ConnectionClosed>())?;
    m.add("PrivilegedIntentsRequired", py.get_type_bound::<PrivilegedIntentsRequired>())?;
    m.add("InteractionResponded", py.get_type_bound::<InteractionResponded>())?;
    m.add("MissingApplicationID", py.get_type_bound::<MissingApplicationID>())?;
    Ok(())
}
