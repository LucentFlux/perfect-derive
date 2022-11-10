macro_rules! make_test {
    ($trait_name:ident $(,$trait_name_tail:ident)*; $method_name:ident) => {
        mod $trait_name {
            use perfect_derive::perfect_derive;

            #[perfect_derive($trait_name $(,$trait_name_tail)*)]
            struct Struct {
                v1: usize,
                pub v2: i32,
            }

            #[perfect_derive($trait_name $(,$trait_name_tail)*)]
            enum Enum {
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
