use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

#[macro_use]
extern crate rocket;

#[derive(Serialize, Deserialize, Debug)]
enum Category {
    Kitchen,
    Toiletries,
    Pharmacy,
}

#[derive(Serialize, Deserialize, Debug)]
enum Store {
    BigBox,
    Grocery,
    Convenience,
}

#[derive(Serialize, Deserialize, Debug)]
struct ListItem {
    needed: bool,
    label: String,
    category: Category,
    stores: Vec<Store>,
}

#[get("/")]
fn index() -> Json<Vec<ListItem>> {
    let hmm = vec![ListItem {
        needed: true,
        label: "Peanut Butter".to_string(),
        category: Category::Kitchen,
        stores: vec![Store::BigBox, Store::Grocery],
    }];

    Json(hmm)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
