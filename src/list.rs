use crate::range::Range;
use crate::strings::Strings;
use bincode::{decode_from_std_read,encode_into_std_write,Encode,Decode,config::Config};
use log::debug;

fn bcconfig() -> impl Config {
    bincode::config::standard()
        .with_little_endian()
        .with_fixed_int_encoding()
}

#[derive(Debug,Default,Encode,Decode)]
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

#[derive(Debug)]
pub struct ListError {
    message: String,
}

impl std::fmt::Display for ListError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ListError {}
impl ListError {
    pub fn new(message: &str) -> ListError {
        ListError {
            message: message.to_string(),
        }
    }
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

    pub fn load(path: &std::path::PathBuf) -> Result<List,ListError> {
        let mut file = std::fs::File::open(path)
            .map_err(|e| ListError::new(&format!("could not open file: {:?}", e)))?;
         decode_from_std_read(&mut file, bcconfig())
            .map_err(|e| ListError::new(&format!("could not deserialize file: {:?}", e)))
    }

    fn used_ranges_len(&self) -> usize {
        let mut sum = 0;
        for range in &self.rank {
            sum += range.len();
        }
        sum
    }
    pub fn should_repack(&self) -> bool {
        if !self.dirty {
            return false;
        }
        self.strings.strlen() > self.used_ranges_len()*2
    }

    pub fn repack(&mut self) {
        let mut new_strings = Strings::new();
        let mut new_rank = Vec::new();
        for range in &self.rank {
            let new_range = new_strings.add(self.strings.get(*range));
            new_rank.push(new_range);
        }
        self.strings = new_strings;
        self.rank = new_rank;
        self.dirty = true;
    }
    pub fn save(&self, path: &std::path::PathBuf) -> Result<(),ListError> {
        if !self.dirty {
            return Ok(());
        }
        let dir = path.parent().ok_or_else(|| ListError::new("could not get parent directory"))?;
        if !dir.exists() {
            std::fs::create_dir_all(dir)
                .map_err(|e| ListError::new(&format!("could not create directory: {:?}", e)))?;
        }
        let mut file = std::fs::File::create(path)
            .map_err(|e| ListError::new(&format!("could not create file: {:?}", e)))?;
        let _ = encode_into_std_write(self, &mut file, bcconfig())
            .map_err(|e| ListError::new(&format!("could not serialize file: {:?}", e)))?;
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

    pub fn complete(&mut self) -> Result<Cursor, ListError> {
        if self.rank.is_empty() {
            return Err(ListError::new("no tasks to complete"));
        }
        debug!("completing root");
        self.delete(0)
    }

    pub fn delete(&mut self, index: usize) -> Result<Cursor,ListError> {
        if index >= self.rank.len() {
            return Err(ListError::new("index out of range"));
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

    pub fn defer(&mut self) -> Cursor {
        debug!("demoting root");
        Cursor::new(self, 0, Direction::Demote)
    }

    pub fn iter(&self) -> ListIterator {
        ListIterator::new(self)
    }
}

pub enum Direction {
    Promote,
    Demote,
}

pub trait ListRanker {
    fn strings(&self) -> Option<(&str, &str, Option<&str>)>;
    fn choose(&mut self, choice: i32) -> Result<bool, &'static str>;
}

pub struct Cursor<'list> {
    list: &'list mut List,
    index: usize,
    direction: Direction,
}

impl<'list> Cursor<'list> {
    pub fn new(list: &'list mut List, index: usize, direction: Direction) -> Cursor<'list> {
        Cursor {
            list,
            index,
            direction,
        }
    }
    pub fn strings_for_promotion(&self) -> Option<(&str, &str, Option<&str>)> {
        if self.index == 0 {
            return None;
        }
        let parent = parent(self.index);
        Some((self.list.get(parent).unwrap(), self.list.get(self.index).unwrap(), None))
    }
    pub fn promote(&mut self) -> bool {
        if self.index == 0 {
            return false;
        }
        let child = self.index;
        self.index = parent(self.index);
        self.list.promote(child)
    }
    pub fn strings_for_demotion(&self) -> Option<(&str, &str, Option<&str>)> {
        let left = left(self.index);
        let right = right(self.index);
        if left >= self.list.rank.len() {
            return None;
        }
        let left = self.list.get(left).unwrap();
        if right >= self.list.rank.len() {
            return Some((self.list.get(self.index).unwrap(), left, None));
        }
        let right = self.list.get(right).unwrap();
        Some((self.list.get(self.index).unwrap(), left, Some(right)))
    }
    pub fn demote_left(&mut self) -> bool {
        self.index = left(self.index);
        self.list.promote(self.index)
    }
    pub fn demote_right(&mut self) -> bool {
        self.index = right(self.index);
        self.list.promote(self.index)
    }
}

impl ListRanker for Cursor<'_> {
    fn strings(&self) -> Option<(&str, &str, Option<&str>)> {
        match self.direction {
            Direction::Promote => self.strings_for_promotion(),
            Direction::Demote => self.strings_for_demotion(),
        }
    }
    fn choose(&mut self, choice: i32) -> Result<bool, &'static str> {
        debug!("chose {}", choice);
        match choice {
            0 => Ok(false),

            1 => match self.direction {
                Direction::Promote => Ok(self.promote()),
                Direction::Demote => Ok(self.demote_left()),
            },

            2 => match self.direction {
                Direction::Demote => Ok(self.demote_right()),
                _ => Err("Invalid choice"),
            },

            _ => Err("Invalid choice"),
        }
    }
}

pub struct ListIterator<'list> {
    list: &'list List,
    index: usize,
}

impl<'list> ListIterator<'list> {
    pub fn new(list: &'list List) -> ListIterator<'list> {
        ListIterator {
            list,
            index: 0,
        }
    }
}

impl<'list> Iterator for ListIterator<'list> {
    type Item = &'list str;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.list.get(self.index);
        self.index += 1;
        result
    }
}
