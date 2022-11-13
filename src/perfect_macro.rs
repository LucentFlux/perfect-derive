use crate::perfect_parsing::{DerivedList, DerivedType, DerivedTypeEnum, StructOrEnum};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use std::collections::HashSet;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Where;
use syn::{
    AttrStyle, Attribute, Fields, FieldsNamed, FieldsUnnamed, GenericParam, Generics, ItemEnum,
    ItemStruct, Lifetime, PredicateType, TypeParamBound, Variant, WhereClause, WherePredicate,
};

fn is_attribute_default(a: &Attribute) -> bool {
    match a.style {
        AttrStyle::Inner(_) => return false,
        AttrStyle::Outer => {}
    }

    // attribute is exactly `#[default]`
    a.path.leading_colon.is_none()
        && a.path.segments.len() == 1
        && a.path
            .segments
            .iter()
            .all(|i| i.ident.to_string() == "default" && i.arguments.is_empty())
}

fn remove_debug_markers(obj: &mut StructOrEnum) {
    if let StructOrEnum::Enum(e) = obj {
        for v in e.variants.iter_mut() {
            let default_attr_index = v
                .attrs
                .iter()
                .enumerate()
                .filter_map(|(i, a)| {
                    if is_attribute_default(a) {
                        Some(i)
                    } else {
                        None
                    }
                })
                .next();
            if let Some(i) = default_attr_index {
                v.attrs.remove(i);
            }
        }
    }
}

pub fn impl_traits(traits: DerivedList, mut obj: StructOrEnum) -> TokenStream {
    let mut output = quote! {};

    let mut already_derived = HashSet::new();
    for derived in traits.0 {
        if already_derived.contains(&derived.name) {
            panic!("cannot derive {:?} twice", derived.name)
        }
        already_derived.insert(derived.name.clone());

        add_type_impl(&mut output, &derived, &obj);
    }

    // If we derived Default, we need to remove any default markers from enums
    if already_derived.contains(&DerivedTypeEnum::Default) {
        remove_debug_markers(&mut obj);
    }

    output = quote! {
        #obj

        #output
    };

    return output;
}

enum IdentOrLifetime {
    Ident(Ident),
    Lifetime(Lifetime),
}

impl ToTokens for IdentOrLifetime {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            IdentOrLifetime::Ident(i) => i.to_tokens(tokens),
            IdentOrLifetime::Lifetime(l) => l.to_tokens(tokens),
        }
    }
}

fn extract_idents(generics: &Generics) -> Vec<IdentOrLifetime> {
    generics
        .params
        .iter()
        .map(|gp| match gp {
            GenericParam::Type(ty) => IdentOrLifetime::Ident(ty.ident.clone()),
            GenericParam::Lifetime(lt) => IdentOrLifetime::Lifetime(lt.lifetime.clone()),
            GenericParam::Const(cst) => IdentOrLifetime::Ident(cst.ident.clone()),
        })
        .collect()
}

fn add_type_impl(output: &mut TokenStream, trait_to_impl: &DerivedType, obj: &StructOrEnum) {
    let ident = obj.ident();
    let generics = obj.generics();
    let gen_names = extract_idents(&generics);
    let gen_lt = generics.lt_token;
    let gen_params = generics.params;
    let gen_gt = generics.gt_token;
    let gen_where = augment_where_clause(generics.where_clause, trait_to_impl, obj);

    let trait_ident = trait_to_impl.path();

    let trait_impl = gen_type_impl_body(trait_to_impl, obj);

    *output = quote! {
        #output

        impl #gen_lt #gen_params #gen_gt #trait_ident for #ident #gen_lt #(#gen_names),* #gen_gt #gen_where {
            #trait_impl
        }
    };
}

fn get_debug_enum_marker(enum_item: &ItemEnum) -> &Variant {
    let default_variants = enum_item
        .variants
        .iter()
        .filter(|v| v.attrs.iter().any(is_attribute_default))
        .collect::<Vec<_>>();
    assert!(
        default_variants.len() > 0,
        "one enum variant must be marked as default"
    );
    assert_eq!(
        default_variants.len(),
        1,
        "only one enum variant may be marked as default"
    );
    return default_variants.first().unwrap();
}

