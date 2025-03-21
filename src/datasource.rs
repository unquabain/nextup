pub mod bincode;
pub mod postgres;

use crate::error::Error;

pub trait DataSource : std::fmt::Debug {
    fn load(&mut self) -> Result<Vec<String>, Error>;
    fn save(&mut self, data: Vec<String>) -> Result<(), Error>;
    fn nuke(&mut self) -> Result<(), Error>;
    fn list_lists(&mut self) -> Result<Vec<String>, Error> {
        Ok(vec![])
    }
}
