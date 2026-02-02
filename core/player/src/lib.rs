//! Sistema de reproducción para Rockola
//! 
//! Maneja:
//! - Cola de reproducción
//! - Estado del reproductor
//! - Historial de reproducción
//! - Configuración de reproducción

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use core_events::{AppEvent, PlayerState, PlaybackEvent, QueueDelta};

#[derive(Debug, Clone)]
pub struct QueueItem {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub media_type: String, // "audio" | "video"
    pub file_path: String,
    pub duration_ms: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct PlayerConfig {
    pub volume: f32,
    pub playback_rate: f32,
    pub repeat_mode: RepeatMode,
    pub shuffle: bool,
    pub auto_next_delay: Duration, // tiempo de espera antes de siguiente
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            volume: 1.0,
            playback_rate: 1.0,
            repeat_mode: RepeatMode::None,
            shuffle: false,
            auto_next_delay: Duration::from_secs(2),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RepeatMode {
    None,
    One,
    All,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerStateEnum {
    Idle,
    Playing,
    Paused,
    Buffering,
    Stopped,
    Error,
}

pub struct Player {
    queue: Arc<Mutex<VecDeque<QueueItem>>>,
    history: Arc<Mutex<Vec<QueueItem>>>,
    current_item: Arc<Mutex<Option<QueueItem>>>,
    current_position: Arc<Mutex<i64>>,
    state: Arc<Mutex<PlayerStateEnum>>,
    config: Arc<Mutex<PlayerConfig>>,
    event_callback: Option<Arc<dyn Fn(AppEvent) + Send + Sync>>,
}

impl Player {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            history: Arc::new(Mutex::new(Vec::new())),
            current_item: Arc::new(Mutex::new(None)),
            current_position: Arc::new(Mutex::new(0)),
            state: Arc::new(Mutex::new(PlayerStateEnum::Idle)),
            config: Arc::new(Mutex::new(PlayerConfig::default())),
            event_callback: None,
        }
    }

    pub fn with_event_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(AppEvent) + Send + Sync + 'static,
    {
        self.event_callback = Some(Arc::new(callback));
        self
    }

    fn emit_event(&self, event: AppEvent) {
        if let Some(ref callback) = self.event_callback {
            callback(event);
        }
    }

    // Gestión de la cola
    pub fn enqueue(&self, item: QueueItem) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(item.clone());
        self.emit_event(AppEvent::QueueDelta(QueueDelta::new("added".to_string(), vec![item.id])));
    }

    pub fn enqueue_multiple(&self, items: Vec<QueueItem>) {
        let mut queue = self.queue.lock().unwrap();
        for item in &items {
            queue.push_back(item.clone());
        }
        let ids: Vec<i64> = items.iter().map(|item| item.id).collect();
        self.emit_event(AppEvent::QueueDelta(QueueDelta::new("added".to_string(), ids)));
    }

    pub fn insert_at(&self, index: usize, item: QueueItem) {
        let mut queue = self.queue.lock().unwrap();
        if index <= queue.len() {
            queue.insert(index, item.clone());
            self.emit_event(AppEvent::QueueDelta(QueueDelta::new("inserted".to_string(), vec![item.id])));
        }
    }

    pub fn remove_from_queue(&self, index: usize) -> Option<QueueItem> {
        let mut queue = self.queue.lock().unwrap();
        if index < queue.len() {
            let item = queue.remove(index);
            if let Some(ref item) = item {
                self.emit_event(AppEvent::QueueDelta(QueueDelta::new("removed".to_string(), vec![item.id])));
            }
            item
        } else {
            None
        }
    }

    pub fn clear_queue(&self) {
        let mut queue = self.queue.lock().unwrap();
        let ids: Vec<i64> = queue.iter().map(|item| item.id).collect();
        queue.clear();
        self.emit_event(AppEvent::QueueDelta(QueueDelta::new("cleared".to_string(), ids)));
    }

    pub fn move_queue_item(&self, from_index: usize, to_index: usize) {
        let mut queue = self.queue.lock().unwrap();
        if from_index < queue.len() && to_index < queue.len() && from_index != to_index {
            let item = queue.remove(from_index).unwrap();
            queue.insert(to_index, item);
            self.emit_event(AppEvent::QueueDelta(QueueDelta::new("moved".to_string(), vec![])));
        }
    }

