use candid::{CandidType, Decode, Encode, Deserialize, Principal};
use ic_cdk::{query, update};
use ic_stable_structures::{Storable, storable::Bound};
use serde::Serialize;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

const MAX_CHUNK_SIZE: usize = 2 * 1024 * 1024; // 2MB

#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct Chunk {
    pub chunk_id: u128,
    pub order: u32,
    pub content: Vec<u8>,
    pub owned_by: Principal,
    pub uploaded_at: u64,
}

impl Storable for Chunk {
    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(&self).unwrap())
    }

    const BOUND: Bound = Bound::Bounded { max_size: MAX_CHUNK_SIZE as u32 + 50, is_fixed_size: false };
}

#[derive(CandidType, Deserialize)]
pub struct ChunkArgs {
    pub order: u32,
    pub content: Vec<u8>,
}

impl From<(u128, ChunkArgs, Principal)> for Chunk {
    fn from((chunk_id, arg, owned_by): (u128, ChunkArgs, Principal)) -> Self {
        Self {
            chunk_id,
            order: arg.order,
            content: arg.content,
            owned_by,
            uploaded_at: ic_cdk::api::time(),
        }
    }
}

pub struct State {
    pub chunks: HashMap<u128, Chunk>,
    pub next_chunk_id: u128,
}

impl State {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            next_chunk_id: 0,
        }
    }

    pub fn generate_chunk_id(&mut self) -> u128 {
        let id = self.next_chunk_id;
        self.next_chunk_id += 1;
        id
    }
}

#[update]
pub fn upload_chunk(arg: ChunkArgs) -> u128 {
    let caller = ic_cdk::caller();
    // if caller == Principal::anonymous() {
    //     ic_cdk::trap("Anonymous Caller")
    // }
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let chunk_id = state.generate_chunk_id();
        let chunk = Chunk::from((chunk_id, arg, caller));
        state.chunks.insert(chunk_id, chunk);
        ic_cdk_timers::set_timer(Duration::from_secs(0), || ic_cdk::spawn(async { update_storage().await }));
        chunk_id
    })
}

async fn update_storage() {
    // Define the storage update logic here
}

#[query]
pub fn chunk_ids_check(ids: Vec<u128>) -> bool {
    chunk_ids_validity_check(&ids)
}

fn chunk_ids_validity_check(_ids: &[u128]) -> bool {
    // Implement your logic here
    true
}

#[query]
fn list_assets() -> Vec<String> {
    STATE.with(|state| {
        let state = state.borrow();
        state.chunks.values().map(|chunk| chunk.owned_by.to_text()).collect()
    })
}

#[query]
fn get_asset(name: String) -> Option<Vec<u8>> {
    ic_cdk::println!("Fetching asset with name: {}", name);
    STATE.with(|state| {
        let state = state.borrow();
        let mut asset_data: Vec<u8> = Vec::new();

        for chunk in state.chunks.values() {
            ic_cdk::println!("Checking chunk with owned_by: {}", chunk.owned_by.to_text());
            if chunk.owned_by.to_text() == name {
                ic_cdk::println!("Found chunk for asset: {}", name);
                asset_data.extend_from_slice(&chunk.content);
            }
        }

        if asset_data.is_empty() {
            None
        } else {
            Some(asset_data)
        }
    })
}


thread_local! {
    static STATE: Rc<RefCell<State>> = Rc::new(RefCell::new(State::new()));
}

ic_cdk::export_candid!();
