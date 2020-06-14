#[macro_use]
extern crate serde_derive;

pub mod configuration;
pub mod directmethod;
pub mod iothub;
pub mod query;
pub mod twin;

pub use iothub::IoTHubService;
pub use configuration::modulescontent::{ModulesContent, ModulesContentBuilder};
use iothub::API_VERSION;

use query::QueryBuilder;
use twin::TwinManager;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
