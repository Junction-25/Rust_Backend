use actix_web::{web, HttpRequest, HttpResponse, Error};
use actix_web_actors::ws;
use actix::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};

// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WSMessage {
    #[serde(rename = "subscribe")]
    Subscribe {
        contact_id: i32,
        subscription_types: Vec<SubscriptionType>,
    },
    #[serde(rename = "unsubscribe")]
    Unsubscribe {
        contact_id: i32,
        subscription_types: Vec<SubscriptionType>,
    },
    #[serde(rename = "property_update")]
    PropertyUpdate {
        property_id: i32,
        update_type: PropertyUpdateType,
        details: PropertyUpdateDetails,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "new_recommendation")]
    NewRecommendation {
        contact_id: i32,
        property_id: i32,
        score: f64,
        reason: String,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "market_alert")]
    MarketAlert {
        location: String,
        property_type: String,
        alert_type: MarketAlertType,
        message: String,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "price_prediction")]
    PricePrediction {
        property_id: i32,
        current_price: f64,
        predicted_price: f64,
        confidence: f64,
        time_horizon: String,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "heartbeat")]
    Heartbeat {
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "error")]
    Error {
        message: String,
        code: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubscriptionType {
    #[serde(rename = "new_properties")]
    NewProperties,
    #[serde(rename = "price_changes")]
    PriceChanges,
    #[serde(rename = "market_updates")]
    MarketUpdates,
    #[serde(rename = "recommendations")]
    Recommendations,
    #[serde(rename = "price_predictions")]
    PricePredictions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyUpdateType {
    #[serde(rename = "new_listing")]
    NewListing,
    #[serde(rename = "price_change")]
    PriceChange,
    #[serde(rename = "status_change")]
    StatusChange,
    #[serde(rename = "description_update")]
    DescriptionUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyUpdateDetails {
    pub old_value: Option<serde_json::Value>,
    pub new_value: serde_json::Value,
    pub change_percentage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketAlertType {
    #[serde(rename = "hot_market")]
    HotMarket,
    #[serde(rename = "price_drop")]
    PriceDrop,
    #[serde(rename = "new_inventory")]
    NewInventory,
    #[serde(rename = "trend_change")]
    TrendChange,
}

// WebSocket session
pub struct WSSession {
    id: String,
    contact_id: Option<i32>,
    subscriptions: Vec<SubscriptionType>,
    last_heartbeat: DateTime<Utc>,
    addr: Option<Addr<WebSocketManager>>,
}

impl WSSession {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            contact_id: None,
            subscriptions: Vec::new(),
            last_heartbeat: Utc::now(),
            addr: None,
        }
    }
}

impl Actor for WSSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("WebSocket session {} started", self.id);
        
        // Send welcome message
        let welcome = WSMessage::Heartbeat {
            timestamp: Utc::now(),
        };
        ctx.text(serde_json::to_string(&welcome).unwrap_or_default());
        
        // Start heartbeat
        self.heartbeat(ctx);
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        log::info!("WebSocket session {} stopped", self.id);
        
        // Notify manager of disconnection
        if let Some(addr) = &self.addr {
            addr.do_send(Disconnect { id: self.id.clone() });
        }
    }
}

impl WSSession {
    fn heartbeat(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(std::time::Duration::from_secs(30), |act, ctx| {
            // Check if client is still alive
            if Utc::now().timestamp() - act.last_heartbeat.timestamp() > 60 {
                log::info!("WebSocket client {} timed out", act.id);
                ctx.stop();
                return;
            }

            // Send heartbeat
            let heartbeat = WSMessage::Heartbeat {
                timestamp: Utc::now(),
            };
            ctx.text(serde_json::to_string(&heartbeat).unwrap_or_default());
        });
    }

