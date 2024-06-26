use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::caller;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};
use std::borrow::Cow;
use std::cell::RefCell;

type Memory = ic_stable_structures::memory_manager::VirtualMemory<DefaultMemoryImpl>;
const MAX_VALUE_SIZE: u32 = 5000;

#[derive(CandidType, Deserialize, Clone)]
struct Item {
    name: String,
    description: String,
    owner: Principal,
    new_owner: Option<Principal>,
    highest_bid: u64,
    is_active: bool,
}

#[derive(CandidType, Deserialize, Clone)]
struct Bid {
    bidder: Principal,
    amount: u64,
}

#[derive(CandidType, Deserialize, Clone)]
struct BidList(Vec<Bid>);

impl Storable for Item {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_VALUE_SIZE,
        is_fixed_size: false,
    };
}

impl Storable for BidList {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(&self.0).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        BidList(Decode!(bytes.as_ref(), Vec<Bid>).unwrap())
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_VALUE_SIZE,
        is_fixed_size: false,
    };
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    static ITEMS: RefCell<StableBTreeMap<u64, Item, Memory>> = RefCell::new(StableBTreeMap::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
    ));
    static BIDS: RefCell<StableBTreeMap<u64, BidList, Memory>> = RefCell::new(StableBTreeMap::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));
}

#[ic_cdk::update]
fn list_item(id: u64, name: String, description: String) -> Result<(), String> {
    let owner = caller();
    let item = Item {
        name,
        description,
        owner,
        new_owner: None,
        highest_bid: 0,
        is_active: true,
    };
    ITEMS.with(|items| {
        items.borrow_mut().insert(id, item);
    });
    Ok(())
}

#[ic_cdk::update]
fn bid_on_item(id: u64, amount: u64) -> Result<(), String> {
    let bidder = caller();
    ITEMS.with(|items| {
        let mut items = items.borrow_mut();
        if let Some(mut item) = items.get(&id) {
            if !item.is_active {
                return Err("Item is no longer active.".to_string());
            }
            if amount <= item.highest_bid {
                return Err("Bid amount must be higher than the current highest bid.".to_string());
            }
            item.highest_bid = amount;
            item.new_owner = Some(bidder.clone());
            items.insert(id, item);

            BIDS.with(|bids| {
                let mut bids = bids.borrow_mut();
                if let Some(bid_list) = bids.get(&id) {
                    let mut bid_list = bid_list.clone();
                    bid_list.0.push(Bid { bidder, amount });
                    bids.insert(id, bid_list);
                } else {
                    bids.insert(id, BidList(vec![Bid { bidder, amount }]));
                }
            });
            Ok(())
        } else {
            Err("Item not found.".to_string())
        }
    })
}

#[ic_cdk::update]
fn update_listing(id: u64, new_name: String, new_description: String) -> Result<(), String> {
    let caller = caller();
    ITEMS.with(|items| {
        let mut items = items.borrow_mut();
        if let Some(mut item) = items.get(&id) {
            if item.owner != caller {
                return Err("Only the owner can update the listing.".to_string());
            }
            item.name = new_name;
            item.description = new_description;
            items.insert(id, item);
            Ok(())
        } else {
            Err("Item not found.".to_string())
        }
    })
}

#[ic_cdk::update]
fn stop_listing(id: u64) -> Result<(), String> {
    let caller = caller();
    ITEMS.with(|items| {
        let mut items = items.borrow_mut();
        if let Some(mut item) = items.get(&id) {
            if item.owner != caller {
                return Err("Only the owner can stop the listing.".to_string());
            }
            item.is_active = false;
            items.insert(id, item);
            Ok(())
        } else {
            Err("Item not found.".to_string())
        }
    })
}

#[ic_cdk::query]
fn get_item(id: u64) -> Option<Item> {
    ITEMS.with(|items| items.borrow().get(&id).map(|item| item.clone()))
}

#[ic_cdk::query]
fn get_items() -> Vec<Item> {
    ITEMS.with(|items| {
        items
            .borrow()
            .iter()
            .map(|(_, item)| item.clone())
            .collect()
    })
}

#[ic_cdk::query]
fn get_items_count() -> u64 {
    ITEMS.with(|items| items.borrow().len())
}

#[ic_cdk::query]
fn get_highest_sold_item() -> Option<Item> {
    ITEMS.with(|items| {
        items
            .borrow()
            .iter()
            .max_by_key(|(_, item)| item.highest_bid)
            .map(|(_, item)| item.clone())
    })
}

#[ic_cdk::query]
fn get_most_bidded_item() -> Option<Item> {
    BIDS.with(|bids| {
        let bids = bids.borrow();
        let most_bidded = bids.iter().max_by_key(|(_, bid_list)| bid_list.0.len());
        most_bidded.and_then(|(id, _)| {
            ITEMS.with(|items| items.borrow().get(&id).map(|item| item.clone()))
        })
    })
}
