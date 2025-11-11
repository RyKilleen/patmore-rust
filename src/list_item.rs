use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Error;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemList {
    pub items: Vec<ListItem>,
}

const DATA_PATH: &str = "data/defaults.toml";

fn get_default_file() -> Result<String, Error> {
    let data = fs::read_to_string(DATA_PATH)?;
    Ok(data)
}

pub fn get_default_list() -> Result<Vec<ListItem>, Error> {
    let data = get_default_file().expect("Dang");
    let parsed: ItemList = toml::from_str(&data).expect("Couldn't Parse");
    Ok(parsed.items)
}

pub fn save_list_to_file(items: Vec<ListItem>) -> std::io::Result<()> {
    let item_list = ItemList { items };
    let data = toml::to_string(&item_list).unwrap();

    fs::write(DATA_PATH, data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_get_defaults() {
        let default_file = get_default_file();
        assert!(default_file.is_ok());
        let default_items = get_default_list();

        assert!(default_items.is_ok());
        assert!(default_items.unwrap().len() > 0);
    }
}
