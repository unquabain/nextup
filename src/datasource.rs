pub mod bincode;

use crate::error::Error;

pub trait DataSource {
    fn load(&self) -> Result<Vec<String>, Error>;
    fn save(&self, data: Vec<String>) -> Result<(), Error>;
}
