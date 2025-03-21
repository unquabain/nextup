use crate::list::List;

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
