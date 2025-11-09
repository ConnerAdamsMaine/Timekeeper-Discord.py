use pyo3::prelude::*;
use std::sync::Arc;
use crate::http::HTTPClient;
use crate::gateway::Gateway;
use crate::state::State;
use crate::enums::Intents;
use crate::errors::DiscordError;
use tokio::sync::RwLock;

/// Discord Client
#[pyclass]
pub struct Client {
    http: Arc<HTTPClient>,
    gateway: Arc<RwLock<Option<Gateway>>>,
    state: Arc<State>,
    token: String,
    intents: Intents,
}

#[pymethods]
impl Client {
    #[new]
    #[pyo3(signature = (*, intents=None))]
    fn new(intents: Option<Intents>) -> PyResult<Self> {
        let http = HTTPClient::new()
            .map_err(|e| PyErr::from(e))?;

        let intents = intents.unwrap_or_else(|| Intents::default());

        Ok(Client {
            http: Arc::new(http),
            gateway: Arc::new(RwLock::new(None)),
            state: Arc::new(State::new()),
            token: String::new(),
            intents,
        })
    }

    /// Run the bot with the given token
    fn run(&mut self, token: String) -> PyResult<()> {
        self.token = token.clone();
        let http = Arc::clone(&self.http);

        let client = self.clone_internals();
        let token_clone = token.clone();
        let intents = self.intents;

        // Run the async runtime in a blocking context
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                http.set_token(token_clone.clone()).await;
                client.connect(token_clone, intents).await
                    .map_err(|e| PyErr::from(e))
            })
    }

    fn __repr__(&self) -> String {
        format!("<Client intents={}>", self.intents.value)
    }
}

impl Client {
    fn clone_internals(&self) -> ClientInternal {
        ClientInternal {
            http: Arc::clone(&self.http),
            state: Arc::clone(&self.state),
        }
    }
}

struct ClientInternal {
    http: Arc<HTTPClient>,
    state: Arc<State>,
}

impl ClientInternal {
    async fn connect(&self, token: String, intents: Intents) -> Result<(), DiscordError> {
        // Get gateway URL
        let gateway_url = self.http.get_gateway().await?;

        // Create gateway connection
        let gateway = Gateway::new(token, intents);
        gateway.connect(&gateway_url).await?;

        // Start receiving events
        loop {
            if let Some(payload) = gateway.receive().await? {
                gateway.handle_payload(&payload).await?;

                // Start heartbeat after receiving HELLO
                if payload["op"].as_u64() == Some(10) {
                    gateway.start_heartbeat().await;
                }

                // Handle events
                if payload["op"].as_u64() == Some(0) {
                    // Dispatch event
                    if let Some(event_type) = payload["t"].as_str() {
                        println!("Received event: {}", event_type);
                        // Event handlers will go here
                    }
                }
            }
        }
    }
}

pub fn register_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Client>()?;
    Ok(())
}
