use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum SignalingMessage {
    #[serde(rename = "CREATE_ROOM")]
    CreateRoom,
    #[serde(rename = "ROOM_CREATED")]
    RoomCreated { roomId: String, pin: String },
    #[serde(rename = "PEER_CONNECTED")]
    PeerConnected,
    #[serde(rename = "PEER_DISCONNECTED")]
    PeerDisconnected,
    #[serde(rename = "offer")]
    Offer { sdp: String },
    #[serde(rename = "answer")]
    Answer { sdp: String },
    #[serde(rename = "candidate")]
    Candidate { candidate: String, sdpMid: Option<String>, sdpMLineIndex: Option<u16> },
    #[serde(rename = "ERROR")]
    Error { message: String },
}

pub struct SignalingClient {
    tx: mpsc::Sender<Message>,
}

impl SignalingClient {
    pub async fn connect(url: &str) -> Result<(Self, mpsc::Receiver<SignalingMessage>), Box<dyn std::error::Error + Send + Sync>> {
        let (ws_stream, _) = connect_async(url).await?;
        let (mut write, mut read) = ws_stream.split();
        
        let (tx, mut rx) = mpsc::channel::<Message>(32);
        let (msg_tx, msg_rx) = mpsc::channel::<SignalingMessage>(32);

        // Write loop
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if write.send(msg).await.is_err() {
                    break;
                }
            }
        });

        // Read loop
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                if let Ok(Message::Text(text)) = msg {
                    if let Ok(signaling_msg) = serde_json::from_str::<SignalingMessage>(&text) {
                        if msg_tx.send(signaling_msg).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });

        Ok((Self { tx }, msg_rx))
    }

    pub async fn create_room(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let msg = serde_json::to_string(&SignalingMessage::CreateRoom)?;
        self.tx.send(Message::Text(msg.into())).await?;
        Ok(())
    }

    pub async fn send_offer(&self, sdp: RTCSessionDescription) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let msg = serde_json::to_string(&SignalingMessage::Offer { sdp: sdp.sdp })?;
        self.tx.send(Message::Text(msg.into())).await?;
        Ok(())
    }

    pub async fn send_candidate(&self, candidate: RTCIceCandidateInit) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let msg = serde_json::to_string(&SignalingMessage::Candidate { 
            candidate: candidate.candidate, 
            sdpMid: candidate.sdp_mid, 
            sdpMLineIndex: candidate.sdp_mline_index 
        })?;
        self.tx.send(Message::Text(msg.into())).await?;
        Ok(())
    }
}
