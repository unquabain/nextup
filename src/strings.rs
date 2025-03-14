use crate::range::{Range,Ranges};
use bincode::{Encode,Decode};

#[derive(Debug,Default,Encode,Decode)]
pub struct Strings {
    free_ranges: Ranges,
    strings: String,
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
