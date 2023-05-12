use perfect_derive::perfect_derive;
use std::rc::Rc;

pub struct Error {}

#[perfect_derive(Clone)]
pub struct ResultWrapper<Ok, Err = Error> {
    data: Rc<Result<Ok, Err>>,
}

#[test]
pub fn functional_list_is_clonable()
where
    ResultWrapper<()>: Clone,
{
    // Nop
}
