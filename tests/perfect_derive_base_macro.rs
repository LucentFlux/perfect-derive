use std::hash::Hash;

use perfect_derive::perfect_derive;

fn hash_to_int(v: &impl Hash) -> u64 {
    use std::hash::Hasher;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    v.hash(&mut hasher);
    hasher.finish()
}

macro_rules! make_test {
    ($trait_name:ident $(,$trait_name_tail:ident)*; $method_name:ident) => {
        #[allow(unused)]
        mod $method_name {
            use perfect_derive::perfect_derive;

            #[perfect_derive($trait_name $(,$trait_name_tail)*)]
            pub struct Struct {
                v1: usize,
                pub v2: i32,
            }

            #[perfect_derive($trait_name $(,$trait_name_tail)*)]
            pub struct Struct2 (
                pub(crate) usize,
                pub i32,
            );

            #[perfect_derive($trait_name $(,$trait_name_tail)*)]
            pub struct Struct3;

            #[perfect_derive($trait_name $(,$trait_name_tail)*)]
            pub struct Struct4 {
                #[allow(unused)]
                pub(super) r#fn: bool
            }

            #[perfect_derive($trait_name $(,$trait_name_tail)*)]
            pub enum Enum {
                E1,
                E2(),
                E3(usize),
                E4(u32, ()),
                E5{name1: u32, name2: ()},
            }

            #[allow(unused_imports)]
            mod inner {
                use super::*;
                use std::fmt::Debug;
                use std::hash::Hash;

                #[test]
                pub fn $method_name()
                where
                    Struct: $trait_name,
                    Struct2: $trait_name,
                    Struct3: $trait_name,
                    Struct4: $trait_name,
                    Enum: $trait_name,
                {
                    // No need to do anything
                }
            }
        }
    };
}

make_test!(Copy, Clone; copy);
make_test!(Clone; clone);
make_test!(PartialEq; peq);
make_test!(Eq, PartialEq; eq);
make_test!(Ord, Eq, PartialOrd, PartialEq; ord);
make_test!(PartialOrd, PartialEq; pord);
make_test!(Debug; debug);
make_test!(Hash; hash);

#[derive(Copy, Clone, Ord, Eq, PartialOrd, PartialEq, Debug, Hash, Default)]
struct EverythingStructCore {
    v1: usize,
    pub v2: i32,
}

#[perfect_derive(Copy, Clone, Ord, Eq, PartialOrd, PartialEq, Debug, Hash, Default)]
struct EverythingStruct {
    v1: usize,
    pub v2: i32,
}

#[test]
fn struct_eq_matches() {
    let c1 = EverythingStructCore { v1: 1, v2: 2 };
    let c2 = EverythingStructCore { v1: 2, v2: 1 };

    let s1 = EverythingStruct { v1: 1, v2: 2 };
    let s2 = EverythingStruct { v1: 2, v2: 1 };

    assert_eq!(s1.eq(&s1), c1.eq(&c1));
    assert_eq!(s1.eq(&s2), c1.eq(&c2));
    assert_eq!(s2.eq(&s1), c2.eq(&c1));
}

#[test]
fn struct_ord_matches() {
    let c1 = EverythingStructCore { v1: 1, v2: 2 };
    let c2 = EverythingStructCore { v1: 2, v2: 1 };

    let s1 = EverythingStruct { v1: 1, v2: 2 };
    let s2 = EverythingStruct { v1: 2, v2: 1 };

    assert_eq!(s1.cmp(&s1), c1.cmp(&c1));
    assert_eq!(s1.cmp(&s2), c1.cmp(&c2));
    assert_eq!(s2.cmp(&s1), c2.cmp(&c1));
}

#[test]
fn struct_partial_ord_matches() {
    let c1 = EverythingStructCore { v1: 1, v2: 2 };
    let c2 = EverythingStructCore { v1: 2, v2: 1 };

    let s1 = EverythingStruct { v1: 1, v2: 2 };
    let s2 = EverythingStruct { v1: 2, v2: 1 };

    assert_eq!(s1.partial_cmp(&s1), c1.partial_cmp(&c1));
    assert_eq!(s1.partial_cmp(&s2), c1.partial_cmp(&c2));
    assert_eq!(s2.partial_cmp(&s1), c2.partial_cmp(&c1));
}

#[test]
fn struct_hash_matches() {
    let c1 = EverythingStructCore { v1: 1, v2: 2 };
    let c2 = EverythingStructCore { v1: 2, v2: 1 };

    let s1 = EverythingStruct { v1: 1, v2: 2 };
    let s2 = EverythingStruct { v1: 2, v2: 1 };

    assert_eq!(hash_to_int(&s1), hash_to_int(&c1));
    assert_eq!(hash_to_int(&s2), hash_to_int(&c2));
}

#[derive(Copy, Clone, Ord, Eq, PartialOrd, PartialEq, Debug, Hash, Default)]
#[allow(unused)]
enum EverythingEnumCore {
    #[default]
    E1,
    E2(),
    E3(usize, usize),
    E4(u32, ()),
    E5 {
        name1: u32,
        name2: (),
    },
}

