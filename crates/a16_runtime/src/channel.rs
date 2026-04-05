//! Bounded MPSC Channel
//!
//! Provides message passing between tasks.
//! Uses a generic ChannelMessage type for maximum flexibility.

use smol_str::SmolStr;
use std::collections::VecDeque;
use thiserror::Error;

/// Channel errors
#[derive(Debug, Error)]
pub enum ChannelError {
    #[error("Channel is closed")]
    Closed,

    #[error("Channel buffer is full (capacity: {capacity})")]
    Full { capacity: usize },

    #[error("Channel is empty")]
    Empty,
}

/// A message in a channel — supports both string and opaque value messages
#[derive(Debug, Clone)]
pub enum ChannelMessage {
    /// A text message
    Text(SmolStr),
    /// An opaque value ID (maps to VM Value or any u64 handle)
    ValueId(u64),
}

impl ChannelMessage {
    /// Get as text, if it is a text message
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ChannelMessage::Text(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Get as value ID, if it is a value message
    pub fn as_value_id(&self) -> Option<u64> {
        match self {
            ChannelMessage::ValueId(id) => Some(*id),
            _ => None,
        }
    }
}

impl From<SmolStr> for ChannelMessage {
    fn from(s: SmolStr) -> Self {
        ChannelMessage::Text(s)
    }
}

impl From<u64> for ChannelMessage {
    fn from(id: u64) -> Self {
        ChannelMessage::ValueId(id)
    }
}

/// A bounded MPSC channel for message passing
#[derive(Debug)]
pub struct Channel {
    /// Channel name
    pub name: SmolStr,
    /// Message buffer
    buffer: VecDeque<ChannelMessage>,
    /// Maximum capacity
    capacity: usize,
    /// Whether the channel is closed
    closed: bool,
    /// Total messages sent
    total_sent: u64,
    /// Total messages received
    total_received: u64,
}

impl Channel {
    /// Create a new bounded channel
    pub fn new(name: SmolStr, capacity: usize) -> Self {
        Self {
            name,
            buffer: VecDeque::with_capacity(capacity),
            capacity,
            closed: false,
            total_sent: 0,
            total_received: 0,
        }
    }

    /// Send a message into the channel
    pub fn send(&mut self, msg: ChannelMessage) -> Result<(), ChannelError> {
        if self.closed {
            return Err(ChannelError::Closed);
        }
        if self.buffer.len() >= self.capacity {
            return Err(ChannelError::Full { capacity: self.capacity });
        }
        self.buffer.push_back(msg);
        self.total_sent += 1;
        Ok(())
    }

    /// Convenience: send a text message
    pub fn send_text(&mut self, msg: SmolStr) -> Result<(), ChannelError> {
        self.send(ChannelMessage::Text(msg))
    }

    /// Try to receive a message from the channel
    pub fn recv(&mut self) -> Result<ChannelMessage, ChannelError> {
        if let Some(msg) = self.buffer.pop_front() {
            self.total_received += 1;
            Ok(msg)
        } else if self.closed {
            Err(ChannelError::Closed)
        } else {
            Err(ChannelError::Empty)
        }
    }

    /// Try to receive without blocking
    pub fn try_recv(&mut self) -> Option<ChannelMessage> {
        let msg = self.buffer.pop_front()?;
        self.total_received += 1;
        Some(msg)
    }

    /// Close the channel
    pub fn close(&mut self) {
        self.closed = true;
    }

    /// Check if the channel is closed
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// Number of messages in the buffer
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Whether the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Whether the buffer is full
    pub fn is_full(&self) -> bool {
        self.buffer.len() >= self.capacity
    }

    /// Channel capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Total messages sent
    pub fn total_sent(&self) -> u64 {
        self.total_sent
    }

    /// Total messages received
    pub fn total_received(&self) -> u64 {
        self.total_received
    }
}
