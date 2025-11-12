use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use rocket::futures::{SinkExt, StreamExt};
use rocket::http::Status;
use rocket::{State, fs::FileServer, fs::NamedFile, tokio::sync::mpsc};
use rocket_ws::{Channel, Message, WebSocket};
use serde::{Deserialize, Serialize};

mod list_item;
use list_item::{ListItem, get_tenant_list, save_list_to_file};

#[macro_use]
extern crate rocket;

type SharedList = Arc<RwLock<Vec<ListItem>>>;
type Clients = Arc<Mutex<Vec<mpsc::UnboundedSender<String>>>>;

// Each tenant has its own list + connected clients
#[derive(Default)]
struct TenantState {
    list: SharedList,
    clients: Clients,
}

// All tenants share this global registry
type TenantMap = Arc<Mutex<HashMap<String, Arc<TenantState>>>>;

// === HTTP Routes ===

#[get("/tenant/<_tenant>")]
async fn serve_tenant_html(_tenant: &str) -> Result<NamedFile, std::io::Error> {
    NamedFile::open("static/tenant.html").await
}

#[get("/health")]
fn health_check() -> Status {
    Status::Ok
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ClientMessage {
    Toggle { label: String },
    Ping,
}

#[derive(Serialize)]
#[serde(tag = "type", content = "data", rename_all = "lowercase")]
enum ServerMessage {
    Init(Vec<ListItem>),
    Update(Vec<ListItem>),
    Pong,
}

#[get("/<tenant>/updates")]
fn updates(ws: WebSocket, tenant: String, tenants: &State<TenantMap>) -> Channel<'static> {
    let tenants = tenants.inner().clone();

    ws.channel(move |stream| {
        Box::pin(async move {
            // Ensure tenant exists
            let tenant_state = {
                let mut tenants = tenants.lock().unwrap();
                tenants
                    .entry(tenant.clone())
                    .or_insert_with(|| {
                        Arc::new(TenantState {
                            list: Arc::new(RwLock::new(
                                get_tenant_list(&tenant).expect("Couldn't get tenant list"),
                            )),
                            clients: Arc::new(Mutex::new(Vec::new())),
                        })
                    })
                    .clone()
            };

            let (mut outgoing, mut incoming) = stream.split();
            let (tx, mut rx) = mpsc::unbounded_channel::<String>();

            // Register client
            {
                let mut clients = tenant_state.clients.lock().unwrap();
                clients.push(tx.clone());
            }

            // ✅ Send current list state immediately as an "init" message
            {
                let current_state = tenant_state.list.read().unwrap().clone();
                let payload = serde_json::to_string(&ServerMessage::Init(current_state)).unwrap();
                let _ = tx.send(payload);
            }

            // Task: send queued messages to this client's WebSocket
            let send_task = rocket::tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    if outgoing.send(Message::Text(msg)).await.is_err() {
                        break;
                    }
                }
            });

            // Handle incoming client messages
            while let Some(Ok(msg)) = incoming.next().await {
                if let Message::Text(text) = msg {
                    match serde_json::from_str::<ClientMessage>(&text) {
                        Ok(ClientMessage::Toggle { label }) => {
                            let updated = {
                                let mut items = tenant_state.list.write().unwrap();
                                if let Some(item) = items.iter_mut().find(|i| i.label == label) {
                                    item.needed = !item.needed;
                                }
                                items.clone()
                            };

                            save_list_to_file(&tenant, updated.clone())
                                .expect("Couldn't save file");

                            let payload =
                                serde_json::to_string(&ServerMessage::Update(updated)).unwrap();

                            // Broadcast update to all clients for this tenant
                            let snapshot = {
                                let guard = tenant_state.clients.lock().unwrap();
                                guard.clone()
                            };
                            for client in snapshot {
                                let _ = client.send(payload.clone());
                            }
                        }
                        Ok(ClientMessage::Ping) => {
                            let payload = serde_json::to_string(&ServerMessage::Pong).unwrap();
                            let _ = tx.send(payload);
                        }
                        Err(_) => eprintln!("Invalid message: {}", text),
                    }
                }
            }

            // Clean up disconnected clients
            {
                let mut clients = tenant_state.clients.lock().unwrap();
                clients.retain(|c| !c.is_closed());
            }

            send_task.abort();
            Ok(())
        })
    })
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(Arc::new(Mutex::new(
            HashMap::<String, Arc<TenantState>>::new(),
        )))
        .mount("/", FileServer::from("static"))
        .mount("/", routes![serve_tenant_html, health_check])
        .mount("/ws", routes![updates])
}
