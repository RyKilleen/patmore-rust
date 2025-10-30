use std::sync::RwLock;

use rocket::{State, fs::FileServer, serde::json::Json};
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

#[get("/")]
fn get_items(list: &State<SharedList>) -> Json<Vec<ListItem>> {
    let items = list.read().expect("lock poisoned");

    Json(items.clone())
}

#[post("/", data = "<item>")]
fn add_item(list: &State<SharedList>, item: Json<ListItem>) -> &'static str {
    let mut items = list.write().expect("lock poisoned");
    items.push(item.into_inner());
    "Item added"
}

#[patch("/<label>")]
fn toggle_item(list: &State<SharedList>, label: &str) -> &'static str {
    let mut items = list.write().expect("lock poisoned");

    if let Some(existing) = items.iter_mut().find(|i| i.label == label) {
        existing.needed = !existing.needed;
        "Item toggled"
    } else {
        "Item not found"
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(RwLock::new(vec![ListItem {
            needed: true,
            label: "Peanut Butter".to_string(),
            category: Category::Kitchen,
            stores: vec![Store::BigBox, Store::Grocery],
        }]))
        .mount("/items", routes![get_items, add_item, toggle_item])
        .mount("/", FileServer::from("static"))
}
