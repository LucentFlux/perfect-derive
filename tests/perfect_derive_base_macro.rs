use perfect_derive::perfect_derive;
macro_rules! make_test {
    ($trait_name:ident $(,$trait_name_tail:ident)*; $method_name:ident) => {
        mod $method_name {
            use perfect_derive::perfect_derive;

            #[perfect_derive($trait_name $(,$trait_name_tail)*)]
            pub struct Struct {
                v1: usize,
                pub v2: i32,
            }

            #[perfect_derive($trait_name $(,$trait_name_tail)*)]
            pub enum Enum {
                E1,
                E2(),
                E3(usize),
                E4(u32, ()),
                E5{name1: u32, name2: ()},
            }

            #[test]
            pub fn $method_name()
            where
                Struct: $trait_name,
                Enum: $trait_name,
            {
                // No need to do anything
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

#[perfect_derive(Copy, Clone, Ord, Eq, PartialOrd, PartialEq, Debug, Hash, Default)]
struct EverythingStruct {
    v1: usize,
    pub v2: i32,
}

#[perfect_derive(Copy, Clone, Ord, Eq, PartialOrd, PartialEq, Debug, Hash, Default)]
enum EverythingEnum {
    E1,
    E2(),
    E3(usize),
    E4(u32, ()),
    #[default]
    E5 {
        name1: u32,
        name2: (),
    },
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
    E2(usize),
}

#[test]
pub fn defaultable_enum_is_default()
where
    DefaultableEnum: Default,
{
    // Nop
}
