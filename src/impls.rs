#[cps::cps]
macro_rules! impls {
    () =>
    {
        Copy,
        Clone,
        PartialEq,
        Eq,
        Ord,
        PartialOrd,
        Hash,
        Default,
        Debug,
    };
}

pub(crate) use impls;
