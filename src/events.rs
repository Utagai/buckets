use std::fmt::Display;
use std::time::SystemTime;

use chrono::{DateTime, Local};
use ratatui::style::Color;

pub enum EventSource {
    Controller,
    Actuator,
    Filler,
}

impl Display for EventSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Controller => write!(f, "Controller"),
            Self::Actuator => write!(f, "Actuator"),
            Self::Filler => write!(f, "Filler"),
        }
    }
}

impl EventSource {
    pub fn color(&self) -> Color {
        match self {
            Self::Controller => Color::Yellow,
            Self::Actuator => Color::Green,
            Self::Filler => Color::Cyan,
        }
    }
}

pub struct Event {
    pub timestamp: DateTime<Local>,
    pub source: EventSource,
    pub message: String,
}

pub struct Events {
    events: Vec<Event>,
}

impl Events {
    pub fn new() -> Self {
        Events { events: Vec::new() }
    }

    pub fn add(&mut self, source: EventSource, message: String) {
        self.events.push(Event {
            timestamp: Local::now(),
            source,
            message,
        });
    }

    pub fn get_all(&self) -> &Vec<Event> {
        &self.events
    }
}
