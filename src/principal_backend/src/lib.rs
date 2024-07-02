use ic_cdk::{api::call::ManualReply, init, post_upgrade, pre_upgrade, query, storage, update};

use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::caller;
use std::cell::RefCell; //Allows for mutable access to the data even though it is stored in a static context.
use std::collections::{BTreeMap, BTreeSet}; //These are sorted collections used for storing data and user sets,
                                            //use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
                                            //use ic_stable_structures::storable::Bound;
                                            //use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};
                                            //use std::borrow::Cow;

type Users = BTreeSet<Principal>;
type Store = BTreeMap<String, Vec<u8>>;

thread_local! {
    //This ensures that each thread has its own instance of these variables, which is useful in a concurrent environment.
    static USERS : RefCell<Users>=RefCell::default();
    static STORE : RefCell<Store>=RefCell::default();

}

//Called when the canister is first deployed. It adds the caller (the one who initializes the canister) to the USERS set.
#[init]
fn init() {
    USERS.with(|users| users.borrow_mut().insert(ic_cdk::api::caller()));
}

//This function updates the state of the canister and requires the is_user function to authorize the caller.
#[update]
fn store(path: String, contents: Vec<u8>) {
    STORE.with(|store| store.borrow_mut().insert(path, contents));
}

//Retrieves the data for the given path
//manual_reply = true: This indicates that the function will manually handle the reply to the query, using the ManualReply type. This provides more control over how the response is constructed and sent back to the caller.
#[query(manual_reply = true)]
fn retrieve(path: String) -> ManualReply<Vec<u8>> {
    STORE.with(|store| match store.borrow().get(&path) {
        Some(content) => ManualReply::one(content),
        //f the path does not exist, it panics (causes the canister to throw an error).
        None => panic!("Path {} not found.", path),
    })
}

//Allows an authorized user to add a new user by their Principal.
#[update]
fn add_user(principal: Principal) {
    USERS.with(|users| users.borrow_mut().insert(principal));
}

#[pre_upgrade]
fn pre_upgrade() {
    USERS.with(|users| storage::stable_save((users,)).unwrap());
}

#[post_upgrade]
fn post_upgrade() {
    let (old_users,): (BTreeSet<Principal>,) = storage::stable_restore().unwrap();
    USERS.with(|users| *users.borrow_mut() = old_users);
}

ic_cdk::export_candid!();
