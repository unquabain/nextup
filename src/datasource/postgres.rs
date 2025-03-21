use crate::error::Error;

use postgres::{Client, Statement};
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;
use postgres::types::Type;
use crate::datasource::DataSource;
use log::*;

pub struct DSPostgres {
    client: Client,
    list: String,
    fetch_query: Statement,
    clear_query: Statement,
    save_query: Statement,
    list_list_query: Statement,
}

impl std::fmt::Debug for DSPostgres {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "DSPostgres {{ list: {} }}", self.list)
    }
}

impl DSPostgres {
    pub fn new(url: &str, list: &str) -> Result<Self, Error> {
        let connector = TlsConnector::new().map_err(Error::from_error)?;
        let connector = MakeTlsConnector::new(connector);
        let mut client = Client::connect(url, connector)
            .map_err(Error::from_error)?;
        let fetch_query = match client.prepare_typed("SELECT task FROM nextup WHERE list = $1 ORDER BY rank", &[Type::TEXT]) {
            Ok(q) => q,
            Err(e) => {
                match e.code() {
                    Some(&postgres::error::SqlState::UNDEFINED_TABLE) => {
                        client.execute(
                            "CREATE TABLE IF NOT EXISTS nextup (rank INT, task TEXT, list VARCHAR(255), UNIQUE (rank, list) )",
                            &[],
                        ) .map_err(Error::from_error)?;
                        client.prepare_typed("SELECT task FROM nextup WHERE list = $1 ORDER BY rank", &[Type::TEXT])
                            .map_err(Error::from_error)?
                    },
                    _ => {
                        error!("Error preparing fetch query: {}", e);
                        return Err(Error::from_error(e));
                    }
                }
            }
        };
        let clear_query = client.prepare_typed("DELETE FROM nextup WHERE list = $1", &[Type::TEXT])
            .map_err(Error::from_error)?;
        let save_query = client.prepare_typed("INSERT INTO nextup (rank, task, list) VALUES ($1, $2, $3) ON CONFLICT (list, rank) DO UPDATE SET task = $2", &[Type::INT4, Type::TEXT, Type::TEXT])
            .map_err(Error::from_error)?;
        let list_list_query = client.prepare("SELECT DISTINCT list FROM nextup")
            .map_err(Error::from_error)?;
            
        Ok(DSPostgres{
            client,
            list: list.to_string(),
            fetch_query,
            clear_query,
            save_query,
            list_list_query,
        })
    }
}

impl DataSource for DSPostgres {
    fn load(&mut self) -> Result<Vec<String>, Error> {
        let rows = self.client.query(&self.fetch_query, &[&self.list])
            .map_err(Error::from_error)?;
       
        let mut strings = Vec::new();
        for row in rows {
            strings.push(row.get(0));
        }
        Ok(strings)
    }
    fn save(&mut self, data: Vec<String>) -> Result<(), Error> {
        let mut txn = self.client.transaction().map_err(Error::from_error)?;
        txn.execute(&self.clear_query, &[&self.list])
            .map_err(Error::from_error)?;
        for (i, task) in data.iter().enumerate() {
            txn.execute(&self.save_query, &[&(i as i32), task, &self.list])
                .map_err(Error::from_error)?;
        }
        txn.commit().map_err(Error::from_error)?;
        Ok(())
    }
    fn nuke(&mut self) -> Result<(), Error> {
        self.client.execute(&self.clear_query, &[&self.list])
            .map_err(Error::from_error)?;
        Ok(())
    }
    fn list_lists(&mut self) -> Result<Vec<String>, Error> {
        let rows = self.client.query(&self.list_list_query, &[])
            .map_err(Error::from_error)?;
        let mut lists = Vec::new();
        for row in rows {
            lists.push(row.get(0));
        }
        Ok(lists)
    }
}


