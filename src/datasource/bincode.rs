use std::path::PathBuf;
use crate::error::Error;

use bincode::{decode_from_std_read,encode_into_std_write,config::Config};

fn bcconfig() -> impl Config {
    bincode::config::standard()
        .with_little_endian()
        .with_fixed_int_encoding()
}

#[derive(Debug)]
pub struct Bincode {
    pub path: PathBuf,
}

impl Bincode {
    pub fn load(&mut self) -> Result<Vec<String>, Error> {
        let mut file = std::fs::File::open(&self.path)
            .map_err(|e| Error::new(&format!("could not open file: {:?}", e)))?;
        let strings: Vec<String> =
         decode_from_std_read(&mut file, bcconfig())
            .map_err(|e| Error::new(&format!("could not deserialize file: {:?}", e)))?;
        Ok(strings)
    }

    pub fn save(&mut self, data: Vec<String>) -> Result<(), Error>{
        let dir = self.path.parent().ok_or_else(|| Error::new("could not get parent directory"))?;
        if !dir.exists() {
            std::fs::create_dir_all(dir)
                .map_err(|e| Error::new(&format!("could not create directory: {:?}", e)))?;
        }
        let mut file = std::fs::File::create(&self.path)
            .map_err(|e| Error::new(&format!("could not create file: {:?}", e)))?;
        let _ = encode_into_std_write(data, &mut file, bcconfig())
            .map_err(|e| Error::new(&format!("could not serialize file: {:?}", e)))?;
        Ok(())
    }
    pub fn nuke(&mut self) -> Result<(), Error> {
        std::fs::remove_file(&self.path)
            .map_err(|e| Error::new(&format!("could not remove file: {:?}", e)))?;
        Ok(())
    }
}


