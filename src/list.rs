mod list_ranker;
mod cursor;
mod iterator;
use crate::range::Range;
use crate::strings::Strings;
use crate::error::Error;
use crate::datasource::DataSource;
use log::debug;

pub use list_ranker::{Direction, ListRanker};
pub use cursor::Cursor;
pub use iterator::ListIterator;

#[derive(Debug,Default)]
pub struct List {
    strings: Strings,
    rank: Vec<Range>,
    dirty: bool,
}

pub fn parent(child: usize) -> usize {
    (child - 1) / 2
}

pub fn left(parent: usize) -> usize {
    parent * 2 + 1
}

pub fn right(parent: usize) -> usize {
    parent * 2 + 2
}

impl List {
    pub fn new() -> List {
        List {
            strings: Strings::new(),
            rank: Vec::new(),
            dirty: false,
        }
    }

    pub fn strings(&self) -> &Strings {
        &self.strings
    }

    pub async fn load(data: &mut DataSource) -> Result<List,Error> {
        log::trace!("Loading list from data source");
        let mut strings = Strings::new();
        let mut rank = Vec::new();
        for string in data.load().await? {
            let range = strings.add(&string);
            rank.push(range);
        }
        Ok(List {
            strings,
            rank,
            dirty: false,
        })
    }

    pub async fn save(&mut self, data: &mut DataSource) -> Result<(),Error> {
        if !self.dirty {
            return Ok(());
        }
        let strings = self.iter().map(|s| s.to_string()).collect();
        data.save(strings).await?;
        Ok(())
    }

    pub fn nextup(&self) -> Option<&str> {
        self.get(0)
    }

    pub fn get(&self, index: usize) -> Option<&str> {
        if index < self.rank.len() {
            Some(self.strings.get(self.rank[index]))
        } else {
            None
        }
    }

    pub fn promote(&mut self, index: usize) -> bool {
        if index == 0 {
            debug!("reached root; not promoting");
            return false;
        }
        let parent = parent(index);
        debug!("promoting {} to {}", index, parent);
        self.rank.swap(parent, index);
        self.dirty = true;
        true
    }
    pub fn add<'receiver>(&'receiver mut self, string: &str) -> Cursor<'receiver> {
        let range = self.strings.add(string);
        let index = self.rank.len();
        self.rank.push(range);
        self.dirty = true;
        debug!("added {} at {}", string, index);
        Cursor::new(self, index, Direction::Promote)
    }

    pub fn complete<'list>(&'list mut self) -> Result<Cursor<'list>, Error> {
        if self.rank.is_empty() {
            return Err(Error::new("no tasks to complete"));
        }
        debug!("completing root");
        self.delete(0)
    }

    pub fn delete<'list>(&'list mut self, index: usize) -> Result<Cursor<'list>,Error> {
        if index >= self.rank.len() {
            return Err(Error::new("index out of range"));
        }
        debug!("freeing string at index {} range {:?}", index, self.rank[index]);
        self.strings.free(self.rank[index]);
        self.rank.swap_remove(index);
        self.dirty = true;
        if index < self.rank.len() {
            debug!("new temporary value is {:?}", self.strings.get(self.rank[index]));
        }
        Ok(Cursor::new(self, index, Direction::Demote))
    }

    pub fn defer<'list>(&'list mut self) -> Cursor<'list> {
        debug!("demoting root");
        Cursor::new(self, 0, Direction::Demote)
    }

    pub fn replace(&mut self, index: usize, task: &str) -> Result<(), Error> {
        if index >= self.rank.len() {
            return Err(Error::new("index out of range"));
        }
        let old_rank = self.rank[index];
        self.strings.free(old_rank);
        let new_rank = self.strings.add(task);
        self.rank[index] = new_rank;
        self.dirty = true;
        Ok(())
    }

    pub fn iter<'list>(&'list self) -> ListIterator<'list> {
        ListIterator::new(self)
    }
}