    pub fn shuffle_queue(&self) {
        use rand::seq::SliceRandom;
        let mut queue = self.queue.lock().unwrap();
        let mut items: Vec<QueueItem> = queue.drain(..).collect();
        items.shuffle(&mut rand::thread_rng());
        *queue = items.into();
        self.emit_event(AppEvent::QueueDelta(QueueDelta::new("shuffled".to_string(), vec![])));
    }

    // Reproducción
    pub fn play(&self) -> Result<(), String> {
        let current_item = self.current_item.lock().unwrap().clone();
        if current_item.is_none() {
            // Intentar reproducir el siguiente de la cola
            if let Err(e) = self.play_next() {
                return Err(e);
            }
        }

        let mut state = self.state.lock().unwrap();
        *state = PlayerStateEnum::Playing;
        
        let current_item = self.current_item.lock().unwrap().clone();
        let position = *self.current_position.lock().unwrap();
        
        self.emit_event(AppEvent::PlayerState(PlayerState {
            state: "playing".to_string(),
            item_id: current_item.as_ref().map(|item| item.id),
            position_ms: position,
            duration_ms: current_item.as_ref().and_then(|item| item.duration_ms),
            volume: self.config.lock().unwrap().volume,
            playback_rate: self.config.lock().unwrap().playback_rate,
            repeat_mode: match self.config.lock().unwrap().repeat_mode {
                RepeatMode::None => "none".to_string(),
                RepeatMode::One => "one".to_string(),
                RepeatMode::All => "all".to_string(),
            },
            shuffle: self.config.lock().unwrap().shuffle,
        }));

        self.emit_event(AppEvent::PlaybackEvent(PlaybackEvent {
            event_type: "resumed".to_string(),
            item_id: current_item.as_ref().map(|item| item.id),
            position_ms: Some(position),
            duration_ms: current_item.as_ref().and_then(|item| item.duration_ms),
            reason: None,
            timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
        }));

        Ok(())
    }

    pub fn pause(&self) {
        let mut state = self.state.lock().unwrap();
        if *state == PlayerStateEnum::Playing {
            *state = PlayerStateEnum::Paused;
            
            let current_item = self.current_item.lock().unwrap().clone();
            let position = *self.current_position.lock().unwrap();
            
            self.emit_event(AppEvent::PlayerState(PlayerState {
                state: "paused".to_string(),
                item_id: current_item.as_ref().map(|item| item.id),
                position_ms: position,
                duration_ms: current_item.as_ref().and_then(|item| item.duration_ms),
                volume: self.config.lock().unwrap().volume,
                playback_rate: self.config.lock().unwrap().playback_rate,
                repeat_mode: match self.config.lock().unwrap().repeat_mode {
                    RepeatMode::None => "none".to_string(),
                    RepeatMode::One => "one".to_string(),
                    RepeatMode::All => "all".to_string(),
                },
                shuffle: self.config.lock().unwrap().shuffle,
            }));

            self.emit_event(AppEvent::PlaybackEvent(PlaybackEvent {
                event_type: "paused".to_string(),
                item_id: current_item.as_ref().map(|item| item.id),
                position_ms: Some(position),
                duration_ms: current_item.as_ref().and_then(|item| item.duration_ms),
                reason: None,
                timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
            }));
        }
    }

    pub fn stop(&self) {
        let mut state = self.state.lock().unwrap();
        *state = PlayerStateEnum::Stopped;
        
        let current_item = self.current_item.lock().unwrap().clone();
        
        self.emit_event(AppEvent::PlayerState(PlayerState {
            state: "stopped".to_string(),
            item_id: current_item.as_ref().map(|item| item.id),
            position_ms: 0,
            duration_ms: current_item.as_ref().and_then(|item| item.duration_ms),
            volume: self.config.lock().unwrap().volume,
            playback_rate: self.config.lock().unwrap().playback_rate,
            repeat_mode: match self.config.lock().unwrap().repeat_mode {
                RepeatMode::None => "none".to_string(),
                RepeatMode::One => "one".to_string(),
                RepeatMode::All => "all".to_string(),
            },
            shuffle: self.config.lock().unwrap().shuffle,
        }));

        self.emit_event(AppEvent::PlaybackEvent(PlaybackEvent {
            event_type: "stopped".to_string(),
            item_id: current_item.as_ref().map(|item| item.id),
            position_ms: Some(0),
            duration_ms: current_item.as_ref().and_then(|item| item.duration_ms),
            reason: None,
            timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
        }));
    }