fn augment_where_clause(
    clause: Option<WhereClause>,
    trait_to_impl: &DerivedType,
    obj: &StructOrEnum,
) -> WhereClause {
    let extra = match (&trait_to_impl.name, obj) {
        (DerivedTypeEnum::Default, StructOrEnum::Enum(e)) => {
            let variant = get_debug_enum_marker(e);
            variant
                .fields
                .iter()
                .map(|f| f.ty.clone())
                .collect::<Vec<_>>()
        }
        (_, StructOrEnum::Struct(s)) => s.fields.iter().map(|f| f.ty.clone()).collect(),
        (_, StructOrEnum::Enum(e)) => e
            .variants
            .iter()
            .flat_map(|v| v.fields.iter().map(|f| f.ty.clone()))
            .collect(),
    };

    let mut bounds = Punctuated::new();
    bounds.push(TypeParamBound::Trait(trait_to_impl.get_trait()));
    let extra = extra.into_iter().map(|bounded_ty| {
        WherePredicate::Type(PredicateType {
            lifetimes: None,
            bounded_ty,
            colon_token: Default::default(),
            bounds: bounds.clone(),
        })
    });

    let mut predicates = clause
        .as_ref()
        .map(|c| c.predicates.clone())
        .unwrap_or(Punctuated::new());
    for predicate in extra {
        predicates.push(predicate)
    }

    return WhereClause {
        where_token: clause.map(|c| c.where_token).unwrap_or(Where {
            span: trait_to_impl.span,
        }),
        predicates,
    };
}

fn get_named_idents(names: &FieldsNamed) -> Vec<Ident> {
    get_named_idents_prefix(names, "")
}

fn get_named_idents_prefix(names: &FieldsNamed, prefix: &str) -> Vec<Ident> {
    names
        .named
        .iter()
        .map(|f| {
            let ident = f.ident.clone().unwrap();
            let new_name = format!("{}{}", prefix, ident.to_string());
            Ident::new(&new_name, ident.span())
        })
        .collect::<Vec<_>>()
}

fn get_unnamed_idents(unnamed: &FieldsUnnamed) -> Vec<Ident> {
    get_unnamed_idents_prefix(unnamed, "v")
}

fn get_unnamed_idents_prefix(unnamed: &FieldsUnnamed, prefix: &str) -> Vec<Ident> {
    unnamed
        .unnamed
        .iter()
        .enumerate()
        .map(|(i, f)| Ident::new(&format! {"{}{}", prefix, i}, f.ty.span()))
        .collect::<Vec<_>>()
}

fn gen_type_impl_body(trait_to_impl: &DerivedType, obj: &StructOrEnum) -> TokenStream {
    match (&trait_to_impl.name, obj) {
        (DerivedTypeEnum::Copy, _) => quote!(),
        (DerivedTypeEnum::Eq, _) => quote!(),
        (DerivedTypeEnum::Clone, StructOrEnum::Struct(s)) => clone_struct(s),
        (DerivedTypeEnum::Clone, StructOrEnum::Enum(e)) => clone_enum(e),
        (DerivedTypeEnum::PartialEq, StructOrEnum::Struct(s)) => peq_struct(s),
        (DerivedTypeEnum::PartialEq, StructOrEnum::Enum(e)) => peq_enum(e),
        (DerivedTypeEnum::Ord, StructOrEnum::Struct(s)) => ord_struct(s),
        (DerivedTypeEnum::Ord, StructOrEnum::Enum(e)) => ord_enum(e),
        (DerivedTypeEnum::PartialOrd, StructOrEnum::Struct(s)) => pord_struct(s),
        (DerivedTypeEnum::PartialOrd, StructOrEnum::Enum(e)) => pord_enum(e),
        (DerivedTypeEnum::Hash, StructOrEnum::Struct(s)) => hash_struct(s),
        (DerivedTypeEnum::Hash, StructOrEnum::Enum(e)) => hash_enum(e),
        (DerivedTypeEnum::Debug, StructOrEnum::Struct(s)) => debug_struct(s),
        (DerivedTypeEnum::Debug, StructOrEnum::Enum(e)) => debug_enum(e),
        (DerivedTypeEnum::Default, StructOrEnum::Struct(s)) => default_struct(s),
        (DerivedTypeEnum::Default, StructOrEnum::Enum(e)) => default_enum(e),
    }
}

