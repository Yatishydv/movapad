//! MovaPad Desktop — Phase 1 WebSocket Server
//!
//! For local development, the desktop app runs a WebSocket server that
//! the phone connects to directly (same network). This will be replaced
//! by WebRTC DataChannel in Phase 3.
//!
//! The server accepts binary messages in the MovaPad protocol format
//! and dispatches them to the mouse controller.

use std::sync::Arc;

use futures_util::StreamExt;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;

use crate::input::MouseController;
use crate::protocol::{self, ProtocolMessage};

/// Start the local WebSocket server for Phase 1 development.
///
/// Listens on the given address and processes incoming protocol messages.
/// Only allows one active connection at a time.
pub async fn start_local_server(
    addr: &str,
    mouse: Arc<dyn MouseController>,
    sensitivity: f64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind(addr).await?;
    log::info!("MovaPad local dev server listening on {addr}");

    loop {
        let (stream, peer) = listener.accept().await?;
        log::info!("Phone connected from {peer}");

        let mouse = Arc::clone(&mouse);
        let sens = sensitivity;

        tokio::spawn(async move {
            let ws = match accept_async(stream).await {
                Ok(ws) => ws,
                Err(e) => {
                    log::error!("WebSocket handshake failed: {e}");
                    return;
                }
            };

            let (_write, mut read) = ws.split();

            while let Some(msg) = read.next().await {
                let msg = match msg {
                    Ok(m) => m,
                    Err(e) => {
                        log::warn!("WebSocket read error: {e}");
                        break;
                    }
                };

                if msg.is_binary() {
                    let data = msg.into_data();
                    match protocol::decode(&data) {
                        Ok(protocol_msg) => {
                            handle_message(&*mouse, &protocol_msg, sens);
                        }
                        Err(e) => {
                            log::warn!("Protocol decode error: {e}");
                        }
                    }
                }
            }

            // Connection closed — reset mouse state to prevent stuck buttons
            log::info!("Phone disconnected from {peer}");
            if let Err(e) = mouse.reset() {
                log::error!("Failed to reset mouse state: {e}");
            }
        });
    }
}

/// Dispatch a decoded protocol message to the mouse controller.
fn handle_message(mouse: &dyn MouseController, msg: &ProtocolMessage, sensitivity: f64) {
    let result = match msg {
        ProtocolMessage::Move { dx, dy } => {
            let scaled_dx = (*dx as f64 * sensitivity).round() as i32;
            let scaled_dy = (*dy as f64 * sensitivity).round() as i32;
            mouse.move_relative(scaled_dx, scaled_dy)
        }
        ProtocolMessage::LeftDown => mouse.left_down(),
        ProtocolMessage::LeftUp => mouse.left_up(),
        ProtocolMessage::RightDown => mouse.right_down(),
        ProtocolMessage::RightUp => mouse.right_up(),
        ProtocolMessage::Scroll { dx, dy } => {
            mouse.scroll(*dx as i32, *dy as i32)
        }
        ProtocolMessage::DragStart => mouse.left_down(),
        ProtocolMessage::DragEnd => mouse.left_up(),
        ProtocolMessage::Ping { seq } => {
            log::debug!("PING seq={seq}");
            Ok(())
        }
        ProtocolMessage::SessionStart { version, .. } => {
            log::info!("Session started (protocol v{version})");
            Ok(())
        }
        ProtocolMessage::SessionEnd => {
            log::info!("Session ended");
            mouse.reset()
        }
        _ => {
            log::debug!("Unhandled message: {msg:?}");
            Ok(())
        }
    };

    if let Err(e) = result {
        log::error!("Mouse controller error: {e}");
    }
}
