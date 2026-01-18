use crate::error::{Error,Result};

use tokio_postgres::{Config, Client, Statement};
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;
use tokio_postgres::types::Type;
use crate::secret::replace_secrets;
use log::trace;

pub struct DSPostgres {
    client: Client,
    list: String,
    fetch_query: Statement,
    clear_query: Statement,
    save_query: Statement,
    list_list_query: Statement,
    all_first_tasks_query: Statement,
}

impl std::fmt::Debug for DSPostgres {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "DSPostgres {{ list: {} }}", self.list)
    }
}

impl DSPostgres {
    async fn init_tables(client: &Client) -> Result<()> {
        client.execute(
            "CREATE TABLE IF NOT EXISTS nextup (rank INT, task TEXT, list VARCHAR(255), UNIQUE (rank, list) )",
            &[],
        ).await?;
        Ok(())
    }
    async fn prepare_fetch_query(client: &Client) -> Result<Statement> {
        let query = "SELECT task FROM nextup WHERE list = $1 ORDER BY rank";
        Ok(client.prepare_typed(query, &[Type::TEXT]).await?)
    }
    async fn prepare_clear_query(client: &Client) -> Result<Statement> {
        let query = "DELETE FROM nextup WHERE list = $1";
        let stmt = client.prepare_typed(query, &[Type::TEXT]).await?;
        Ok(stmt)
    }
    async fn prepare_save_query(client: &Client) -> Result<Statement> {
        let query = "INSERT INTO nextup (rank, task, list) VALUES ($1, $2, $3) ON CONFLICT (list, rank) DO UPDATE SET task = $2";
        let stmt = client.prepare_typed(query, &[Type::INT4, Type::TEXT, Type::TEXT]).await?;
        Ok(stmt)
    }
    async fn prepare_list_lists_query(client: &Client) -> Result<Statement> {
        let query = "SELECT DISTINCT list FROM nextup ORDER BY list";
        let stmt = client.prepare_typed(query, &[]).await?;
        Ok(stmt)
    }
    async fn prepare_all_first_tasks_query(client: &Client) -> Result<Statement> {
        let query = "SELECT list, task FROM nextup WHERE rank = 0 ORDER BY list";
        let stmt = client.prepare_typed(query, &[]).await?;
        Ok(stmt)
    }
    pub async fn new(url: &str, list: &str) -> Result<Self> {
        let connector = TlsConnector::new()?;
        let connector = MakeTlsConnector::new(connector);
        let mut surl = url.to_string();
        replace_secrets(&mut surl)?;
        let config: Config = surl.parse()?;
        let (client, connection) = config.connect(connector)
            .await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Postgres connection error: {}", e);
            }
        });
        trace!("{}:{}", file!(), line!());
        let fetch_query = match Self::prepare_fetch_query(&client).await {
            Ok(q) => q,
            Err(Error::DbError(e)) => {
                match e.code() {
                    Some(&tokio_postgres::error::SqlState::UNDEFINED_TABLE) => {
                        Self::init_tables(&client).await?;
                        Self::prepare_fetch_query(&client).await?
                    },
                    _ => return Err(Error::DbError(e)),
                }
            }
            Err(e) => {
                return Err(e);
            }
        };
        trace!("{}:{}", file!(), line!());
        let clear_query = Self::prepare_clear_query(&client).await?;
        trace!("{}:{}", file!(), line!());
        let save_query =  Self::prepare_save_query(&client).await?;
        trace!("{}:{}", file!(), line!());
        let list_list_query = Self::prepare_list_lists_query(&client).await?;
        trace!("{}:{}", file!(), line!());
        let all_first_tasks_query = Self::prepare_all_first_tasks_query(&client).await?;
        trace!("{}:{}", file!(), line!());

        Ok(DSPostgres{
            client,
            list: list.to_string(),
            fetch_query,
            clear_query,
            save_query,
            list_list_query,
            all_first_tasks_query,
        })
    }
    pub async fn load(&mut self) -> Result<Vec<String>> {
        trace!("Loading list '{}' from Postgres", self.list);
        let rows = self.client.query(&self.fetch_query, &[&self.list])
            .await?;

        let mut strings: Vec<String> = Vec::new();
        for row in rows {
            strings.push(row.get::<usize, String>(0));
        }
        Ok(strings)
    }
    pub async fn save(&mut self, data: Vec<String>) -> Result<()> {
        let txn = self.client.transaction().await?;
        txn.execute(&self.clear_query, &[&self.list])
            .await?;
        for (i, task) in data.iter().enumerate() {
            txn.execute(&self.save_query, &[&(i as i32), task, &self.list])
                .await?;
            }
        txn.commit().await?;
        Ok(())
    }
    pub async fn nuke(&mut self) -> Result<()> {
        self.client.execute(&self.clear_query, &[&self.list])
            .await?;
        Ok(())
    }
    pub async fn list_lists(&mut self) -> Result<Vec<String>> {
        let rows = self.client.query(&self.list_list_query, &[])
            .await?;
        let mut lists = Vec::new();
        for row in rows {
            lists.push(row.get(0));
        }
        Ok(lists)
    }
    pub async fn all_first_tasks(&mut self) -> Result<Vec<(String,String)>> {
        let rows = self.client.query(&self.all_first_tasks_query, &[])
            .await?;
        let mut tasks = Vec::new();
        for row in rows {
            tasks.push((row.get(0), row.get(1)));
        }
        Ok(tasks)
    }
}