fn clone_struct(s: &ItemStruct) -> TokenStream {
    match &s.fields {
        Fields::Named(names) => {
            let idents = get_named_idents(names);

            quote! {
                fn clone(&self) -> Self {
                    let Self{ #(#idents),* } = self;
                    Self{ #(#idents : #idents.clone()),* }
                }
            }
        }
        Fields::Unnamed(unnamed) => {
            let idents = get_unnamed_idents(unnamed);

            quote! {
                fn clone(&self) -> Self {
                    let Self( #(#idents),* ) = self;
                    Self( #(#idents.clone()),* )
                }
            }
        }
        Fields::Unit => quote! {
            fn clone(&self) -> Self {
                Self
            }
        },
    }
}

fn clone_enum(e: &ItemEnum) -> TokenStream {
    let variant_cases = e
        .variants
        .iter()
        .map(|v| {
            let ident = v.ident.clone();
            match &v.fields {
                Fields::Named(names) => {
                    let idents = get_named_idents(names);

                    quote! {
                        Self::#ident{#(#idents),*} => Self::#ident{#(#idents : #idents.clone()),*}
                    }
                }
                Fields::Unnamed(unnamed) => {
                    let idents = get_unnamed_idents(unnamed);

                    quote! {
                        Self::#ident(#(#idents),*) => Self::#ident(#(#idents.clone()),*)
                    }
                }
                Fields::Unit => quote! {
                    Self::#ident => Self::#ident
                },
            }
        })
        .collect::<Vec<_>>();
    quote! {
        fn clone(&self) -> Self {
            match self {
                #(
                    #variant_cases
                ),*
            }
        }
    }
}

fn peq_struct(s: &ItemStruct) -> TokenStream {
    match &s.fields {
        Fields::Named(names) => {
            let idents = get_named_idents(names);

            quote! {
                fn eq(&self, other: &Self) -> bool {
                    true #(&& self.#idents.eq(& other.#idents))*
                }
            }
        }
        Fields::Unnamed(unnamed) => {
            let idents1 = get_unnamed_idents_prefix(unnamed, "u");
            let idents2 = get_unnamed_idents_prefix(unnamed, "v");

            quote! {
                fn eq(&self, other: &Self) -> bool {
                    let Self( #(#idents1),* ) = self;
                    let Self( #(#idents2),* ) = other;
                    true #(&& #idents1.eq(#idents2))*
                }
            }
        }
        Fields::Unit => quote! {
            fn eq(&self, other: &Self) -> bool {
                true
            }
        },
    }
}

fn peq_enum(e: &ItemEnum) -> TokenStream {
    let variant_cases = e
        .variants
        .iter()
        .map(|v| {
            let ident = v.ident.clone();
            match &v.fields {
                Fields::Named(names) => {
                    let idents = get_named_idents(names);
                    let idents1 = get_named_idents_prefix(names, "u");
                    let idents2 = get_named_idents_prefix(names, "v");

                    quote! {
                        (Self::#ident{#(#idents: #idents1),*}, Self::#ident{#(#idents: #idents2),*})
                            => true #(&& #idents1.eq(#idents2))*
                    }
                }
                Fields::Unnamed(unnamed) => {
                    let idents1 = get_unnamed_idents_prefix(unnamed, "u");
                    let idents2 = get_unnamed_idents_prefix(unnamed, "v");

                    quote! {
                        (Self::#ident(#(#idents1),*), Self::#ident(#(#idents2),*))
                            => true #(&& #idents1.eq(#idents2))*
                    }
                }
                Fields::Unit => quote! {
                    (Self::#ident, Self::#ident) => true
                },
            }
        })
        .collect::<Vec<_>>();
    quote! {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                #(
                    #variant_cases,
                )*
                _ => false
            }
        }
    }
}

fn ord_struct(s: &ItemStruct) -> TokenStream {
    match &s.fields {
        Fields::Named(names) => {
            let idents = get_named_idents(names);

            quote! {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    std::cmp::Ordering::Equal #(.then(self.#idents.cmp(&other.#idents)))*
                }
            }
        }
        Fields::Unnamed(unnamed) => {
            let idents1 = get_unnamed_idents_prefix(unnamed, "u");
            let idents2 = get_unnamed_idents_prefix(unnamed, "v");

            quote! {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    let Self( #(#idents1),* ) = self;
                    let Self( #(#idents2),* ) = other;
                    std::cmp::Ordering::Equal #(.then(#idents1.cmp(#idents2)))*
                }
            }
        }
        Fields::Unit => quote! {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                std::cmp::Ordering::Equal
            }
        },
    }
}

fn enum_cmp_lexographic(e: &ItemEnum) -> TokenStream {
    let variant_cases = e
        .variants
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let ident = v.ident.clone();
            let match_vars = match &v.fields {
                Fields::Named(_) => {
                    quote! {
                        { .. }
                    }
                }
                Fields::Unnamed(unnamed) => {
                    let empty_ident = Ident::new("_", v.span());
                    let blanks = unnamed.unnamed.iter().map(|_| empty_ident.clone());

                    quote! {
                        ( #(#blanks),* )
                    }
                }
                Fields::Unit => quote! {},
            };

            quote! {
                Self::#ident #match_vars => #i
            }
        })
        .collect::<Vec<_>>();
    quote! {
        {
            let i1 = match self {
                #(#variant_cases),*
            };
            let i2 = match other {
                #(#variant_cases),*
            };
            i1.cmp(&i2)
        }
    }
}

fn ord_enum(e: &ItemEnum) -> TokenStream {
    let variant_cases = e
        .variants
        .iter()
        .map(|v| {
            let ident = v.ident.clone();
            match &v.fields {
                Fields::Named(names) => {
                    let idents = get_named_idents(names);
                    let idents1 = get_named_idents_prefix(names, "u");
                    let idents2 = get_named_idents_prefix(names, "v");

                    quote! {
                        (Self::#ident{#(#idents: #idents1),*}, Self::#ident{#(#idents: #idents2),*})
                            => std::cmp::Ordering::Equal #(.then(#idents1.cmp(#idents2)))*
                    }
                }
                Fields::Unnamed(unnamed) => {
                    let idents1 = get_unnamed_idents_prefix(unnamed, "u");
                    let idents2 = get_unnamed_idents_prefix(unnamed, "v");

                    quote! {
                        (Self::#ident(#(#idents1),*), Self::#ident(#(#idents2),*))
                            => std::cmp::Ordering::Equal #(.then(#idents1.cmp(#idents2)))*
                    }
                }
                Fields::Unit => quote! {
                    (Self::#ident, Self::#ident) => std::cmp::Ordering::Equal
                },
            }
        })
        .collect::<Vec<_>>();

    let base_case = enum_cmp_lexographic(e);
    quote! {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            match (self, other) {
                #(
                    #variant_cases,
                )*
                _ => #base_case
            }
        }
    }
}

fn pord_struct(s: &ItemStruct) -> TokenStream {
    match &s.fields {
        Fields::Named(names) => {
            let idents = get_named_idents(names);

            quote! {
                fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                    Some(std::cmp::Ordering::Equal) #(
                        .and_then(|o| self.#idents.partial_cmp(&other.#idents).map(|v| v.then(o)))
                    )*
                }
            }
        }
        Fields::Unnamed(unnamed) => {
            let idents1 = get_unnamed_idents_prefix(unnamed, "u");
            let idents2 = get_unnamed_idents_prefix(unnamed, "v");

            quote! {
                fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                    let Self( #(#idents1),* ) = self;
                    let Self( #(#idents2),* ) = other;

                    Some(std::cmp::Ordering::Equal) #(
                        .and_then(|o| #idents1.partial_cmp(#idents2).map(|v| v.then(o)))
                    )*
                }
            }
        }
        Fields::Unit => quote! {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(std::cmp::Ordering::Equal)
            }
        },
    }
}

fn pord_enum(e: &ItemEnum) -> TokenStream {
    let variant_cases = e
        .variants
        .iter()
        .map(|v| {
            let ident = v.ident.clone();
            match &v.fields {
                Fields::Named(names) => {
                    let idents = get_named_idents(names);
                    let idents1 = get_named_idents_prefix(names, "u");
                    let idents2 = get_named_idents_prefix(names, "v");

                    quote! {
                        (Self::#ident{#(#idents: #idents1),*}, Self::#ident{#(#idents: #idents2),*})
                            => Some(std::cmp::Ordering::Equal) #(
                                .and_then(|o| #idents1.partial_cmp(#idents2).map(|v| v.then(o)))
                            )*
                    }
                }
                Fields::Unnamed(unnamed) => {
                    let idents1 = get_unnamed_idents_prefix(unnamed, "u");
                    let idents2 = get_unnamed_idents_prefix(unnamed, "v");

                    quote! {
                        (Self::#ident(#(#idents1),*), Self::#ident(#(#idents2),*))
                            => Some(std::cmp::Ordering::Equal) #(
                                .and_then(|o| #idents1.partial_cmp(#idents2).map(|v| v.then(o)))
                            )*
                    }
                }
                Fields::Unit => quote! {
                    (Self::#ident, Self::#ident) => Some(std::cmp::Ordering::Equal)
                },
            }
        })
        .collect::<Vec<_>>();

    let base_case = enum_cmp_lexographic(e);
    quote! {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            match (self, other) {
                #(
                    #variant_cases,
                )*
                _ => Some(#base_case)
            }
        }
    }
}

fn hash_struct(s: &ItemStruct) -> TokenStream {
    match &s.fields {
        Fields::Named(names) => {
            let idents = get_named_idents(names);

            quote! {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    #(
                        self.#idents.hash(state);
                    )*
                }
            }
        }
        Fields::Unnamed(unnamed) => {
            let idents = get_unnamed_idents(unnamed);

            quote! {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    let Self( #(#idents),* ) = self;
                    #(
                        #idents.hash(state);
                    )*
                }
            }
        }
        Fields::Unit => quote! {
            fn hash<H: std::hash::Hasher>(&self, _: &mut H) { }
        },
    }
}

fn hash_enum(e: &ItemEnum) -> TokenStream {
    let variant_cases = e
        .variants
        .iter()
        .map(|v| {
            let ident = v.ident.clone();
            match &v.fields {
                Fields::Named(names) => {
                    let idents = get_named_idents(names);

                    quote! {
                        Self::#ident{#(#idents),*}
                            => {
                                #( #idents.hash(state); )*
                            }
                    }
                }
                Fields::Unnamed(unnamed) => {
                    let idents = get_unnamed_idents(unnamed);

                    quote! {
                        Self::#ident(#(#idents),*)
                            => {
                                #( #idents.hash(state); )*
                            }
                    }
                }
                Fields::Unit => quote! {
                    Self::#ident => {}
                },
            }
        })
        .collect::<Vec<_>>();

    quote! {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            match self {
                #(
                    #variant_cases,
                )*
            }
        }
    }
}

