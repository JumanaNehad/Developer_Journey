use candid::{CandidType, Deserialize};
use ic_cdk::{init, query, update};
use std::collections::HashMap;

type Asset = Vec<u8>;

#[derive(CandidType, Deserialize)]
struct Store {
    assets: HashMap<String, Asset>,
}

static mut STORE: Option<Store> = None;

#[init]
fn init() {
    unsafe { STORE = Some(Store { assets: HashMap::new() }) };
}

#[update]
fn upload_asset(name: String, content: Vec<u8>) {
    let store = unsafe { STORE.as_mut().unwrap() };
    store.assets.insert(name, content);
}

#[query]
fn get_asset(name: String) -> Option<&'static Vec<u8>> {
    let store = unsafe { STORE.as_ref().unwrap() };
    store.assets.get(&name)
}

#[query]
fn list_assets() -> Vec<String> {
    let store = unsafe { STORE.as_ref().unwrap() };
    store.assets.keys().cloned().collect()
}

ic_cdk::export_candid!();
