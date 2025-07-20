use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use serde_json::Value;
use local_ip_address::list_afinet_netifas;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::collections::{VecDeque, HashMap};
use uuid::Uuid;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Copy, Eq, Hash, Serialize, Deserialize)]
pub enum GameAction { MoveLeft, MoveRight, Shoot, Idle }

#[derive(Debug, Clone)]
pub struct ActionEvent {
    pub connection_id: String,
    pub action: GameAction,
    pub pressure: f64,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct SimultaneousActions {
    pub connection_id: String,
    pub movement: Option<GameAction>,
    pub shooting: bool,
    pub movement_pressure: f64,
    pub shoot_pressure: f64,
    pub timestamp: Instant,
}

#[derive(Deserialize)]
struct ActionRequest { 
    action: String, 
    pressure: Option<f64>,
}

#[derive(Clone, Default)]
struct ConnectionState {
    current_movement: Option<GameAction>,
    is_shooting: bool,
    movement_pressure: f64,
    shoot_pressure: f64,
}

pub struct ArduinoServer {
    ip: String,
    port: u16,
    action_queue: Arc<Mutex<VecDeque<ActionEvent>>>,
    simultaneous_queue: Arc<Mutex<VecDeque<SimultaneousActions>>>,
    pressure_threshold: Arc<Mutex<f64>>,
    connection_states: Arc<Mutex<HashMap<String, ConnectionState>>>,
}

impl ArduinoServer {
    pub fn new() -> Self {
        let ip = Self::get_wifi_ip().unwrap_or_else(|| "127.0.0.1".parse().unwrap());
        
        Self {
            ip: ip.to_string(),
            port: 3030,
            action_queue: Arc::new(Mutex::new(VecDeque::new())),
            simultaneous_queue: Arc::new(Mutex::new(VecDeque::new())),
            pressure_threshold: Arc::new(Mutex::new(500.0)),
            connection_states: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn set_pressure_threshold(&self, threshold: f64) {
        if let Ok(mut t) = self.pressure_threshold.lock() { *t = threshold; }
    }
    
    pub fn get_pressure_threshold(&self) -> f64 {
        self.pressure_threshold.lock().map(|t| *t).unwrap_or(500.0)
    }
    
    fn get_wifi_ip() -> Option<IpAddr> {
        let nics = list_afinet_netifas().ok()?;
        let patterns = ["wlan", "wifi", "Wi-Fi", "en0", "en1", "wlp"];
        
        patterns.iter()
            .find_map(|pattern| nics.iter()
                .find(|(name, ip)| name.to_lowercase().contains(&pattern.to_lowercase()) 
                    && ip.is_ipv4() && !ip.is_loopback())
                .map(|(_, ip)| *ip))
            .or_else(|| nics.iter()
                .find(|(_, ip)| ip.is_ipv4() && !ip.is_loopback())
                .map(|(_, ip)| *ip))
    }
    

    
    pub async fn start(&self) {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await.unwrap();
        println!("Server running on {}:{}", self.ip, self.port);
        
        loop {
            if let Ok((stream, _)) = listener.accept().await {
                let server = self.clone();
                tokio::spawn(async move { let _ = server.handle_connection(stream).await; });
            }
        }
    }
    
    async fn handle_connection(&self, stream: tokio::net::TcpStream) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let connection_id = format!("arduino-{}", Uuid::new_v4().simple());
        let ws_stream = accept_async(stream).await?;
        let (mut sender, mut receiver) = ws_stream.split();
        
        self.connection_states.lock().unwrap().insert(connection_id.clone(), ConnectionState::default());
        
        let _ = sender.send(Message::Text(serde_json::json!({
            "type": "connected",
            "connection_id": connection_id
        }).to_string().into())).await;
        
        while let Some(msg) = receiver.next().await {
            match msg? {
                Message::Text(text) => self.process_message(&connection_id, &text),
                Message::Close(_) => break,
                Message::Ping(ping) => { let _ = sender.send(Message::Pong(ping)).await; }
                _ => {}
            }
        }
        
        self.connection_states.lock().unwrap().remove(&connection_id);
        Ok(())
    }
    
    fn process_message(&self, connection_id: &str, text: &str) {
        let Ok(request) = serde_json::from_str::<ActionRequest>(text) else { return; };
        
        let pressure = request.pressure.unwrap_or(0.0);
        let threshold = self.get_pressure_threshold();
        let Ok(mut states) = self.connection_states.try_lock() else { return; };
        let state = states.entry(connection_id.to_string()).or_insert_with(ConnectionState::default);
        
        let (action, send_discrete) = match request.action.as_str() {
            "peakleft" if pressure > threshold => {
                state.current_movement = Some(GameAction::MoveLeft);
                state.movement_pressure = pressure;
                (GameAction::MoveLeft, true)
            }
            "peakright" if pressure > threshold => {
                state.current_movement = Some(GameAction::MoveRight);
                state.movement_pressure = pressure;
                (GameAction::MoveRight, true)
            }
            "peakshoot" if pressure > threshold => {
                state.is_shooting = true;
                state.shoot_pressure = pressure;
                (GameAction::Shoot, true)
            }
            "stop" => {
                *state = ConnectionState::default();
                (GameAction::Idle, true)
            }
            "stopmovement" => {
                state.current_movement = None;
                state.movement_pressure = 0.0;
                (GameAction::Idle, true)
            }
            "stopshooting" => {
                state.is_shooting = false;
                state.shoot_pressure = 0.0;
                (GameAction::Idle, false)
            }
            _ => return,
        };
        
        let simultaneous = SimultaneousActions {
            connection_id: connection_id.to_string(),
            movement: state.current_movement,
            shooting: state.is_shooting,
            movement_pressure: state.movement_pressure,
            shoot_pressure: state.shoot_pressure,
            timestamp: Instant::now(),
        };
        
        drop(states);
        
        if let Ok(mut queue) = self.simultaneous_queue.try_lock() {
            queue.push_back(simultaneous);
            if queue.len() > 50 { queue.pop_front(); }
        }
        
        if send_discrete {
            if let Ok(mut queue) = self.action_queue.try_lock() {
                queue.push_back(ActionEvent {
                    connection_id: connection_id.to_string(),
                    action,
                    pressure,
                    timestamp: Instant::now(),
                });
                if queue.len() > 50 { queue.pop_front(); }
            }
        }
    }
    
    pub fn get_action_queue(&self) -> Arc<Mutex<VecDeque<ActionEvent>>> {
        Arc::clone(&self.action_queue)
    }
    
    pub fn drain_actions(&self) -> Vec<ActionEvent> {
        self.action_queue.try_lock().map_or(Vec::new(), |mut q| q.drain(..).collect())
    }
    
    pub fn get_simultaneous_actions(&self) -> Vec<SimultaneousActions> {
        self.simultaneous_queue.try_lock().map_or(Vec::new(), |mut q| q.drain(..).collect())
    }
}

impl Clone for ArduinoServer {
    fn clone(&self) -> Self {
        Self {
            ip: self.ip.clone(),
            port: self.port,
            action_queue: Arc::clone(&self.action_queue),
            simultaneous_queue: Arc::clone(&self.simultaneous_queue),
            pressure_threshold: Arc::clone(&self.pressure_threshold),
            connection_states: Arc::clone(&self.connection_states),
        }
    }
}