    fn handle_message(&mut self, msg: &str, ctx: &mut ws::WebsocketContext<Self>) {
        match serde_json::from_str::<WSMessage>(msg) {
            Ok(WSMessage::Subscribe { contact_id, subscription_types }) => {
                self.contact_id = Some(contact_id);
                self.subscriptions = subscription_types.clone();
                
                // Register with manager
                if let Some(addr) = &self.addr {
                    addr.do_send(RegisterClient {
                        id: self.id.clone(),
                        contact_id,
                        subscriptions: subscription_types,
                    });
                }
                
                log::info!("Client {} subscribed for contact {} with {:?}", 
                    self.id, contact_id, self.subscriptions);
            },
            Ok(WSMessage::Unsubscribe { contact_id: _, subscription_types }) => {
                self.subscriptions.retain(|s| !subscription_types.contains(s));
                
                log::info!("Client {} unsubscribed from {:?}", self.id, subscription_types);
            },
            Ok(WSMessage::Heartbeat { .. }) => {
                self.last_heartbeat = Utc::now();
            },
            Err(e) => {
                log::warn!("Invalid WebSocket message from {}: {}", self.id, e);
                let error = WSMessage::Error {
                    message: "Invalid message format".to_string(),
                    code: "INVALID_MESSAGE".to_string(),
                };
                ctx.text(serde_json::to_string(&error).unwrap_or_default());
            },
            _ => {
                log::warn!("Unhandled message type from {}", self.id);
            }
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WSSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.last_heartbeat = Utc::now();
                ctx.pong(&msg);
            },
            Ok(ws::Message::Pong(_)) => {
                self.last_heartbeat = Utc::now();
            },
            Ok(ws::Message::Text(text)) => {
                self.handle_message(&text, ctx);
            },
            Ok(ws::Message::Binary(_)) => {
                log::warn!("Binary messages not supported");
            },
            Ok(ws::Message::Close(reason)) => {
                log::info!("WebSocket client {} closed: {:?}", self.id, reason);
                ctx.stop();
            },
            Err(e) => {
                log::error!("WebSocket error for {}: {}", self.id, e);
                ctx.stop();
            },
            _ => (),
        }
    }
}

// WebSocket Manager Messages
#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterClient {
    pub id: String,
    pub contact_id: i32,
    pub subscriptions: Vec<SubscriptionType>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastMessage {
    pub message: WSMessage,
    pub target_contact_id: Option<i32>,
    pub subscription_type: SubscriptionType,
}

// WebSocket Manager
pub struct WebSocketManager {
    clients: HashMap<String, ClientInfo>,
    contact_clients: HashMap<i32, Vec<String>>,
}

#[derive(Clone)]
struct ClientInfo {
    addr: Recipient<WSClientMessage>,
    contact_id: i32,
    subscriptions: Vec<SubscriptionType>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct WSClientMessage {
    pub message: WSMessage,
}

impl Handler<WSClientMessage> for WSSession {
    type Result = ();

    fn handle(&mut self, msg: WSClientMessage, ctx: &mut Self::Context) {
        let message_text = serde_json::to_string(&msg.message).unwrap_or_default();
        ctx.text(message_text);
    }
}

impl Actor for WebSocketManager {
    type Context = Context<Self>;
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self {
            clients: HashMap::new(),
            contact_clients: HashMap::new(),
        }
    }
}

impl Handler<RegisterClient> for WebSocketManager {
    type Result = ();

    fn handle(&mut self, msg: RegisterClient, _: &mut Self::Context) {
        log::info!("Registering WebSocket client {} for contact {}", msg.id, msg.contact_id);
        
        // Remove from old contact if exists
        if let Some(client_info) = self.clients.get(&msg.id) {
            if let Some(clients) = self.contact_clients.get_mut(&client_info.contact_id) {
                clients.retain(|id| id != &msg.id);
            }
        }
        
        // Note: In a real implementation, you would store the client address here
        // For now, we just track the mapping
        
        // Add to new contact
        self.contact_clients
            .entry(msg.contact_id)
            .or_insert_with(Vec::new)
            .push(msg.id.clone());
    }
}

impl Handler<Disconnect> for WebSocketManager {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) {
        log::info!("Disconnecting WebSocket client {}", msg.id);
        
        if let Some(client_info) = self.clients.remove(&msg.id) {
            if let Some(clients) = self.contact_clients.get_mut(&client_info.contact_id) {
                clients.retain(|id| id != &msg.id);
                if clients.is_empty() {
                    self.contact_clients.remove(&client_info.contact_id);
                }
            }
        }
    }
}

impl Handler<BroadcastMessage> for WebSocketManager {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, _: &mut Self::Context) {
        let target_clients = if let Some(contact_id) = msg.target_contact_id {
            // Send to specific contact
            self.contact_clients.get(&contact_id).cloned().unwrap_or_default()
        } else {
            // Send to all clients with matching subscription
            self.clients
                .keys()
                .cloned()
                .collect::<Vec<_>>()
        };

