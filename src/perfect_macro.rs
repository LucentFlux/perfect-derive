use crate::perfect_parsing::{DerivedList, DerivedType, DerivedTypeEnum, StructOrEnum};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Where;
use syn::{
    Fields, GenericParam, Generics, ItemEnum, Lifetime, PredicateType, TypeParamBound, WhereClause,
    WherePredicate,
};

pub fn impl_traits(traits: DerivedList, obj: StructOrEnum) -> TokenStream {
    let mut output = quote! {
        #obj
    };

    for derived in traits.0 {
        add_type_impl(&mut output, derived, &obj);
    }

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

fn add_type_impl(output: &mut TokenStream, trait_to_impl: DerivedType, obj: &StructOrEnum) {
    let ident = obj.ident();
    let generics = obj.generics();
    let gen_names = extract_idents(&generics);
    let gen_lt = generics.lt_token;
    let gen_params = generics.params;
    let gen_gt = generics.gt_token;
    let gen_where = augment_where_clause(generics.where_clause, &trait_to_impl, obj);

    let trait_ident = trait_to_impl.ident();

    let trait_impl = gen_type_impl_body(trait_to_impl, obj);

    *output = quote! {
        #output

        impl #gen_lt #gen_params #gen_gt #trait_ident for #ident #gen_lt #(#gen_names),* #gen_gt #gen_where {
            #trait_impl
        }
    };
}

fn get_debug_enum_marker_index(enum_item: &ItemEnum) -> usize {
    let default_variants = enum_item
        .variants
        .iter()
        .enumerate()
        .filter(|(_, v)| {
            v.attrs.iter().any(|a| {
                // attribute is exactly `#[default]`
                a.path.segments.len() == 1
                    && a.path
                        .segments
                        .iter()
                        .all(|i| i.ident.to_string() == "default" && i.arguments.is_empty())
            })
        })
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
    return (*default_variants.first().unwrap()).0;
}

fn augment_where_clause(
    clause: Option<WhereClause>,
    trait_to_impl: &DerivedType,
    obj: &StructOrEnum,
) -> WhereClause {
    let extra = match (&trait_to_impl.name, obj) {
        (DerivedTypeEnum::Debug, StructOrEnum::Enum(e)) => {
            let index = get_debug_enum_marker_index(e);
            let variant = e.variants.iter().skip(index).next().unwrap();
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

fn gen_type_impl_body(trait_to_impl: DerivedType, obj: &StructOrEnum) -> TokenStream {
    match (trait_to_impl.name, obj) {
        (DerivedTypeEnum::Copy, _) => quote!(),
        (DerivedTypeEnum::Clone, StructOrEnum::Struct(s)) => match &s.fields {
            Fields::Named(names) => {
                let idents = names
                    .named
                    .iter()
                    .map(|f| f.ident.clone().unwrap())
                    .collect::<Vec<_>>();

                quote! {
                    fn clone(&self) -> Self {
                        let Self{ #(#idents),* } = self;
                        Self{ #(#idents : #idents.clone()),* }
                    }
                }
            }
            Fields::Unnamed(unnamed) => {
                let idents = unnamed
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, f)| Ident::new(&format! {"v{}", i}, f.ty.span()))
                    .collect::<Vec<_>>();

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
        },
        (DerivedTypeEnum::Clone, StructOrEnum::Enum(e)) => {
            let variant_cases = e
                .variants
                .iter()
                .map(|v| {
                    let ident = v.ident.clone();
                    match &v.fields {
                        Fields::Named(names) => {
                            let idents = names
                                .named
                                .iter()
                                .map(|f| f.ident.clone().unwrap())
                                .collect::<Vec<_>>();

                            quote! {
                                Self::#ident{#(#idents),*} => Self::#ident{#(#idents : #idents.clone()),*}
                            }
                        }
                        Fields::Unnamed(unnamed) => {
                            let idents = unnamed
                                .unnamed
                                .iter()
                                .enumerate()
                                .map(|(i, f)| Ident::new(&format! {"v{}", i}, f.ty.span()))
                                .collect::<Vec<_>>();

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
        (DerivedTypeEnum::PartialEq, StructOrEnum::Struct(s)) => unimplemented!(),
        (DerivedTypeEnum::PartialEq, StructOrEnum::Enum(e)) => unimplemented!(),
        (DerivedTypeEnum::Eq, _) => quote!(),
        (DerivedTypeEnum::Ord, StructOrEnum::Struct(s)) => unimplemented!(),
        (DerivedTypeEnum::Ord, StructOrEnum::Enum(e)) => unimplemented!(),
        (DerivedTypeEnum::PartialOrd, StructOrEnum::Struct(s)) => unimplemented!(),
        (DerivedTypeEnum::PartialOrd, StructOrEnum::Enum(e)) => unimplemented!(),
        (DerivedTypeEnum::Hash, StructOrEnum::Struct(s)) => unimplemented!(),
        (DerivedTypeEnum::Hash, StructOrEnum::Enum(e)) => unimplemented!(),
        (DerivedTypeEnum::Debug, StructOrEnum::Struct(s)) => unimplemented!(),
        (DerivedTypeEnum::Debug, StructOrEnum::Enum(e)) => unimplemented!(),
        (DerivedTypeEnum::Default, StructOrEnum::Struct(s)) => unimplemented!(),
        (DerivedTypeEnum::Default, StructOrEnum::Enum(e)) => unimplemented!(),
    }
}
