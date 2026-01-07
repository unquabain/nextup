use std::ops::{Bound, RangeBounds};
use log::debug;

#[derive(Debug,Default,Copy,Clone, PartialEq, Eq)]
pub struct Range {
    pub start: usize,
    pub end: usize,
}

impl Range {
    pub fn new(start: usize, end: usize) -> Range {
        Range { start, end }
    }
    fn len(&self) -> usize {
        self.end - self.start
    }
    fn is_empty(&self) -> bool {
        self.start == self.end
    }
    fn is_extended_by(&self, other: &Range) -> bool {
        self.end == other.start
    }
    fn extends(&self, other: &Range) -> bool{
        other.is_extended_by(self)
    }
    fn is_contiguous(&self, other: &Range) -> bool {
        self.is_extended_by(other) || self.extends(other)
    }
    fn extend(&mut self, other: &Range) {
        if self.is_extended_by(other) {
            self.end = other.end;
        } else if self.extends(other) {
            self.start = other.start;
        }
    }

    fn maybe_extend(&mut self, other: Option<&Range>) -> bool {
        match other {
            Some(r) if self.is_contiguous(r) => {
                self.extend(r);
                true
            },
            _ => false,
        }
    }
}

impl Ord for Range {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start.cmp(&other.start)
    }
}

impl PartialOrd for Range {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
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

#[derive(Debug,Default)]
pub struct Ranges {
    ranges: Vec<Range>,
    highest: usize,
}

enum RangesFreeOp {
    Overwrite(usize), // Replace range at index
    Shorten(usize), // Replace range at index and remove next
    Extend(usize), // Insert before index
    Push, // Insert at end
}

impl Ranges {
    pub fn new() -> Ranges {
        Ranges::default()
    }
    pub fn get(&self, index: usize) -> Option<&Range> {
        self.ranges.get(index)
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

    pub fn free(&mut self, mut to_free: Range) {
        if self.ranges.is_empty() {
            self.ranges.push(to_free);
            self.validate();
            return;
        }

        // Find the next range after the one being freed.
        let next_idx = self.ranges.iter().position(|r| r > &to_free);
        let next = next_idx
            .and_then(|idx| self.ranges.get(idx));

        // Find the previous range before the one being freed.
        let prev_idx = match next_idx {
            None => Some(self.ranges.len() - 1),
            Some(0) => None,
            Some(n) => Some(n - 1),
        };
        let prev = prev_idx.and_then(|idx| self.ranges.get(idx));

        // Determine the operation to perform based on contiguity.
        let operation = if to_free.maybe_extend(prev) {
            if to_free.maybe_extend(next) {
                // Logically, prev_idx must be Some here.
                RangesFreeOp::Shorten(prev_idx.unwrap_or_default())
            } else {
                // Logically, prev_idx must be Some here.
                RangesFreeOp::Overwrite(prev_idx.unwrap_or_default())
            }
        } else if to_free.maybe_extend(next) {
            // Logically, next_idx must be Some here.
            RangesFreeOp::Overwrite(next_idx.unwrap_or_default())
        } else {
            match next_idx {
                Some(idx) => {
                    RangesFreeOp::Extend(idx)
                },
                None => {
                    RangesFreeOp::Push
                },
            }
        };
        match operation {
            RangesFreeOp::Overwrite(idx) => {
                self.ranges[idx] = to_free;
            },
            RangesFreeOp::Shorten(idx) => {
                self.ranges[idx] = to_free;
                self.ranges.remove(idx + 1);
            },
            RangesFreeOp::Extend(idx) => {
                self.ranges.insert(idx, to_free);
            },
            RangesFreeOp::Push => {
                self.ranges.push(to_free);
            },
        }
    }

    #[cfg(debug_assertions)]
    fn validate(&self) {
        if self.ranges.len() < 2 {
            return;
        }
        for (i, r) in self.ranges[1..self.ranges.len()].iter().enumerate() {
            let previous = &self.ranges[i];
            debug!("validating range {} {:?} > {:?}", i+1, r, previous);
            assert!(r.start >= previous.end);
            assert!(r.end <= self.highest);
        }
    }

    #[cfg(not (debug_assertions))]
    fn validate(&self) {}
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
