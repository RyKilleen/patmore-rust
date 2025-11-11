use std::sync::{Arc, Mutex, RwLock};

use rocket::futures::{SinkExt, StreamExt};
use rocket::http::Status;
use rocket::{State, fs::FileServer, serde::json::Json, tokio::sync::mpsc};
use rocket_ws::{Channel, Message, WebSocket};
use serde::Deserialize;

mod list_item;

use list_item::ListItem;

use crate::list_item::{get_default_list, save_list_to_file};

#[macro_use]
extern crate rocket;

type SharedList = Arc<RwLock<Vec<ListItem>>>;
type Clients = Arc<Mutex<Vec<mpsc::UnboundedSender<String>>>>;

// === HTTP Routes ===

#[get("/items")]
fn get_items(list: &State<SharedList>) -> Json<Vec<ListItem>> {
    Json(list.read().unwrap().clone())
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

#[get("/updates")]
fn updates(ws: WebSocket, clients: &State<Clients>, list: &State<SharedList>) -> Channel<'static> {
    // Clone Arcs for `'static` closure
    let clients = clients.inner().clone();
    let list = list.inner().clone();

    ws.channel(move |stream| {
        Box::pin(async move {
            let (mut outgoing, mut incoming) = stream.split();
            let (tx, mut rx) = mpsc::unbounded_channel::<String>();

            // Register this client
            {
                let mut locked = clients.lock().unwrap();
                locked.push(tx.clone());
            }

            // Task: forward messages from `rx` → WebSocket
            let send_task = rocket::tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    if outgoing.send(Message::Text(msg)).await.is_err() {
                        break;
                    }
                }
            });

            // Read loop: handle client messages
            while let Some(Ok(msg)) = incoming.next().await {
                if let Message::Text(text) = msg {
                    match serde_json::from_str::<ClientMessage>(&text) {
                        Ok(ClientMessage::Toggle { label }) => {
                            // Update shared state
                            let updated = {
                                let mut items = list.write().unwrap();
                                if let Some(item) = items.iter_mut().find(|i| i.label == label) {
                                    item.needed = !item.needed;
                                }
                                items.clone()
                            };

                            let _ = save_list_to_file(updated.clone()).expect("Couldn't save file");

                            // Broadcast new list to all clients
                            let payload = serde_json::to_string(&updated).unwrap();
                            let snapshot = {
                                let guard = clients.lock().unwrap();
                                guard.clone()
                            };
                            for client in snapshot {
                                let _ = client.send(payload.clone());
                            }
                        }
                        Ok(ClientMessage::Ping) => {
                            // Respond to this client via its own sender
                            let _ = tx.send("pong".to_string());
                        }
                        Err(_) => {
                            eprintln!("Invalid message: {}", text);
                        }
                    }
                }
            }

            // Clean up closed clients
            {
                let mut locked = clients.lock().unwrap();
                locked.retain(|c| !c.is_closed());
            }

            send_task.abort();
            Ok(())
        })
    })
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(Arc::new(RwLock::new(
            get_default_list().expect("Couldn't get default list"),
        )))
        .manage(Arc::new(Mutex::new(
            Vec::<mpsc::UnboundedSender<String>>::new(),
        )))
        .mount("/", routes![get_items, health_check])
        .mount("/ws", routes![updates])
        .mount("/", FileServer::from("static"))
}
