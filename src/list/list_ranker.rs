use crate::error::Error;

pub enum Direction {
    Promote,
    Demote,
}

pub trait ListRanker {
    fn strings(&self) -> Option<(&str, &str, Option<&str>)>;
    fn choose(&mut self, choice: i32) -> Result<bool, Error>;
}

