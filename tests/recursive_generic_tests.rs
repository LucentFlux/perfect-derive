use perfect_derive::perfect_derive;
use std::ops::Deref;
use std::rc::Rc;

pub struct NonClonable {}

#[perfect_derive(Clone)]
pub struct List<T> {
    data: Rc<T>,
    next: Option<Rc<List<T>>>,
}

impl<T> Deref for List<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.data
    }
}

#[test]
pub fn functional_list_is_clonable()
where
    List<NonClonable>: Clone,
{
    // Nop
}
