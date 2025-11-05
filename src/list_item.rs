use std::fs;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Category {
    Kitchen,
    Toiletries,
    Pharmacy,
    Pantry,
    Household,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Aisle {
    Condiments,
    Cereal,
    Pharmacy,
    Baking,
    Spices,
    Beverages,
    Produce,
    Snacks,
    Refrigerated,
    Deli,
    Dairy,
    Meat,
    Household,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Store {
    BigBox,
    Grocery,
    Convenience,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListItem {
    pub needed: bool,
    pub label: String,
    pub aisle: Vec<Aisle>,
    pub category: Category,
    pub stores: Vec<Store>,
}

#[derive(Debug, Deserialize)]
pub struct ItemList {
    pub items: Vec<ListItem>,
}

pub fn init_list() -> Vec<ListItem> {
    let data = fs::read_to_string("data/items.toml").expect("Failed to read items.toml");
    let parsed: ItemList = toml::from_str(&data).expect("Failed to parse TOML");
    parsed.items
}