#[perfect_derive(Copy, Clone, Ord, Eq, PartialOrd, PartialEq, Debug, Hash, Default)]
enum EverythingEnum {
    #[default]
    E1,
    E2(),
    E3(usize, usize),
    E4(u32, ()),
    E5 {
        name1: u32,
        name2: (),
    },
}

#[test]
fn enum_eq_matches() {
    let c1 = EverythingEnumCore::E1;
    let c2 = EverythingEnumCore::E3(1, 2);
    let c3 = EverythingEnumCore::E3(2, 1);

    let s1 = EverythingEnum::E1;
    let s2 = EverythingEnum::E3(1, 2);
    let s3 = EverythingEnum::E3(2, 1);

    assert_eq!(s1.eq(&s1), c1.eq(&c1));
    assert_eq!(s2.eq(&s2), c2.eq(&c2));
    assert_eq!(s3.eq(&s3), c3.eq(&c3));

    assert_eq!(s1.eq(&s2), c1.eq(&c2));
    assert_eq!(s2.eq(&s1), c2.eq(&c1));

    assert_eq!(s1.eq(&s3), c1.eq(&c3));
    assert_eq!(s3.eq(&s1), c3.eq(&c1));

    assert_eq!(s3.eq(&s2), c3.eq(&c2));
    assert_eq!(s2.eq(&s3), c2.eq(&c3));
}

#[test]
fn enum_ord_matches() {
    let c1 = EverythingEnumCore::E1;
    let c2 = EverythingEnumCore::E3(1, 2);
    let c3 = EverythingEnumCore::E3(2, 1);

    let s1 = EverythingEnum::E1;
    let s2 = EverythingEnum::E3(1, 2);
    let s3 = EverythingEnum::E3(2, 1);

    assert_eq!(s1.cmp(&s1), c1.cmp(&c1));
    assert_eq!(s2.cmp(&s2), c2.cmp(&c2));
    assert_eq!(s3.cmp(&s3), c3.cmp(&c3));

    assert_eq!(s1.cmp(&s2), c1.cmp(&c2));
    assert_eq!(s2.cmp(&s1), c2.cmp(&c1));

    assert_eq!(s1.cmp(&s3), c1.cmp(&c3));
    assert_eq!(s3.cmp(&s1), c3.cmp(&c1));

    assert_eq!(s3.cmp(&s2), c3.cmp(&c2));
    assert_eq!(s2.cmp(&s3), c2.cmp(&c3));
}

#[test]
fn enum_partial_ord_matches() {
    let c1 = EverythingEnumCore::E1;
    let c2 = EverythingEnumCore::E3(1, 2);
    let c3 = EverythingEnumCore::E3(2, 1);

    let s1 = EverythingEnum::E1;
    let s2 = EverythingEnum::E3(1, 2);
    let s3 = EverythingEnum::E3(2, 1);

    assert_eq!(s1.partial_cmp(&s1), c1.partial_cmp(&c1));
    assert_eq!(s2.partial_cmp(&s2), c2.partial_cmp(&c2));
    assert_eq!(s3.partial_cmp(&s3), c3.partial_cmp(&c3));

    assert_eq!(s1.partial_cmp(&s2), c1.partial_cmp(&c2));
    assert_eq!(s2.partial_cmp(&s1), c2.partial_cmp(&c1));

    assert_eq!(s1.partial_cmp(&s3), c1.partial_cmp(&c3));
    assert_eq!(s3.partial_cmp(&s1), c3.partial_cmp(&c1));

    assert_eq!(s3.partial_cmp(&s2), c3.partial_cmp(&c2));
    assert_eq!(s2.partial_cmp(&s3), c2.partial_cmp(&c3));
}

#[test]
fn enum_hash_matches() {
    let c1 = EverythingEnumCore::E1;
    let c2 = EverythingEnumCore::E3(1, 2);

    let s1 = EverythingEnum::E1;
    let s2 = EverythingEnum::E3(1, 2);

    assert_eq!(hash_to_int(&s1), hash_to_int(&c1));
    assert_eq!(hash_to_int(&s2), hash_to_int(&c2));
}

#[test]
pub fn copy_struct_eq() {
    let s1 = EverythingStruct {
        v1: 177294,
        v2: 98264,
    };

    let s2 = s1;

    assert!(s1.eq(&s2));
}

#[test]
pub fn copy_unit_enum_eq() {
    let s1 = EverythingEnum::E1;

    let s2 = s1;

    assert!(s1.eq(&s2));
}

#[test]
pub fn copy_full_enum_eq() {
    let s1 = EverythingEnum::E4(1029, ());

    let s2 = s1;

    assert!(s1.eq(&s2));
}

// check that default on enum doesn't require default on non-default objects
pub struct NonDefaultable {}

#[perfect_derive(Default)]
pub enum DefaultableEnum {
    E1(NonDefaultable),
    #[default]
    E2,
}

#[test]
pub fn defaultable_enum_is_default()
where
    DefaultableEnum: Default,
{
    // Nop
}
