use std::net::TcpListener;
use std::thread::{spawn, JoinHandle};
use tungstenite::{accept, Message, WebSocket};
use tungstenite::protocol::WebSocket as WS;
use std::net::TcpStream;
use serde_json::Value;
use local_ip_address::local_ip;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum GameAction {
    MoveLeft,
    MoveRight,
    Shoot,
    StopMoving,
}

pub struct ArduinoServer {
    ip: String,
    port: u16,
    action_queue: Arc<Mutex<VecDeque<GameAction>>>,
}

impl ArduinoServer {
    pub fn new(port: u16) -> Self {
        let local_ip = local_ip().unwrap();
        ArduinoServer {
            ip: local_ip.to_string(),
            port,
            action_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn get_action_queue(&self) -> Arc<Mutex<VecDeque<GameAction>>> {
        self.action_queue.clone()
    }

    pub fn start(&self) -> JoinHandle<()> {
        let ip = self.ip.clone();
        let port = self.port;
        let action_queue = self.action_queue.clone();
        
        spawn(move || {
            let bind_address = format!("{}:{}", ip, port);
            
            let server = TcpListener::bind(&bind_address).unwrap();
            println!("WebSocket server listening on {}", bind_address);
            println!("Connect your Arduino to: {}", ip);

            // Set non-blocking mode for the server
            server.set_nonblocking(true).unwrap();
            
            let mut last_status_print = Instant::now();
            let status_interval = Duration::from_secs(2); // Print every 2 seconds
            
            loop {
                // Print status periodically
                if last_status_print.elapsed() >= status_interval {
                    println!("Server running - listening for connections on {}", bind_address);
                    last_status_print = Instant::now();
                }
                
                // Check for incoming connections
                match server.accept() {
                    Ok((stream, _)) => {
                        let queue = action_queue.clone();
                        spawn(move || {
                            Self::handle_client(stream, queue);
                        });
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No connection available, sleep briefly and continue
                        std::thread::sleep(Duration::from_millis(100));
                    }
                    Err(e) => {
                        println!("Error accepting connection: {}", e);
                    }
                }
            }
        })
    }

    fn handle_client(stream: TcpStream, action_queue: Arc<Mutex<VecDeque<GameAction>>>) {
        let mut websocket = accept(stream).unwrap();
        println!("New WebSocket connection established");

        loop {
            match websocket.read_message() {
                Ok(msg) => {
                    if msg.is_text() {
                        let text = msg.to_text().unwrap();
                        println!("Received: {}", text);
                        if let Ok(json) = serde_json::from_str::<Value>(text) {
                            if let Some(action) = json.get("action") {
                                if let Some(action_str) = action.as_str() {
                                    let game_action = match action_str {
                                        "peakleft" => {
                                            if let Some(value) = json.get("value") {
                                                println!("   Left movement value: {}", value);
                                                Some(GameAction::MoveLeft)
                                            } else {
                                                None
                                            }
                                        }
                                        "peakright" => {
                                            if let Some(value) = json.get("value") {
                                                println!("   Right movement value: {}", value);
                                                Some(GameAction::MoveRight)
                                            } else {
                                                None
                                            }
                                        }
                                        "peakshoot" => {
                                            if let Some(value) = json.get("value") {
                                                println!("   Shoot value: {}", value);
                                                Some(GameAction::Shoot)
                                            } else {
                                                None
                                            }
                                        }
                                        "stop" => {
                                            println!("   Stop movement");
                                            Some(GameAction::StopMoving)
                                        }
                                        _ => {
                                            println!("Unknown action: {}", action_str);
                                            None
                                        }
                                    };

                                    if let Some(action) = game_action {
                                        if let Ok(mut queue) = action_queue.lock() {
                                            queue.push_back(action);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("WebSocket error: {}", e);
                    break;
                }
            }
        }
        
        println!("Client disconnected");
    }
}

// #include <WiFiS3.h>
// #include <WebSocketsClient.h>
// #include <ArduinoJson.h>

// const char* ssid = "gooddogL";
// const char* password = "eatbadman";

// const char* websocket_server = "192.168.1.122";
// const int websocket_port = 3030;

// const int sensorPin = A0;

// WebSocketsClient webSocket;
// bool isConnected = false;

// void setup() {
//     Serial.begin(9600);
//     while (!Serial); 
//     delay(2000);     

//     Serial.println("Attempting to connect to WiFi...");
//     int status = WL_IDLE_STATUS;
//     int retries = 0;
//     while (status != WL_CONNECTED && retries < 10) {
//         status = WiFi.begin(ssid, password);
//         delay(1000);
//         retries++;
//         Serial.print("WiFi connection attempt ");
//         Serial.println(retries);
//     }

//     if (status != WL_CONNECTED) {
//         Serial.println("Failed to connect to WiFi.");
//         return;
//     }

//     Serial.println("Initializing WebSocket connection...");
//     webSocket.begin(websocket_server, websocket_port, "/");
//     webSocket.onEvent(webSocketEvent);
//     webSocket.setReconnectInterval(5000);  
// }

// // WebSocket event handler
// void webSocketEvent(WStype_t type, uint8_t * payload, size_t length) {
//     switch (type) {
//         case WStype_DISCONNECTED:
//             isConnected = false;
//             Serial.println("WebSocket disconnected.");
//             break;

//         case WStype_CONNECTED:
//             isConnected = true;
//             Serial.println("WebSocket connected.");
//             break;

//         case WStype_TEXT:
//             Serial.print("Received text: ");
//             Serial.write(payload, length);
//             Serial.println();
//             break;

//         case WStype_BIN:
//             Serial.println("Received binary data (not used).");
//             break;

//         case WStype_ERROR:
//             isConnected = false;
//             Serial.println("WebSocket error occurred.");
//             break;

//         case WStype_PING:
//             Serial.println("WebSocket ping received.");
//             break;

//         case WStype_PONG:
//             Serial.println("WebSocket pong received.");
//             break;

//         default:
//             Serial.println("Unknown WebSocket event.");
//             break;
//     }
// }

// void sendSensorData(int sensorValue) {
//     if (!isConnected) {
//         Serial.println("Not connected. Skipping data send.");
//         return;
//     }

//     StaticJsonDocument<200> doc;
//     doc["action"] = "left";          
//     doc["value"] = sensorValue;       

//     String jsonString;
//     serializeJson(doc, jsonString);

//     Serial.print("Sending JSON: ");
//     Serial.println(jsonString);

//     webSocket.sendTXT(jsonString);   
// }

// void loop() {
//     webSocket.loop();                     

//     int sensorValue = analogRead(sensorPin);
//     Serial.print("Sensor value: ");
//     Serial.println(sensorValue);

//     sendSensorData(sensorValue);
// }
