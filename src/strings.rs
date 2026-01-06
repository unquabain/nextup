use crate::range::{Range,Ranges};
use log::debug;

#[derive(Debug,Default)]
pub struct Strings {
    free_ranges: Ranges,
    strings: String,
}

#[derive(Debug)]
pub struct StringsInspectionEntry<'a> {
    pub range: Range,
    pub free: bool,
    pub value: &'a str,
}

#[derive(Debug)]
pub struct StringsInspector<'a> {
    strings: &'a Strings,
    index: usize,
    prefix_emitted: bool,
}

impl<'a> StringsInspector<'a> {
    pub fn new(strings: &'a Strings) -> StringsInspector<'a> {
        StringsInspector {
            strings,
            index: 0,
            prefix_emitted: false,
        }
    }
    fn first_entries(&mut self) -> Option<StringsInspectionEntry<'a>> {
        let range = match self.strings.free_ranges.get(self.index)  {
            Some(&range) => range,
            None => Range::new(self.strings.free_ranges.highest(), self.strings.free_ranges.highest()),
        };

        if range.start == 0 {
            self.prefix_emitted = true;
        }
        if self.prefix_emitted {
            let value = self.strings.get(range);
            self.index += 1;
            self.prefix_emitted = false;
            return Some(StringsInspectionEntry {
                range,
                free: true,
                value,
            });
        }
        let value = self.strings.get(Range::new(0, range.start));
        self.prefix_emitted = true;
        Some(StringsInspectionEntry {
            range: Range::new(0, range.start),
            free: false,
            value,
        })
    }

    fn last_entry(&mut self) -> Option<StringsInspectionEntry<'a>> {
        if self.prefix_emitted {
            return None;
        }
        if self.index == 0 {
            return None;
        }
        let range = match self.strings.free_ranges.get(self.index - 1) {
            Some(&range) => range,
            None => Range::new(0, 0),
        };
        if self.strings.free_ranges.highest() <= range.end {
            return None;
        }
        let string = self.strings.get(Range { start: range.end, end: self.strings.free_ranges.highest() });
        self.prefix_emitted = true;
        Some(StringsInspectionEntry {
            range: Range { start: range.end, end: self.strings.free_ranges.highest() },
            free: false,
            value: string,
        })
    }
}

impl<'a> std::iter::Iterator for StringsInspector<'a> {
    type Item = StringsInspectionEntry<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.strings.free_ranges.len() {
            return self.last_entry();
        }
        if self.index == 0 {
            return self.first_entries();
        }
        let range = self.strings.free_ranges.get(self.index).unwrap();
        if self.prefix_emitted {
            let string = self.strings.get(*range);
            self.index += 1;
            self.prefix_emitted = false;
            debug!("emitting free range: {:?}", range);
            Some(StringsInspectionEntry {
                range: *range,
                free: true,
                value: string,
            })
        } else {
            let last_range = self.strings.free_ranges.get(self.index - 1).unwrap();
            assert!(last_range.end <= range.start);
            let string = self.strings.get(Range::new(last_range.end, range.start));
            self.prefix_emitted = true;
            debug!("emitting used range between: {:?} and {:?}", last_range, range);
            Some(StringsInspectionEntry {
                range: Range::new(last_range.end, range.start),
                free: false,
                value: string,
            })
        }
    }
}

impl Strings {
    pub fn new() -> Strings {
        Strings {
            free_ranges: Ranges::new(),
            strings: String::new(),
        }
    }
    pub fn add(&mut self, string: &str) -> Range {
        let length = string.len();
        match self.free_ranges.find(length) {
            Ok(range) => {
                self.strings.replace_range(range, string);
                range
            },
            Err(range) => {
                self.strings.push_str(string);
                range
            },
        }
    }
    pub fn strlen(&self) -> usize {
        self.strings.len()
    }

    pub fn iter<'s>(&'s self) -> StringsInspector<'s> {
        StringsInspector::new(self)
    }
    pub fn get(&self, range: Range) -> &str {
        &self.strings[range.start..range.end]
    }
    pub fn free(&mut self, range: Range) {
        self.free_ranges.free(range);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_strings() {
        let mut strings = Strings::new();
        let range1 = strings.add("hello");
        let range2 = strings.add("world");
        assert_eq!(strings.get(range1), "hello");
        assert_eq!(strings.get(range2), "world");

        // The underlying data should have the words concatenated
        assert_eq!(strings.strings, "helloworld");

        // Freeing a range should not alter the data.
        strings.free(range1);
        assert_eq!(strings.strings, "helloworld");

        // Adding a new string that is longer than any freed
        // range should append to the end of the data.
        let range3 = strings.add("goodbye");
        assert_eq!(strings.get(range3), "goodbye");
        assert_eq!(strings.strings, "helloworldgoodbye");

        // Adding a new string that fits in a freed range
        // should replace the data in that range.
        let range4 = strings.add("cruel");
        assert_eq!(strings.get(range4), "cruel");
        assert_eq!(strings.strings, "cruelworldgoodbye");

        // Adding a shorter string should reuse the freed range.
        strings.free(range4);
        let range5 = strings.add("mean");
        assert_eq!(strings.get(range5), "mean");
        assert_eq!(strings.strings, "meanlworldgoodbye");

        // Old indexes should still work.
        assert_eq!(strings.get(range2), "world");

    }
}
