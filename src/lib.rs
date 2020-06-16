#[macro_use]
extern crate serde_derive;

pub mod configuration;
pub mod directmethod;
pub mod error;
pub mod iothub;
pub mod query;
pub mod twin;

pub use iothub::IoTHubService;
pub use configuration::modulescontent::{EdgeModuleBuilder, ModulesContent, ModulesContentBuilder};
use iothub::API_VERSION;
