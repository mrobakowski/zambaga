use proc_macro::TokenStream as PMTokenStream;
use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};

mod mode;

#[proc_macro_attribute]
pub fn derive(_attr: PMTokenStream, item: PMTokenStream) -> PMTokenStream {
    derive_impl(_attr.into(), item.into()).into()
}

fn derive_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let trait_name = attr.into_iter().next().unwrap();
    enum Adt {
        Struct(syn::ItemStruct),
        Enum(syn::ItemEnum),
    }

    let adt: Adt = if let Ok(item) = syn::parse2::<syn::ItemStruct>(item.clone()) {
        Adt::Struct(item)
    } else if let Ok(item) = syn::parse2::<syn::ItemEnum>(item.clone()) {
        Adt::Enum(item)
    } else {
        panic!("Expected struct or enum");
    };

    let item = item.into_iter().collect::<Vec<_>>();
    let name = item[1].clone();
    let name_lit = name.to_string();

    let trait_macro_invocation = quote! {
        #trait_name!(#name);
    };

    let with_mirror_impl = match adt {
        Adt::Struct(item) => {
            let field_acceptors = item.fields.iter().enumerate().map(|(i, field)| {
                let field_accessor = field
                    .ident
                    .as_ref()
                    .map(|x| x.to_token_stream())
                    .unwrap_or_else(|| syn::Index::from(i).to_token_stream());
                let field_name_value = match &field.ident {
                    Some(ident) => {
                        let ident = ident.to_string();
                        quote! { Some(FieldName(#ident)) }
                    }
                    None => quote! { None },
                };
                let ty = &field.ty;
                let ty_lit = ty.to_token_stream().to_string();
                quote! {
                    <#trait_name!(@MDT) as MakeDynTrait>::IsTrait::<'_, #ty>
                        ::VISITOR_ACCEPTOR.accept(
                            &this_value.#field_accessor,
                            #field_name_value,
                            TypeName::from_source::<#ty>(#ty_lit),
                            visitor,
                        );
                }
            });

            let field_metas = item.fields.iter().enumerate().map(|(i, field)| {
                let ty = &field.ty;
                let ty_lit = ty.to_token_stream().to_string();
                let field_name_lit = field.ident.as_ref().map(|x| x.to_string()).unwrap_or_else(|| i.to_string());
                let field_accessor = field
                    .ident
                    .as_ref()
                    .map(|x| x.to_token_stream())
                    .unwrap_or_else(|| syn::Index::from(i).to_token_stream());

                quote! {
                    (
                        FieldName(#field_name_lit),
                        TypeName::from_source::<#ty>(#ty_lit),
                        FieldExtractor::new({
                            use ::std::any::Any;
                            fn __zambaga_extractor(this_value: &dyn Any) -> Result<&dyn Any, TypeError> {
                                let value = this_value.downcast_ref::<#name>().ok_or(TypeError)?;
                                Ok(&value.#field_accessor)
                            }
                            __zambaga_extractor
                        }),
                        ImplExtractor::<#trait_name!(@MDT)> {
                            extractor_fn: {
                                use ::std::any::Any;
                                fn __zambaga_extractor(
                                    field_value: &dyn Any,
                                ) -> Result<Option<&<#trait_name!(@MDT) as MakeDynTrait>::DynTrait<'_>>, TypeError> {
                                    let value = field_value.downcast_ref::<#ty>().ok_or(TypeError)?;
                                    Ok(<#trait_name!(@MDT) as MakeDynTrait>::IsTrait::new(value).as_dyn())
                                }

                                __zambaga_extractor
                            },
                            has_impl: {
                                <#trait_name!(@MDT) as MakeDynTrait>::IsTrait::<'_, #ty>::DOES_IMPLEMENT
                            },
                        },
                    )
                }
            });

            quote! {
                impl WithMirror<#trait_name!(@MDT)> for #name {
                    const MIRROR: Mirror<#trait_name!(@MDT)> = {
                        Mirror {
                            name: TypeName::from_source::<#name>(#name_lit),
                            field_traverser: {
                                fn __zambaga_visitor(this_value: &#name, visitor: &mut <#trait_name!(@MDT) as MakeDynTrait>::FieldVisitor) {
                                    #(#field_acceptors)*
                                }
                                FieldTraverser::new(__zambaga_visitor)
                            },
                            fields_or_variants: FieldsOrVariants::Struct {
                                fields: &[
                                    #(#field_metas,)*
                                ],
                            },
                        }
                    };
                }
            }
        }
        Adt::Enum(_item) => todo!(),
    };

    let res = quote! {
        #(#item)*

        #trait_name! { @uses;
            #trait_macro_invocation
            #with_mirror_impl
        }
    };

    // println!("{}", res);

    res
}