    pub fn play_next(&self) -> Result<(), String> {
        let mut queue = self.queue.lock().unwrap();
        if queue.is_empty() {
            if matches!(self.config.lock().unwrap().repeat_mode, RepeatMode::All) {
                // Si está en repeat all y la cola está vacía, reiniciar la cola desde el historial
                let history = self.history.lock().unwrap();
                if !history.is_empty() {
                    *queue = history.iter().cloned().collect();
                } else {
                    return Err("La cola está vacía y no hay elementos en el historial".to_string());
                }
            } else {
                return Err("La cola está vacía".to_string());
            }
        }

        let next_item = queue.pop_front().unwrap();
        
        // Agregar al historial
        self.history.lock().unwrap().push(next_item.clone());
        
        // Actualizar estado actual
        *self.current_item.lock().unwrap() = Some(next_item.clone());
        *self.current_position.lock().unwrap() = 0;
        
        // Cambiar estado a reproducción
        let mut state = self.state.lock().unwrap();
        *state = PlayerStateEnum::Playing;
        
        // Emitir eventos
        self.emit_event(AppEvent::PlayerState(PlayerState {
            state: "playing".to_string(),
            item_id: Some(next_item.id),
            position_ms: 0,
            duration_ms: next_item.duration_ms,
            volume: self.config.lock().unwrap().volume,
            playback_rate: self.config.lock().unwrap().playback_rate,
            repeat_mode: match self.config.lock().unwrap().repeat_mode {
                RepeatMode::None => "none".to_string(),
                RepeatMode::One => "one".to_string(),
                RepeatMode::All => "all".to_string(),
            },
            shuffle: self.config.lock().unwrap().shuffle,
        }));

        self.emit_event(AppEvent::PlaybackEvent(PlaybackEvent {
            event_type: "started".to_string(),
            item_id: Some(next_item.id),
            position_ms: Some(0),
            duration_ms: next_item.duration_ms,
            reason: Some("play_next".to_string()),
            timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
        }));

        Ok(())
    }

    pub fn play_previous(&self) -> Result<(), String> {
        let mut history = self.history.lock().unwrap();
        if history.len() < 2 {
            return Err("No hay elementos anteriores en el historial".to_string());
        }

        // Sacar el elemento actual del historial
        history.pop(); // Remover el actual
        
        // Obtener el anterior
        if let Some(prev_item) = history.pop() {
            // Volver a poner el actual en la cola (al principio)
            let current_item = self.current_item.lock().unwrap().clone();
            if let Some(current) = current_item {
                let mut queue = self.queue.lock().unwrap();
                queue.push_front(current);
            }
            
            // Establecer el anterior como actual
            *self.current_item.lock().unwrap() = Some(prev_item.clone());
            *self.current_position.lock().unwrap() = 0;
            
            let mut state = self.state.lock().unwrap();
            *state = PlayerStateEnum::Playing;
            
            // Emitir eventos
            self.emit_event(AppEvent::PlayerState(PlayerState {
                state: "playing".to_string(),
                item_id: Some(prev_item.id),
                position_ms: 0,
                duration_ms: prev_item.duration_ms,
                volume: self.config.lock().unwrap().volume,
                playback_rate: self.config.lock().unwrap().playback_rate,
                repeat_mode: match self.config.lock().unwrap().repeat_mode {
                    RepeatMode::None => "none".to_string(),
                    RepeatMode::One => "one".to_string(),
                    RepeatMode::All => "all".to_string(),
                },
                shuffle: self.config.lock().unwrap().shuffle,
            }));

            self.emit_event(AppEvent::PlaybackEvent(PlaybackEvent {
                event_type: "started".to_string(),
                item_id: Some(prev_item.id),
                position_ms: Some(0),
                duration_ms: prev_item.duration_ms,
                reason: Some("play_previous".to_string()),
                timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
            }));

            Ok(())
        } else {
            Err("No hay elementos anteriores en el historial".to_string())
        }
    }

