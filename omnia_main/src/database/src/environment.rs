use std::collections::BTreeMap;
use std::cell::RefCell;

pub mod interface_types;
pub mod store_types;
pub mod utils;

use store_types as StoreTypes;

type EnvironmentUID = u32;

//  ENVIRONMENTS DATABASE
type EnvironmentStore = BTreeMap<EnvironmentUID, StoreTypes::EnvironmentInfo>;

thread_local! {
    static ENVIRONMENT_STORE: RefCell<EnvironmentStore> = RefCell::default();
}
