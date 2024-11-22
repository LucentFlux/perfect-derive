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

#[allow(clippy::single_component_path_imports)]
pub(crate) use impls;
