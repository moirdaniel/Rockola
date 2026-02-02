use core_events::AppEvent;
use tauri::{AppHandle, Emitter};
use crate::event_sink::TauriEventSink;


pub struct TauriEventSink {
    app: AppHandle,
}

impl TauriEventSink {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl core_scan::EventSink for TauriEventSink {
    fn emit(&self, event: AppEvent) {
        let _ = self.app.emit("app_event", event);
    }
}
