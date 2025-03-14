use std::ops::{Bound, RangeBounds};
use bincode::{Encode,Decode};

#[derive(Debug,Default,Copy,Clone,Encode,Decode)]
pub struct Range {
    pub start: usize,
    pub end: usize,
}

impl Range {
    pub fn new(start: usize, end: usize) -> Range {
        Range { start, end }
    }
    pub fn len(&self) -> usize {
        self.end - self.start
    }
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

impl RangeBounds<usize> for Range {
    fn start_bound(&self) -> Bound<&usize> {
        Bound::Included(&self.start)
    }
    fn end_bound(&self) -> Bound<&usize> {
        Bound::Excluded(&self.end)
    }
}

#[derive(Debug,Default,Encode,Decode)]
pub struct Ranges {
    ranges: Vec<Range>,
    highest: usize,
}


impl Ranges {
    pub fn new() -> Ranges {
        Ranges::default()
    }
    pub fn find(&mut self, length: usize) -> Result<Range, Range> {
        let mut least_larger_index: Option<usize> = None;
        let mut least_larger_size: usize = 0;
        for (i, range) in self.ranges.iter_mut().enumerate() {
            if range.len() >= length && (least_larger_index.is_none() || range.len() < least_larger_size) {
                least_larger_index = Some(i);
                least_larger_size = range.len();
            }
        }
        match least_larger_index {
            Some(index) => {
                let range = &mut self.ranges[index];
                let ret = Ok(Range{start: range.start, end: range.start + length});
                range.start += length;
                if range.is_empty() {
                    self.ranges.remove(index);
                }
                ret
            },
            None => {
                let range = Range::new(self.highest, self.highest + length);
                self.highest = range.end;
                Err(range)
            },
        }
    }

    pub fn len(&self) -> usize {
        self.ranges.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    pub fn highest(&self) -> usize {
        self.highest
    }

    pub fn free(&mut self, to_free: Range) {
        if self.ranges.is_empty() {
            self.ranges.push(to_free);
            return;
        }
        let mut j = self.ranges.len();
        let mut extended: Option<usize> = None;
        let mut consolidated: Option<usize> = None;
        for (i, range) in self.ranges.iter_mut().enumerate() {
            if extended.is_some() {
                if range.start == to_free.end {
                    consolidated = Some(i);
                    break;
                }
                return;
            }
            if range.end < to_free.start {
                continue;
            }
            if range.start == to_free.end {
                range.start = to_free.start;
                return;
            }
            if range.end == to_free.start {
                range.end = to_free.end;
                extended = Some(i);
                continue;
            }
            if range.start > to_free.end {
                j = i;
                break;
            }
        }
        if extended.is_some() && consolidated.is_some() {
            let consolidated_range = *self.ranges.get(consolidated.unwrap()).unwrap();
            let extended_range = self.ranges.get_mut(extended.unwrap()).unwrap();
            extended_range.end = consolidated_range.end;
            self.ranges.remove(consolidated.unwrap());
            return
        }
        self.ranges.insert(j, to_free);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ranges() {
        /*
        use log::LevelFilter;
        colog::default_builder()
            .filter(None, LevelFilter::Debug)
            .init();
        */

        let mut ranges = Ranges::new();

        // Try allocating ranges and expect consecutive spans.
        let range = match ranges.find(3) {
            Ok(range) => panic!("Expected new allocation, found {:?}", range),
            Err(range) => range,
        };
        assert_eq!(range.start, 0);
        assert_eq!(range.end, 3);
        assert_eq!(ranges.highest(), 3);
        let range = match ranges.find(5) {
            Ok(range) => panic!("Expected new allocation, found {:?}", range),
            Err(range) => range,
        };
        assert_eq!(range.start, 3);
        assert_eq!(range.end, 8);
        assert_eq!(ranges.highest(), 8);
        let range = match ranges.find(7) {
            Ok(range) => panic!("Expected new allocation, found {:?}", range),
            Err(range) => range,
        };
        assert_eq!(range.start, 8);
        assert_eq!(range.end, 15);
        assert_eq!(ranges.highest(), 15);

        // No free ranges should have been recorded.
        assert_eq!(ranges.len(), 0);

        // Try freeing a range
        ranges.free(Range::new(3, 8));
        assert_eq!(ranges.len(), 1);

        // Try re-allocating the same range
        // Expect the same range to be re-used.
        let range = match ranges.find(5) {
            Ok(range) => range,
            Err(range) => panic!("Expected re-use, found {:?}", range),
        };
        assert_eq!(range.start, 3);
        assert_eq!(range.end, 8);

        ranges.free(Range::new(3, 8));
        assert_eq!(ranges.len(), 1);

        // Try allocating a smaller range that fits most tightly
        // in the middle span.
        let range = match ranges.find(4) {
            Ok(range) => range,
            Err(range) => panic!("Expected re-use, found {:?}", range),
        };
        assert_eq!(range.start, 3);
        assert_eq!(range.end, 7);
        assert_eq!(ranges.len(), 1);

        // Try allocating the tiny remainder of that span.
        let range = match ranges.find(1) {
            Ok(range) => range,
            Err(range) => panic!("Expected re-use, found {:?}", range),
        };
        assert_eq!(range.start, 7);
        assert_eq!(range.end, 8);
        assert_eq!(ranges.len(), 0);

        // Try freeing up the first and last spans and
        // finding a new span that only fits in the last.
        ranges.free(Range::new(0, 3));
        ranges.free(Range::new(8, 15));
        let range = match ranges.find(6) {
            Ok(range) => range,
            Err(range) => panic!("Expected re-use, found {:?}", range),
        };
        assert_eq!(range.start, 8);
        assert_eq!(range.end, 14);
        assert_eq!(ranges.len(), 2);

        // Try freeing up the last span to make sure that
        // the free span extends the last span rather than
        // recording a new one.
        ranges.free(range);
        assert_eq!(ranges.len(), 2);

        // Try freeing up the middle span to make sure that
        // the two remaining spans are consolidated.
        ranges.free(Range::new(3, 8));
        assert_eq!(ranges.len(), 1);

    }
}
