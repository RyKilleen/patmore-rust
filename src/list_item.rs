use serde::{Deserialize, Serialize};
use std::fs::{self, create_dir_all};
use std::io::Error;

use log::info;

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

const DEFAULT_DATA_FILE: &str = "data/defaults.toml";
const TENANT_DATA_FOLDER: &str = "data/tenant/";

fn get_default_file() -> Result<String, Error> {
    let data = fs::read_to_string(DEFAULT_DATA_FILE)?;
    Ok(data)
}

pub fn get_tenant_list(tenant: &str) -> Result<Vec<ListItem>, Error> {
    let data = get_or_create_tenant_file(tenant).expect("Whoops");
    let parsed: ItemList = toml::from_str(&data).expect("Shucks");

    Ok(parsed.items)
}
struct TenantPath {
    file: String,
    directory: String,
}

fn get_tenant_folder_pathes(tenant_name: &str) -> TenantPath {
    let directory_path = format!("{}{}/", TENANT_DATA_FOLDER, tenant_name);
    let file_path = format!("{}{}", directory_path, "items.toml");

    TenantPath {
        file: file_path,
        directory: directory_path,
    }
}

fn create_new_tenant_file(tenant: &str) -> Result<(String, TenantPath), Error> {
    let data = get_default_file()?;
    let tenant_pathes = get_tenant_folder_pathes(tenant);
    create_dir_all(&tenant_pathes.directory)?;

    fs::write(&tenant_pathes.file, &data)?;
    info!("Created new file: {tenant} ");
    Ok((data, tenant_pathes))
}

fn get_tenant_file(name: &str) -> Result<String, Error> {
    let tenant_pathes = get_tenant_folder_pathes(name);

    let data = fs::read_to_string(tenant_pathes.file)?;
    Ok(data)
}

pub fn get_or_create_tenant_file(name: &str) -> Result<String, Error> {
    let tenant_file = get_tenant_file(name);
    if tenant_file.is_ok() {
        info!("Getting existing tenant");
        let file_data = tenant_file.unwrap();
        Ok(file_data)
    } else {
        info!("Creating new  tenant");
        println!("{:?}", tenant_file.err());
        let (data, _paths) = create_new_tenant_file(name).unwrap();
        let file_data = data;
        Ok(file_data)
    }
}

pub fn save_list_to_file(tenant: &str, items: Vec<ListItem>) -> std::io::Result<()> {
    let item_list = ItemList { items };

    let tenant_pathes = get_tenant_folder_pathes(tenant);
    let data = toml::to_string(&item_list).unwrap();
    let file_path = tenant_pathes.file;

    fs::write(&file_path, data)?;
    info!("Wrote to file: {file_path}");
    Ok(())
}

pub struct Noisy<F: FnMut()> {
    closure: F,
}

impl<F: FnMut()> Noisy<F> {
    pub fn new(closure: F) -> Self {
        Noisy { closure }
    }
}

impl<F: FnMut()> Drop for Noisy<F> {
    fn drop(&mut self) {
        (self.closure)()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_get_defaults() {
        let default_file = get_default_file();
        assert!(default_file.is_ok());
        let res = get_default_file();

        assert!(res.is_ok());
    }

    #[test]
    fn can_create_a_new_tenant() {
        let res = create_new_tenant_file("test/goobs");

        assert!(res.is_ok());

        let _noisy = Noisy::new(|| std::fs::remove_dir_all("data/tenant/test/").unwrap());
    }

    #[test]
    fn can_get_or_create_a_new_tenant() {
        let res1 = get_or_create_tenant_file("test/goobs");
        let res2 = get_or_create_tenant_file("test/goobs");

        assert!(res1.is_ok());
        assert!(res2.is_ok());
        let _noisy = Noisy::new(|| std::fs::remove_dir_all("data/tenant/test/").unwrap());
    }
}
