use crate::list::{List, ListRanker, parent, left, right};
use crate::list::Direction;
use crate::error::Error;
use log::debug;

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
    fn choose(&mut self, choice: i32) -> Result<bool, Error> {
        debug!("chose {}", choice);
        match choice {
            0 => Ok(false),

            1 => match self.direction {
                Direction::Promote => Ok(self.promote()),
                Direction::Demote => Ok(self.demote_left()),
            },

            2 => match self.direction {
                Direction::Demote => Ok(self.demote_right()),
                _ => Err(Error::new("Invalid choice")),
            },

            _ => Err(Error::new("Invalid choice")),
        }
    }
}

