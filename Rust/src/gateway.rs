use pyo3::prelude::*;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration, interval};
use crate::errors::DiscordError;
use crate::enums::Intents;

const GATEWAY_VERSION: u8 = 10;
const GATEWAY_ENCODING: &str = "json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayOpcode {
    Dispatch = 0,
    Heartbeat = 1,
    Identify = 2,
    PresenceUpdate = 3,
    VoiceStateUpdate = 4,
    Resume = 6,
    Reconnect = 7,
    RequestGuildMembers = 8,
    InvalidSession = 9,
    Hello = 10,
    HeartbeatAck = 11,
}

impl GatewayOpcode {
    pub fn from_u64(value: u64) -> Option<Self> {
        match value {
            0 => Some(GatewayOpcode::Dispatch),
            1 => Some(GatewayOpcode::Heartbeat),
            2 => Some(GatewayOpcode::Identify),
            3 => Some(GatewayOpcode::PresenceUpdate),
            4 => Some(GatewayOpcode::VoiceStateUpdate),
            6 => Some(GatewayOpcode::Resume),
            7 => Some(GatewayOpcode::Reconnect),
            8 => Some(GatewayOpcode::RequestGuildMembers),
            9 => Some(GatewayOpcode::InvalidSession),
            10 => Some(GatewayOpcode::Hello),
            11 => Some(GatewayOpcode::HeartbeatAck),
            _ => None,
        }
    }
}

pub struct Gateway {
    ws: Arc<RwLock<Option<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
    token: String,
    intents: Intents,
    sequence: Arc<RwLock<Option<u64>>>,
    session_id: Arc<RwLock<Option<String>>>,
    heartbeat_interval: Arc<RwLock<Option<u64>>>,
    last_heartbeat_ack: Arc<RwLock<bool>>,
}

impl Gateway {
    pub fn new(token: String, intents: Intents) -> Self {
        Gateway {
            ws: Arc::new(RwLock::new(None)),
            token,
            intents,
            sequence: Arc::new(RwLock::new(None)),
            session_id: Arc::new(RwLock::new(None)),
            heartbeat_interval: Arc::new(RwLock::new(None)),
            last_heartbeat_ack: Arc::new(RwLock::new(true)),
        }
    }

    pub async fn connect(&self, gateway_url: &str) -> Result<(), DiscordError> {
        let url = format!("{}/?v={}&encoding={}", gateway_url, GATEWAY_VERSION, GATEWAY_ENCODING);
        let (ws_stream, _) = connect_async(url).await?;
        *self.ws.write().await = Some(ws_stream);
        Ok(())
    }

    pub async fn send_identify(&self) -> Result<(), DiscordError> {
        let identify = json!({
            "op": GatewayOpcode::Identify as u8,
            "d": {
                "token": self.token,
                "intents": self.intents.value,
                "properties": {
                    "$os": std::env::consts::OS,
                    "$browser": "discord.py-rust",
                    "$device": "discord.py-rust"
                }
            }
        });

        self.send_json(&identify).await
    }

    pub async fn send_heartbeat(&self) -> Result<(), DiscordError> {
        let seq = *self.sequence.read().await;
        let heartbeat = json!({
            "op": GatewayOpcode::Heartbeat as u8,
            "d": seq
        });

        *self.last_heartbeat_ack.write().await = false;
        self.send_json(&heartbeat).await
    }

    async fn send_json(&self, data: &Value) -> Result<(), DiscordError> {
        let msg = Message::Text(data.to_string());
        let mut ws_guard = self.ws.write().await;
        if let Some(ws) = &mut *ws_guard {
            ws.send(msg).await?;
        }
        Ok(())
    }

    pub async fn receive(&self) -> Result<Option<Value>, DiscordError> {
        let mut ws_guard = self.ws.write().await;
        if let Some(ws) = &mut *ws_guard {
            if let Some(msg) = ws.next().await {
                let msg = msg?;
                match msg {
                    Message::Text(text) => {
                        let data: Value = serde_json::from_str(&text)?;
                        Ok(Some(data))
                    }
                    Message::Binary(bin) => {
                        // Handle zlib-compressed payloads if needed
                        let text = String::from_utf8_lossy(&bin);
                        let data: Value = serde_json::from_str(&text)?;
                        Ok(Some(data))
                    }
                    Message::Close(frame) => {
                        let code = frame.as_ref().map(|f| f.code.into()).unwrap_or(1000);
                        let reason = frame.as_ref().map(|f| f.reason.to_string()).unwrap_or_default();
                        Err(DiscordError::ConnectionClosed {
                            code: code as i32,
                            reason,
                        })
                    }
                    _ => Ok(None),
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn handle_payload(&self, payload: &Value) -> Result<(), DiscordError> {
        let op = payload["op"].as_u64().unwrap_or(0);
        let opcode = GatewayOpcode::from_u64(op);

        if let Some(seq) = payload["s"].as_u64() {
            *self.sequence.write().await = Some(seq);
        }

        match opcode {
            Some(GatewayOpcode::Hello) => {
                let heartbeat_interval = payload["d"]["heartbeat_interval"].as_u64().unwrap_or(41250);
                *self.heartbeat_interval.write().await = Some(heartbeat_interval);
                self.send_identify().await?;
            }
            Some(GatewayOpcode::HeartbeatAck) => {
                *self.last_heartbeat_ack.write().await = true;
            }
            Some(GatewayOpcode::Dispatch) => {
                if let Some(event_name) = payload["t"].as_str() {
                    if event_name == "READY" {
                        if let Some(session_id) = payload["d"]["session_id"].as_str() {
                            *self.session_id.write().await = Some(session_id.to_string());
                        }
                    }
                }
            }
            Some(GatewayOpcode::Reconnect) => {
                // Handle reconnect
            }
            Some(GatewayOpcode::InvalidSession) => {
                // Handle invalid session
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn start_heartbeat(&self) {
        if let Some(interval_ms) = *self.heartbeat_interval.read().await {
            let sequence = Arc::clone(&self.sequence);
            let last_heartbeat_ack = Arc::clone(&self.last_heartbeat_ack);
            let ws = Arc::clone(&self.ws);

            tokio::spawn(async move {
                let mut interval_timer = interval(Duration::from_millis(interval_ms));
                loop {
                    interval_timer.tick().await;

                    // Send heartbeat
                    let seq = *sequence.read().await;
                    let heartbeat = json!({
                        "op": GatewayOpcode::Heartbeat as u8,
                        "d": seq
                    });

                    *last_heartbeat_ack.write().await = false;

                    {
                        let mut ws_guard = ws.write().await;
                        if let Some(ws_stream) = &mut *ws_guard {
                            let msg = Message::Text(heartbeat.to_string());
                            if let Err(e) = ws_stream.send(msg).await {
                                eprintln!("Heartbeat error: {}", e);
                                break;
                            }
                        }
                    }

                    // Check if we got an ack
                    sleep(Duration::from_secs(5)).await;
                    if !*last_heartbeat_ack.read().await {
                        eprintln!("No heartbeat ack received, connection may be dead");
                        break;
                    }
                }
            });
        }
    }

    fn clone_for_heartbeat(&self) -> Self {
        Gateway {
            ws: Arc::clone(&self.ws),
            token: self.token.clone(),
            intents: self.intents,
            sequence: Arc::clone(&self.sequence),
            session_id: Arc::clone(&self.session_id),
            heartbeat_interval: Arc::clone(&self.heartbeat_interval),
            last_heartbeat_ack: Arc::clone(&self.last_heartbeat_ack),
        }
    }
}

pub fn register_module(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
