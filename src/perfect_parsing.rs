use crate::impls::impls;
use proc_macro2::Span;
use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Generics, ItemEnum, ItemStruct, Path, Token, TraitBound, TraitBoundModifier};

#[cps::cps]
macro_rules! supported_types_enum {

    ($p:vis enum $name:ident) =>
    let $($val:ident,)* = impls!() in
    {
        $p enum $name {
            $($val,)*
        }
    };
}

supported_types_enum!(pub enum DerivedTypeEnum);

pub struct DerivedType {
    pub name: DerivedTypeEnum,
    pub span: Span,
}

#[cps::cps]
macro_rules! ident_enum {
    (pub fn $fn_ident:ident(&self) -> $ret:ident;) =>
    let $($val:ident,)* = impls!() in
    {
        pub fn $fn_ident(&self) -> $ret {
            match self.name {
                $(
                    DerivedTypeEnum::$val => Ident::new(stringify!($val), self.span.into())
                ),*
            }
        }
    };
}

impl DerivedType {
    ident_enum!(
        pub fn ident(&self) -> Ident;
    );

    pub fn get_trait(&self) -> TraitBound {
        let ident = self.ident();
        TraitBound {
            paren_token: None,
            modifier: TraitBoundModifier::None,
            lifetimes: None,
            path: Path::from(ident),
        }
    }
}

#[cps::cps]
macro_rules! parse_types_enum {
    (match $e:ident {
        $ident:ident...,
        _ => $err:expr
    }) =>
    let $($type_name:tt,)* = impls!() in
    {
        match $e {
            $(stringify!($type_name) => Ok(Self {
                name: DerivedTypeEnum::$type_name,
                span: $ident.span()
            }),)*
            _ => $err
        }
    };
}

impl Parse for DerivedType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let binding = ident.to_string();
        let name = binding.as_ref();

        parse_types_enum! {
            match name {
                ident...,
                _ => Err(input.error(format!("type identifier {} is not supported - did you mean to use #[derive(...)]?", ident.to_string())))
            }
        }
    }
}

pub struct DerivedList(pub Punctuated<DerivedType, Token![,]>);

impl Parse for DerivedList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(Punctuated::parse_terminated(input)?))
    }
}

pub enum StructOrEnum {
    Struct(ItemStruct),
    Enum(ItemEnum),
}

impl StructOrEnum {
    pub fn ident(&self) -> Ident {
        match self {
            StructOrEnum::Struct(s) => s.ident.clone(),
            StructOrEnum::Enum(e) => e.ident.clone(),
        }
    }

    pub fn generics(&self) -> Generics {
        match self {
            StructOrEnum::Struct(s) => s.generics.clone(),
            StructOrEnum::Enum(e) => e.generics.clone(),
        }
    }
}

impl Parse for StructOrEnum {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let enum_fork = input.fork();
        let enum_parse = enum_fork.parse::<ItemEnum>();
        if let Ok(enum_val) = enum_parse {
            input.advance_to(&enum_fork);
            Ok(Self::Enum(enum_val))
        } else {
            Ok(Self::Struct(input.parse()?))
        }
    }
}

impl ToTokens for StructOrEnum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            StructOrEnum::Struct(s) => s.to_tokens(tokens),
            StructOrEnum::Enum(e) => e.to_tokens(tokens),
        }
    }
}