    pub fn seek_to(&self, position_ms: i64) {
        *self.current_position.lock().unwrap() = position_ms;
        
        let current_item = self.current_item.lock().unwrap().clone();
        
        self.emit_event(AppEvent::PlayerState(PlayerState {
            state: "playing".to_string(), // Mantener el estado actual
            item_id: current_item.as_ref().map(|item| item.id),
            position_ms,
            duration_ms: current_item.as_ref().and_then(|item| item.duration_ms),
            volume: self.config.lock().unwrap().volume,
            playback_rate: self.config.lock().unwrap().playback_rate,
            repeat_mode: match self.config.lock().unwrap().repeat_mode {
                RepeatMode::None => "none".to_string(),
                RepeatMode::One => "one".to_string(),
                RepeatMode::All => "all".to_string(),
            },
            shuffle: self.config.lock().unwrap().shuffle,
        }));

        self.emit_event(AppEvent::PlaybackEvent(PlaybackEvent {
            event_type: "seeked".to_string(),
            item_id: current_item.as_ref().map(|item| item.id),
            position_ms: Some(position_ms),
            duration_ms: current_item.as_ref().and_then(|item| item.duration_ms),
            reason: None,
            timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
        }));
    }

    // Control de volumen y configuración
    pub fn set_volume(&self, volume: f32) {
        let mut config = self.config.lock().unwrap();
        config.volume = volume.max(0.0).min(1.0);
    }

    pub fn set_playback_rate(&self, rate: f32) {
        let mut config = self.config.lock().unwrap();
        config.playback_rate = rate.max(0.1).min(4.0);
    }

    pub fn set_repeat_mode(&self, mode: RepeatMode) {
        let mut config = self.config.lock().unwrap();
        config.repeat_mode = mode;
    }

    pub fn set_shuffle(&self, shuffle: bool) {
        let mut config = self.config.lock().unwrap();
        config.shuffle = shuffle;
    }

    // Getters
    pub fn get_queue(&self) -> Vec<QueueItem> {
        self.queue.lock().unwrap().iter().cloned().collect()
    }

    pub fn get_queue_length(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    pub fn get_current_item(&self) -> Option<QueueItem> {
        self.current_item.lock().unwrap().clone()
    }

    pub fn get_current_position(&self) -> i64 {
        *self.current_position.lock().unwrap()
    }

    pub fn get_state(&self) -> PlayerStateEnum {
        self.state.lock().unwrap().clone()
    }

    pub fn get_config(&self) -> PlayerConfig {
        self.config.lock().unwrap().clone()
    }

    pub fn get_history(&self) -> Vec<QueueItem> {
        self.history.lock().unwrap().iter().cloned().collect()
    }

    // Fin de reproducción de un elemento
    pub fn on_item_finished(&self) -> Result<(), String> {
        let current_item = self.current_item.lock().unwrap().clone();
        
        self.emit_event(AppEvent::PlaybackEvent(PlaybackEvent {
            event_type: "ended".to_string(),
            item_id: current_item.as_ref().map(|item| item.id),
            position_ms: current_item.as_ref().and_then(|item| item.duration_ms),
            duration_ms: current_item.as_ref().and_then(|item| item.duration_ms),
            reason: Some("completed".to_string()),
            timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
        }));

        match self.config.lock().unwrap().repeat_mode {
            RepeatMode::One => {
                // Repetir el mismo elemento
                *self.current_position.lock().unwrap() = 0;
                
                self.emit_event(AppEvent::PlayerState(PlayerState {
                    state: "playing".to_string(),
                    item_id: current_item.as_ref().map(|item| item.id),
                    position_ms: 0,
                    duration_ms: current_item.as_ref().and_then(|item| item.duration_ms),
                    volume: self.config.lock().unwrap().volume,
                    playback_rate: self.config.lock().unwrap().playback_rate,
                    repeat_mode: "one".to_string(),
                    shuffle: self.config.lock().unwrap().shuffle,
                }));

                Ok(())
            }
            _ => {
                // Pasar al siguiente elemento
                std::thread::sleep(self.config.lock().unwrap().auto_next_delay);
                self.play_next()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_creation() {
        let player = Player::new();
        assert_eq!(player.get_queue_length(), 0);
        assert_eq!(player.get_state(), PlayerStateEnum::Idle);
    }

    #[test]
    fn test_enqueue_and_play() {
        let player = Player::new();
        let item = QueueItem {
            id: 1,
            title: "Test Song".to_string(),
            artist: "Test Artist".to_string(),
            media_type: "audio".to_string(),
            file_path: "/path/to/test.mp3".to_string(),
            duration_ms: Some(180000),
        };

        player.enqueue(item);
        assert_eq!(player.get_queue_length(), 1);

        // Intentar reproducir debería fallar porque no hay siguiente en la cola
        let result = player.play_next();
        assert!(result.is_ok());
    }
}