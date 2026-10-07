//! Connection state machine.
//!
//! Manages the lifecycle of a phone→desktop connection.
//! Single authoritative state — no scattered booleans.

use std::fmt;

/// Connection states — the single source of truth for connection lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// No connection.
    Disconnected,
    /// Attempting to establish connection.
    Connecting,
    /// Verifying device identity.
    Authenticating,
    /// Waiting for user to approve pairing.
    Pairing,
    /// Active session, mouse input flowing.
    Connected,
    /// Connection lost, attempting to re-establish.
    Reconnecting,
    /// Unrecoverable error state.
    Error,
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Connecting => write!(f, "Connecting"),
            Self::Authenticating => write!(f, "Authenticating"),
            Self::Pairing => write!(f, "Pairing"),
            Self::Connected => write!(f, "Connected"),
            Self::Reconnecting => write!(f, "Reconnecting"),
            Self::Error => write!(f, "Error"),
        }
    }
}

impl ConnectionState {
    /// Check if a transition is valid.
    pub fn can_transition_to(&self, next: ConnectionState) -> bool {
        matches!(
            (self, next),
            // Normal flow
            (Self::Disconnected, ConnectionState::Connecting)
            | (Self::Connecting, ConnectionState::Authenticating)
            | (Self::Authenticating, ConnectionState::Pairing)
            | (Self::Authenticating, ConnectionState::Connected)
            | (Self::Pairing, ConnectionState::Connected)
            // Disconnection
            | (Self::Connected, ConnectionState::Disconnected)
            | (Self::Connected, ConnectionState::Reconnecting)
            // Reconnection
            | (Self::Reconnecting, ConnectionState::Authenticating)
            | (Self::Reconnecting, ConnectionState::Disconnected)
            // Error transitions
            | (Self::Connecting, ConnectionState::Error)
            | (Self::Authenticating, ConnectionState::Error)
            | (Self::Pairing, ConnectionState::Error)
            | (Self::Connected, ConnectionState::Error)
            | (Self::Reconnecting, ConnectionState::Error)
            // Recovery
            | (Self::Error, ConnectionState::Disconnected)
            | (Self::Error, ConnectionState::Connecting)
        )
    }
}

/// Manages connection state with transition validation and subscriber notification.
pub struct ConnectionStateMachine {
    state: ConnectionState,
}

impl ConnectionStateMachine {
    pub fn new() -> Self {
        Self {
            state: ConnectionState::Disconnected,
        }
    }

    pub fn state(&self) -> ConnectionState {
        self.state
    }

    /// Attempt a state transition. Returns Err if the transition is invalid.
    pub fn transition(&mut self, next: ConnectionState) -> Result<ConnectionState, String> {
        if self.state.can_transition_to(next) {
            let prev = self.state;
            self.state = next;
            log::info!("Connection: {prev} → {next}");
            Ok(next)
        } else {
            let msg = format!("Invalid transition: {} → {}", self.state, next);
            log::warn!("{msg}");
            Err(msg)
        }
    }

    /// Force transition (used only for error recovery).
    pub fn force_transition(&mut self, next: ConnectionState) {
        let prev = self.state;
        self.state = next;
        log::warn!("Connection FORCED: {prev} → {next}");
    }
}

impl Default for ConnectionStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state() {
        let sm = ConnectionStateMachine::new();
        assert_eq!(sm.state(), ConnectionState::Disconnected);
    }

    #[test]
    fn valid_normal_flow() {
        let mut sm = ConnectionStateMachine::new();
        assert!(sm.transition(ConnectionState::Connecting).is_ok());
        assert!(sm.transition(ConnectionState::Authenticating).is_ok());
        assert!(sm.transition(ConnectionState::Connected).is_ok());
        assert!(sm.transition(ConnectionState::Disconnected).is_ok());
    }

    #[test]
    fn valid_reconnect_flow() {
        let mut sm = ConnectionStateMachine::new();
        sm.force_transition(ConnectionState::Connected);
        assert!(sm.transition(ConnectionState::Reconnecting).is_ok());
        assert!(sm.transition(ConnectionState::Authenticating).is_ok());
        assert!(sm.transition(ConnectionState::Connected).is_ok());
    }

    #[test]
    fn invalid_transition() {
        let mut sm = ConnectionStateMachine::new();
        assert!(sm.transition(ConnectionState::Connected).is_err());
        assert_eq!(sm.state(), ConnectionState::Disconnected);
    }

    #[test]
    fn error_recovery() {
        let mut sm = ConnectionStateMachine::new();
        sm.force_transition(ConnectionState::Error);
        assert!(sm.transition(ConnectionState::Disconnected).is_ok());
        assert!(sm.transition(ConnectionState::Connecting).is_ok());
    }
}
