macro_rules! make_test {
    ($trait_name:ident $(,$trait_name_tail:ident)*; $method_name:ident) => {
        mod $method_name {
            use perfect_derive::perfect_derive;

            #[perfect_derive($trait_name $(,$trait_name_tail)*)]
            pub struct Struct<U, T> {
                v1: T,
                pub v2: U,
            }

            #[perfect_derive($trait_name $(,$trait_name_tail)*)]
            pub enum Enum<U, T> {
                E1,
                E2(),
                E3(usize),
                E4(u32, (), U),
                E5{name1: T, name2: ()},
            }

            #[test]
            pub fn $method_name()
            where
                Struct<u32, usize>: $trait_name,
                Enum<i64, ()>: $trait_name,
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
