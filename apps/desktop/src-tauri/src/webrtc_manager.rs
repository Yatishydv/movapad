use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use webrtc::api::APIBuilder;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data_channel::RTCDataChannel;

use crate::input::MouseController;
use crate::protocol;

pub struct WebRtcManager {
    peer_connection: Arc<RTCPeerConnection>,
    data_channel: Arc<Mutex<Option<Arc<RTCDataChannel>>>>,
    mouse: Arc<dyn MouseController>,
    sensitivity: f64,
}

impl WebRtcManager {
    pub async fn new<F>(
        mouse: Arc<dyn MouseController>,
        sensitivity: f64,
        on_ice_candidate: F,
    ) -> Result<Arc<Self>, Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn(RTCIceCandidateInit) + Send + Sync + 'static,
    {
        // Create a MediaEngine object to configure the supported codec
        let mut m = webrtc::api::media_engine::MediaEngine::default();
        m.register_default_codecs()?;

        // Create a InterceptorRegistry
        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut m)?;

        // Create the API object with the MediaEngine
        let api = APIBuilder::new()
            .with_media_engine(m)
            .with_interceptor_registry(registry)
            .build();

        // Prepare the configuration
        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        // Create a new RTCPeerConnection
        let peer_connection = Arc::new(api.new_peer_connection(config).await?);
        let data_channel_wrap = Arc::new(Mutex::new(None));

        let manager = Arc::new(Self {
            peer_connection: Arc::clone(&peer_connection),
            data_channel: Arc::clone(&data_channel_wrap),
            mouse,
            sensitivity,
        });

        // Set the handler for ICE connection state
        peer_connection.on_ice_connection_state_change(Box::new(move |connection_state| {
            log::info!("ICE Connection State has changed: {connection_state}");
            Box::pin(async {})
        }));

        peer_connection.on_ice_candidate(Box::new(move |c: Option<webrtc::ice_transport::ice_candidate::RTCIceCandidate>| {
            if let Some(candidate) = c {
                if let Ok(init) = candidate.to_json() {
                    on_ice_candidate(init);
                }
            }
            Box::pin(async {})
        }));

        Ok(manager)
    }

    pub async fn create_data_channel(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let dc = self.peer_connection.create_data_channel("movapad", None).await?;
        
        let mouse_clone = Arc::clone(&self.mouse);
        let sens = self.sensitivity;

        dc.on_open(Box::new(move || {
            log::info!("Data channel opened!");
            Box::pin(async {})
        }));

        dc.on_message(Box::new(move |msg: DataChannelMessage| {
            if msg.is_string { return Box::pin(async {}); }
            let data = msg.data;
            match protocol::decode(&data) {
                Ok(protocol_msg) => {
                    handle_message(&*mouse_clone, &protocol_msg, sens);
                }
                Err(e) => {
                    log::warn!("Protocol decode error: {e}");
                }
            }
            Box::pin(async {})
        }));

        let mut lock = self.data_channel.lock().await;
        *lock = Some(dc);
        
        Ok(())
    }

    pub async fn create_offer(&self) -> Result<RTCSessionDescription, Box<dyn std::error::Error + Send + Sync>> {
        let offer = self.peer_connection.create_offer(None).await?;
        self.peer_connection.set_local_description(offer.clone()).await?;
        Ok(offer)
    }

    pub async fn set_remote_description(&self, sdp: RTCSessionDescription) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.peer_connection.set_remote_description(sdp).await?;
        Ok(())
    }

    pub async fn add_ice_candidate(&self, candidate: RTCIceCandidateInit) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.peer_connection.add_ice_candidate(candidate).await?;
        Ok(())
    }
}

fn handle_message(mouse: &dyn MouseController, msg: &protocol::ProtocolMessage, sensitivity: f64) {
    let result = match msg {
        protocol::ProtocolMessage::Move { dx, dy } => {
            let scaled_dx = (*dx as f64 * sensitivity).round() as i32;
            let scaled_dy = (*dy as f64 * sensitivity).round() as i32;
            mouse.move_relative(scaled_dx, scaled_dy)
        }
        protocol::ProtocolMessage::LeftDown => mouse.left_down(),
        protocol::ProtocolMessage::LeftUp => mouse.left_up(),
        protocol::ProtocolMessage::RightDown => mouse.right_down(),
        protocol::ProtocolMessage::RightUp => mouse.right_up(),
        protocol::ProtocolMessage::Scroll { dx, dy } => {
            mouse.scroll(*dx as i32, *dy as i32)
        }
        protocol::ProtocolMessage::DragStart => mouse.left_down(),
        protocol::ProtocolMessage::DragEnd => mouse.left_up(),
        protocol::ProtocolMessage::Ping { seq } => {
            log::debug!("PING seq={seq}");
            Ok(())
        }
        protocol::ProtocolMessage::SessionStart { version, .. } => {
            log::info!("Session started (protocol v{version})");
            Ok(())
        }
        protocol::ProtocolMessage::SessionEnd => {
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
