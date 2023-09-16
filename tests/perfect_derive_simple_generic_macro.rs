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
            pub struct Struct2<U, T> (
                T,
                pub(super) U,
            );

            #[perfect_derive($trait_name $(,$trait_name_tail)*)]
            pub struct Struct3<T, U: Clone> {
                v1: T,
                v2: U
            }

            #[perfect_derive($trait_name $(,$trait_name_tail)*)]
            pub struct Struct4<'a, 'b: 'a, T> {
                v1: T,
                v2: std::marker::PhantomData<&'a mut &'b T>
            }

            #[perfect_derive($trait_name $(,$trait_name_tail)*)]
            pub struct Struct5<
                #[allow(unused)]
                'a,
                #[allow(unused)]
                T,
                #[allow(unused)]
                const VALUE: usize,
            > {
                #[allow(unused)]
                phantom: std::marker::PhantomData<&'a T>,
            }

            #[perfect_derive($trait_name $(,$trait_name_tail)*)]
            pub enum Enum<U, T> {
                E1,
                E2(),
                E3(usize),
                E4(u32, (), U),
                E5{name1: T, name2: ()},
            }

            #[allow(unused_imports)]
            mod inner {
                use super::*;
                use std::fmt::Debug;
                use std::hash::Hash;

                #[test]
                pub fn $method_name()
                where
                    Struct<u32, usize>: $trait_name,
                    Struct2<u8, isize>: $trait_name,
                    Struct3<u64, u64>: $trait_name,
                    Struct4<'static, 'static, isize>: $trait_name,
                    Struct5<'static, isize, 12>: $trait_name,
                    Enum<i64, ()>: $trait_name,
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
