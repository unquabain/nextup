pub mod bincode;
pub mod postgres;

use crate::error::Error;

#[derive(Debug)]
pub enum DataSource {
    Bincode(bincode::Bincode),
    Postgres(postgres::DSPostgres),
}

impl DataSource {
    pub async fn load(&mut self) -> Result<Vec<String>, Error> {
        match self {
            DataSource::Bincode(ds) => ds.load(),
            DataSource::Postgres(ds) => ds.load().await,
        }
    }
    pub async fn save(&mut self, data: Vec<String>) -> Result<(), Error> {
        match self {
            DataSource::Bincode(ds) => ds.save(data),
            DataSource::Postgres(ds) => ds.save(data).await,
        }
    }
    pub async fn nuke(&mut self) -> Result<(), Error> {
        match self {
            DataSource::Bincode(ds) => ds.nuke(),
            DataSource::Postgres(ds) => ds.nuke().await,
        }
    }
    pub async fn list_lists(&mut self) -> Result<Vec<String>, Error> {
        match self {
            DataSource::Bincode(_) => Ok(vec![]),
            DataSource::Postgres(ds) => ds.list_lists().await,
        }
    }
    pub async fn all_first_tasks(&mut self) -> Result<Vec<(String,String)>, Error> {
        match self {
            DataSource::Bincode(_) => Ok(vec![]),
            DataSource::Postgres(ds) => ds.all_first_tasks().await,
        }
    }
}