fn debug_struct(s: &ItemStruct) -> TokenStream {
    let name = s.ident.clone();
    match &s.fields {
        Fields::Named(names) => {
            let idents = get_named_idents(names);

            quote! {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.debug_struct(stringify!(#name))
                    #(
                        .field(stringify!(#idents), &self.#idents)
                    )*
                        .finish()
                }
            }
        }
        Fields::Unnamed(unnamed) => {
            let idents = get_unnamed_idents(unnamed);

            quote! {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.debug_tuple(stringify!(#name))
                    #(
                        .field(&self.#idents)
                    )*
                        .finish()
                }
            }
        }
        Fields::Unit => quote! {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple(stringify!(#name)).finish()
            }
        },
    }
}

fn debug_enum(e: &ItemEnum) -> TokenStream {
    let name = e.ident.clone();
    let variant_cases = e
        .variants
        .iter()
        .map(|v| {
            let ident = v.ident.clone();
            let name = quote! { concat!(stringify!(#name), "::", stringify!(#ident)) };
            match &v.fields {
                Fields::Named(names) => {
                    let idents = get_named_idents(names);

                    quote! {
                        Self::#ident{#(#idents),*}
                            => f.debug_struct(#name)
                                #(
                                    .field(stringify!(#idents), #idents)
                                )*
                                    .finish()
                    }
                }
                Fields::Unnamed(unnamed) => {
                    let idents = get_unnamed_idents(unnamed);

                    quote! {
                        Self::#ident(#(#idents),*)
                            => f.debug_tuple(#name)
                                #(
                                    .field(#idents)
                                )*
                                    .finish()
                    }
                }
                Fields::Unit => quote! {
                    Self::#ident => f.debug_tuple(#name).finish()
                },
            }
        })
        .collect::<Vec<_>>();

    quote! {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                #(
                    #variant_cases,
                )*
            }
        }
    }
}

fn default_inner(fields: &Fields, root: TokenStream) -> TokenStream {
    match &fields {
        Fields::Named(names) => {
            let idents = get_named_idents(names);

            quote! {
                fn default() -> Self {
                    #root {
                        #(
                            #idents : Default::default(),
                        )*
                    }
                }
            }
        }
        Fields::Unnamed(unnamed) => {
            let idents = get_unnamed_idents(unnamed);
            let defaults = idents.into_iter().map(|_| quote! {Default::default()});

            quote! {
                fn default() -> Self {
                    #root (
                        #(
                            #defaults
                        ),*
                    )
                }
            }
        }
        Fields::Unit => quote! {
            fn default() -> Self {
                #root
            }
        },
    }
}

fn default_struct(s: &ItemStruct) -> TokenStream {
    default_inner(&s.fields, quote! { Self })
}

fn default_enum(e: &ItemEnum) -> TokenStream {
    let default_variant = get_debug_enum_marker(e);
    let default_ident = default_variant.ident.clone();

    default_inner(&default_variant.fields, quote! { Self::#default_ident })
}