#[proc_macro_attribute]
pub fn reflect(_attr: PMTokenStream, item: PMTokenStream) -> PMTokenStream {
    reflect_impl(_attr.into(), item.into()).into()
}

macro_rules! make_ident {
    ($($arg:tt)*) => {
        TokenTree::Ident(Ident::new(&format!($($arg)*), Span::call_site()))
    };
}

fn reflect_impl(_attributes: TokenStream, item: TokenStream) -> TokenStream {
    let input: syn::ItemTrait = syn::parse2(item).unwrap();
    let trait_name = input.ident.clone();
    let trait_name_str_literal = trait_name.to_string();
    let trait_name_snake_case = heck::AsSnakeCase(trait_name.to_string());

    let visit_field_trait_name = make_ident!("Visit{}Field", trait_name);
    let derive_trait_name = make_ident!("Derive{}", trait_name);
    let mdt_struct_name = make_ident!("MDT{}", trait_name);
    let is_trait_struct_name = make_ident!("Is{}", trait_name);
    let impl_make_dyn_trait_macro_name = make_ident!("{}_field_visitor", trait_name_snake_case);
    let impl_module_name = make_ident!("__zambaga_{}_impl", trait_name_snake_case);
    let impl_trait_macro_name = make_ident!("{}Macro", trait_name);

    let mdt = quote! {
        pub struct #mdt_struct_name;

        macro_rules! #impl_make_dyn_trait_macro_name {
            ($visitor:path) => {
                impl ::zambaga::MakeDynTrait for #impl_module_name::#mdt_struct_name {
                    type DynTrait<'a> = dyn #trait_name + 'a;
                    type IsTrait<'a, T: 'a> = #impl_module_name::#is_trait_struct_name<'a, T>;
                    type FieldVisitor = $visitor;
                    const TRAIT_NAME: &'static str = #trait_name_str_literal;
                }
            };
        }
        pub(crate) use #impl_make_dyn_trait_macro_name;
    };

    let is_trait_struct = quote! {
        pub struct #is_trait_struct_name<'a, T>(pub &'a T);

        impl<'a, T> #is_trait_struct_name<'a, T> {
            pub const fn new(value: &'a T) -> Self {
                Self(value)
            }
        }

        pub trait DefaultCase<'a> {
            fn as_dyn(&self) -> Option<&'a dyn #trait_name> {
                None
            }
            const DOES_IMPLEMENT: bool = false;
            const VISITOR_ACCEPTOR: AnyVisitorAcceptor = AnyVisitorAcceptor;
        }

        impl<'a, T> DefaultCase<'a> for #is_trait_struct_name<'a, T> {}

        impl<'a, T: #trait_name> #is_trait_struct_name<'a, T> {
            pub fn as_dyn(&self) -> Option<&'a dyn #trait_name> {
                Some(self.0 as &dyn #trait_name)
            }

            pub const DOES_IMPLEMENT: bool = true;

            pub const VISITOR_ACCEPTOR: ImplementedVisitorAcceptor = ImplementedVisitorAcceptor;
        }

        pub struct AnyVisitorAcceptor;

        impl AnyVisitorAcceptor {
            pub fn accept<Visitor: #visit_field_trait_name>(
                self,
                field_value: &dyn Any,
                field_name: Option<FieldName>,
                field_type: TypeName,
                visitor: &mut Visitor,
            ) {
                visitor.visit_any(field_value, field_name, field_type);
            }
        }

        pub struct ImplementedVisitorAcceptor;

        impl ImplementedVisitorAcceptor {
            pub fn accept<T: #trait_name, Visitor: #visit_field_trait_name>(
                self,
                field_value: &T,
                field_name: Option<FieldName>,
                field_type: TypeName,
                visitor: &mut Visitor,
            ) {
                visitor.visit_implemented(field_value, field_name, field_type);
            }
        }
    };

    let visit_field_trait = quote! {
        pub trait #visit_field_trait_name {
            fn visit_implemented<T: #trait_name>(
                &mut self,
                field_value: &T,
                field_name: Option<FieldName>,
                field_type: TypeName,
            );

            fn visit_any(
                &mut self,
                _field_value: &dyn Any,
                _field_name: Option<FieldName>,
                _field_type: TypeName,
            ) {
                panic!(
                    "This should not be called. If this is being called it means that you turned off \
                    the validator but didn't override the `visit_any` method."
                );
            }
        }
    };

    let trait_item_declarations = input.items.clone();

    let derive_trait = quote! {
        pub trait #derive_trait_name: WithMirror<#mdt_struct_name> {
            const VALIDATION: Validation = Validation::all_fields_impl(&Self::MIRROR);

            #(#trait_item_declarations)*

            fn visit_fields(
                &self,
                mut visitor: <#mdt_struct_name as MakeDynTrait>::FieldVisitor,
            ) -> <#mdt_struct_name as MakeDynTrait>::FieldVisitor {
                Self::MIRROR
                    .field_traverser
                    .accept::<Self>(&mut visitor, self);

                visitor
            }
        }
    };

    let make_forward_trait_items = |for_derivation| {
        input
            .items
            .iter()
            .map(|item| match item.clone() {
                syn::TraitItem::Const(syn::TraitItemConst {
                    attrs,
                    const_token,
                    ident,
                    generics,
                    colon_token,
                    ty,
                    ..
                }) => syn::ImplItemConst {
                    attrs,
                    vis: syn::Visibility::Inherited,
                    defaultness: Default::default(),
                    const_token,
                    ident: ident.clone(),
                    generics,
                    colon_token,
                    ty,
                    eq_token: Default::default(),
                    expr: syn::parse2(if for_derivation {
                        quote! {
                            derivation::#ident
                        }
                    } else {
                        quote! {
                            <Self as #impl_module_name::#derive_trait_name>::#ident
                        }
                    })
                    .unwrap(),
                    semi_token: Default::default(),
                }
                .to_token_stream(),
                syn::TraitItem::Fn(syn::TraitItemFn { attrs, sig, .. }) => {
                    let ident = sig.ident.clone();
                    let args = sig
                        .inputs
                        .iter()
                        .map(|arg| match arg {
                            syn::FnArg::Receiver(_) => quote! { self },
                            syn::FnArg::Typed(pat_type) => {
                                // TODO: convert pattern to an expression properly
                                // * strip mut
                                // * handle ref
                                pat_type.pat.to_token_stream()
                            }
                        })
                        .collect::<Vec<_>>();

                    syn::ImplItemFn {
                        attrs,
                        vis: syn::Visibility::Inherited,
                        defaultness: Default::default(),
                        sig,
                        block: syn::parse2(if for_derivation {
                            quote! {{
                                derivation::#ident(#(#args),*)
                            }}
                        } else {
                            quote! {{
                                <Self as #impl_module_name::#derive_trait_name>::#ident(#(#args),*)
                            }}
                        })
                        .inspect_err(|e| {
                            eprintln!("hello????? here {}", e);
                        })
                        .unwrap(),
                    }
                    .to_token_stream()
                }
                syn::TraitItem::Type(syn::TraitItemType {
                    attrs,
                    type_token,
                    ident,
                    generics,
                    semi_token,
                    ..
                }) => syn::ImplItemType {
                    attrs,
                    vis: syn::Visibility::Inherited,
                    defaultness: Default::default(),
                    type_token,
                    ident: ident.clone(),
                    generics,
                    eq_token: Default::default(),
                    ty: syn::parse2(if for_derivation {
                        quote! {{
                            derivation::#ident
                        }}
                    } else {
                        quote! {
                            <Self as #impl_module_name::#derive_trait_name>::#ident
                        }
                    })
                    .inspect_err(|e| {
                        eprintln!("hello????? {}", e);
                    })
                    .unwrap(),
                    semi_token,
                }
                .to_token_stream(),
                _ => unimplemented!(),
            })
            .collect::<Vec<_>>()
    };

    let forward_trait_items = make_forward_trait_items(false);
    let forward_trait_items_for_derivation_syntax = make_forward_trait_items(true);

    let impl_trait_macro = quote! {
        #[macro_export]
        macro_rules! #impl_trait_macro_name {
            ($name:ident) => {
                const _: Validation = <$name as #impl_module_name::#derive_trait_name>::VALIDATION;

                impl #trait_name for $name {
                    #(#forward_trait_items)*
                }
            };

            (@MDT) => {
                #impl_module_name::#mdt_struct_name
            };

            (@blanket_impl $derivation:path; $validation:expr) => {
                const _: () = {
                    use $derivation as derivation;
                    impl<T> #derive_trait_name for T
                    where T: ::zambaga::WithMirror<#trait_name!(@MDT)> {
                        const VALIDATION: ::zambaga::Validation = $validation;
                        #(#forward_trait_items_for_derivation_syntax)*
                    }
                };
            };

            { @uses; $($i:item)* } => {
                const _: () = {
                    use #impl_module_name::*;
                    use ::zambaga::*;
                    $($i)*
                };
            };
        }
        pub use #impl_trait_macro_name as #impl_trait_macro_name;
    };

    let res = quote! {
        #input

        pub mod #impl_module_name {
            use super::*;
            use ::zambaga::*;
            use ::std::any::Any;
            #mdt
            #is_trait_struct
            #visit_field_trait
            #derive_trait
            #impl_trait_macro
       }
       pub(crate) use #impl_module_name::#impl_make_dyn_trait_macro_name as #impl_make_dyn_trait_macro_name;
       pub(crate) use #impl_module_name::#mdt_struct_name as #mdt_struct_name;
       pub(crate) use #impl_module_name::#visit_field_trait_name as #visit_field_trait_name;
       pub(crate) use #impl_module_name::#derive_trait_name as #derive_trait_name;
       pub use #impl_module_name::#impl_trait_macro_name as #trait_name;
    };

    // println!("{}", res);

    res
}

#[proc_macro_attribute]
pub fn derivation(trait_name: PMTokenStream, item: PMTokenStream) -> PMTokenStream {
    derivation_impl(trait_name.into(), item.into()).into()
}

fn derivation_impl(trait_name: TokenStream, item: TokenStream) -> TokenStream {
    let input: syn::ItemMod = syn::parse2(item.clone()).unwrap();
    let derivation_name = input.ident.clone();
    let trait_name_snake_case = heck::AsSnakeCase(trait_name.to_string());

    let has_visit_any_method = false; // TODO
    let validation = if has_visit_any_method {
        quote! { ::zambaga::Validation::ok() }
    } else {
        quote! { ::zambaga::Validation::all_fields_impl(&Self::MIRROR) }
    };

    let derivation = quote! {
        #trait_name!(@blanket_impl #derivation_name; #validation);
    };

    quote! {
        #item
        #derivation
    }
}
