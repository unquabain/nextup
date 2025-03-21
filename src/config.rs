use serde::Deserialize;
use std::path::PathBuf;
use crate::error::Error;
use crate::datasource::{DataSource, bincode::Bincode, postgres::DSPostgres};
use log::{debug,warn};

#[derive(Debug,Deserialize)]
pub enum DataSourceType {
    Bincode(PathBuf),
    Postgres(String),
}

impl DataSourceType {
    pub fn data_source(&self, list: &str) -> Result<Box::<dyn DataSource>,Error> {
        Ok(match self {
            DataSourceType::Bincode(path) => {
                let mut path = path.clone();
                path.push(list);
                Box::new(Bincode{path})
            },
            DataSourceType::Postgres(connection_string) => {
                let ds = DSPostgres::new(connection_string, list)?;
                Box::new(ds)
            },
        })
    }
    pub fn from_toml(table: &toml::Table) -> Result<Self, Error> {
        let dstype = match table.get("data_source") {
            Some(toml::Value::String(s)) => s,
            _ => return Err(Error::new("data source must be a string")),
        };
        let dstype_table = match table.get(dstype) {
            Some(toml::Value::Table(t)) => Some(t),
            None => None,
            _ => return Err(Error::new(&format!("{} must be a table", dstype))),
        };
        match dstype.as_str() {
            "bincode" => {
                match dstype_table {
                    Some(dstype_table) =>
                        match dstype_table.get("path") {
                            Some(toml::Value::String(p)) => Ok(DataSourceType::Bincode(PathBuf::from(p))),
                            // path exists, but is something weird
                            Some(_) => Err(Error::new("path must be a string")),
                            // No path, use default
                            None => Ok(Self::default()),
                        },
                    // No [bincode] table, use default
                    None => Ok(Self::default()),
                }
            },
            "postgres" => {
                match dstype_table {
                    Some(dstype_table) =>
                        match dstype_table.get("connection_string") {
                            Some(toml::Value::String(p)) => Ok(DataSourceType::Postgres(p.clone())),
                            // connection_string is required
                            _ => Err(Error::new("connection_string must be provided")),
                        },
                    None => Ok(Self::default()),
                }
            },
            _ => Err(Error::new(&format!("unknown data source type: {}", dstype)))
        }
    }
}

impl Default for DataSourceType {
    fn default() -> Self {
        let mut path = dirs::config_dir().unwrap_or("./".into());
        path.push("nextup");
        path.push("default");

        DataSourceType::Bincode(path)
    }
}

const DEFAULT_LIST_STRING: &str = "list";

fn default_list() -> String {
    DEFAULT_LIST_STRING.to_string()
}

#[derive(Debug,Deserialize)]
pub struct Config {
    #[serde(default)]
    pub data_source: DataSourceType,

    #[serde(default="default_list")]
    pub list: String,
}

impl Config {
    pub fn filepath_or_default(path: &Option<PathBuf>) -> Option<PathBuf> {
        if path.is_some() {
            return path.clone()
        }
        let mut path = PathBuf::from("./.nextup.toml");
        if path.exists() {
            return Some(path)
        }
        path = PathBuf::from("./.nextup.conf");
        if path.exists() {
            return Some(path)
        }
        path = dirs::config_dir().unwrap_or("./".into());
        path.push("nextup");
        path.push("config.toml");
        if path.exists() {
            return Some(path)
        }
        path.pop();
        path.push("config.conf");
        if path.exists() {
            return Some(path)
        }
        path = PathBuf::from("/etc/nextup.toml");
        if path.exists() {
            return Some(path)
        }
        path.pop();
        path.push("nextup.conf");
        None
    }
    pub fn from_read(read: &mut impl std::io::Read) -> Result<Self, Error> {
        let mut s = String::new();
        read.read_to_string(&mut s).map_err(|e| { Error::from_error(e) })?;
        let table: toml::Table = toml::from_str(&s).map_err(|e| {
            warn!("error parsing config file: {}", e);
            Error::from_error(e)
        })?;
        let list = match table.get("list") {
            Some(toml::Value::String(s)) => s.clone(),
            None => default_list(),
            _ => return Err(Error::new("list must be a string")),
        };
        let data_source = DataSourceType::from_toml(&table)?;
        Ok(Config {
            data_source,
            list,
        })
    }

    pub fn from_filepath(path: &std::path::Path) -> Result<Self, Error> {
        debug!("reading config from file: {:?}", path);
        let mut file = std::fs::File::open(path)
            .map_err(|e| Error::new(&format!("could not open file: {:?}", e)))?;
        debug!("file opened");
        Self::from_read(&mut file)
    }

    pub fn data_source(&self) -> Result<Box<dyn DataSource>,Error> {
        self.data_source.data_source(&self.list)
    }
}


impl Default for Config {
    fn default() -> Self {
        Config {
            data_source: DataSourceType::default(),
            list: default_list(),
        }
    }
}
