use std::sync::{Arc, RwLock};

use rocket::{
    State,
    fs::FileServer,
    response::status::NotFound,
    serde::json::Json,
    tokio::sync::mpsc::{self, UnboundedSender},
};
use serde::{Deserialize, Serialize};

#[macro_use]
extern crate rocket;

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Category {
    Kitchen,
    Toiletries,
    Pharmacy,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Store {
    BigBox,
    Grocery,
    Convenience,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ListItem {
    needed: bool,
    label: String,
    category: Category,
    stores: Vec<Store>,
}

type SharedList = RwLock<Vec<ListItem>>;

type Clients = Arc<RwLock<Vec<UnboundedSender<String>>>>;

#[get("/")]
fn get_items(list: &State<SharedList>) -> Json<Vec<ListItem>> {
    let items = list.read().expect("lock poisoned");
    Json(items.clone())
}

#[patch("/<label>")]
fn toggle_item(
    list: &State<SharedList>,
    label: &str,
) -> Result<Json<Vec<ListItem>>, NotFound<String>> {
    let mut items = list.write().expect("lock poisoned");

    if let Some(existing) = items.iter_mut().find(|i| i.label == label) {
        existing.needed = !existing.needed;
        Ok(Json(items.clone()))
    } else {
        Err(NotFound("Oh no".into()))
    }
}

#[get("/subscribe")]
fn subscribe_channel(
    ws: rocket_ws::WebSocket,
    clients: &State<Clients>,
) -> rocket_ws::Channel<'static> {
    use rocket::futures::{SinkExt, StreamExt};

    let clients = clients.inner().clone();

    ws.channel(move |stream| {
        Box::pin(async move {
            let (mut outgoing, mut incoming) = stream.split();
            let (tx, mut rx) = mpsc::unbounded_channel::<String>();

            // Register this client
            {
                let mut locked = clients.write().unwrap();
                locked.push(tx);
            }

            // Task for sending messages from server → this client
            let send_task = rocket::tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    let _ = outgoing.send(rocket_ws::Message::Text(msg)).await;
                }
            });

            // Read loop for messages from client → broadcast
            while let Some(Ok(message)) = incoming.next().await {
                if let rocket_ws::Message::Text(text) = message {
                    // Broadcast to all
                    let clients_guard = clients.read().unwrap();
                    for sender in clients_guard.iter() {
                        let _ = sender.send(text.clone());
                    }
                }
            }

            // Clean up: stop sending and prune closed connections
            send_task.abort();
            {
                let mut clients_guard = clients.write().unwrap();
                clients_guard.retain(|s| !s.is_closed());
            }

            Ok(())
        })
    })
}
#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(RwLock::new(vec![
            ListItem {
                needed: true,
                label: "Peanut Butter".to_string(),
                category: Category::Kitchen,
                stores: vec![Store::BigBox, Store::Grocery],
            },
            ListItem {
                needed: true,
                label: "Omeperazole".to_string(),
                category: Category::Pharmacy,
                stores: vec![Store::BigBox, Store::Grocery],
            },
        ]))
        .manage(Arc::new(RwLock::new(Vec::<UnboundedSender<String>>::new())))
        .mount("/items", routes![get_items, toggle_item])
        .mount("/ws", routes![subscribe_channel])
        .mount("/", FileServer::from("static"))
}