        // Log the broadcast for demo purposes
        log::info!("Broadcasting {:?} to {} clients", 
            std::mem::discriminant(&msg.message), target_clients.len());
        
        // In a real implementation, you would send to actual WebSocket connections
        // For now, we just log the activity
    }
}

// Real-time notification service
#[derive(Clone)]
pub struct RealtimeNotificationService {
    ws_manager: Addr<WebSocketManager>,
}

impl RealtimeNotificationService {
    pub fn new(ws_manager: Addr<WebSocketManager>) -> Self {
        Self { ws_manager }
    }

    /// Notify about new property recommendations
    pub async fn notify_new_recommendation(
        &self,
        contact_id: i32,
        property_id: i32,
        score: f64,
        reason: String,
    ) {
        let message = WSMessage::NewRecommendation {
            contact_id,
            property_id,
            score,
            reason,
            timestamp: Utc::now(),
        };

        self.ws_manager.do_send(BroadcastMessage {
            message,
            target_contact_id: Some(contact_id),
            subscription_type: SubscriptionType::Recommendations,
        });
    }

    /// Notify about property updates
    pub async fn notify_property_update(
        &self,
        property_id: i32,
        update_type: PropertyUpdateType,
        old_value: Option<serde_json::Value>,
        new_value: serde_json::Value,
        change_percentage: Option<f64>,
    ) {
        let message = WSMessage::PropertyUpdate {
            property_id,
            update_type: update_type.clone(),
            details: PropertyUpdateDetails {
                old_value,
                new_value,
                change_percentage,
            },
            timestamp: Utc::now(),
        };

        let subscription_type = match update_type {
            PropertyUpdateType::PriceChange => SubscriptionType::PriceChanges,
            PropertyUpdateType::NewListing => SubscriptionType::NewProperties,
            _ => SubscriptionType::MarketUpdates,
        };

        self.ws_manager.do_send(BroadcastMessage {
            message,
            target_contact_id: None,
            subscription_type,
        });
    }

    /// Notify about market alerts
    pub async fn notify_market_alert(
        &self,
        location: String,
        property_type: String,
        alert_type: MarketAlertType,
        message: String,
    ) {
        let ws_message = WSMessage::MarketAlert {
            location,
            property_type,
            alert_type,
            message,
            timestamp: Utc::now(),
        };

        self.ws_manager.do_send(BroadcastMessage {
            message: ws_message,
            target_contact_id: None,
            subscription_type: SubscriptionType::MarketUpdates,
        });
    }

    /// Notify about price predictions
    pub async fn notify_price_prediction(
        &self,
        property_id: i32,
        current_price: f64,
        predicted_price: f64,
        confidence: f64,
        time_horizon: String,
    ) {
        let message = WSMessage::PricePrediction {
            property_id,
            current_price,
            predicted_price,
            confidence,
            time_horizon,
            timestamp: Utc::now(),
        };

        self.ws_manager.do_send(BroadcastMessage {
            message,
            target_contact_id: None,
            subscription_type: SubscriptionType::PricePredictions,
        });
    }

    /// Start background task for periodic notifications
    pub fn start_background_notifications(&self) {
        let ws_manager = self.ws_manager.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes
            
            loop {
                interval.tick().await;
                
                // Simulate market updates
                let market_alert = WSMessage::MarketAlert {
                    location: "Algiers".to_string(),
                    property_type: "apartment".to_string(),
                    alert_type: MarketAlertType::TrendChange,
                    message: "Market activity increasing in Algiers apartment sector".to_string(),
                    timestamp: Utc::now(),
                };

                ws_manager.do_send(BroadcastMessage {
                    message: market_alert,
                    target_contact_id: None,
                    subscription_type: SubscriptionType::MarketUpdates,
                });
            }
        });
    }
}

// WebSocket endpoint
pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    ws_manager: web::Data<Addr<WebSocketManager>>,
) -> Result<HttpResponse, Error> {
    let mut session = WSSession::new();
    session.addr = Some(ws_manager.get_ref().clone());
    
    log::info!("New WebSocket connection from {:?}", req.connection_info().peer_addr());
    
    ws::start(session, &req, stream)
}

pub fn configure_websocket_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/ws", web::get().to(websocket_handler));
}
