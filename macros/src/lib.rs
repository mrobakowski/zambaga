mod mode;

use enumflags2::BitFlags;
use mode::ReflectionMode;
use proc_macro::TokenStream as PMTokenStream;
use proc_macro2::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::quote;

#[proc_macro_attribute]
pub fn derive(_attr: PMTokenStream, item: PMTokenStream) -> PMTokenStream {
    derive_impl(_attr.into(), item.into()).into()
}

fn derive_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let only = attr.into_iter().next().unwrap();
    let semi = TokenTree::Punct(Punct::new(';', Spacing::Alone));
    let mut item = item.into_iter().collect::<Vec<_>>();
    let name = item[1].clone();
    let adt_name = TokenTree::Ident(Ident::new(&format!("{}Adt", name), Span::call_site()));
    let mut name_and_item = vec![name, semi.clone(), adt_name, semi.clone()];
    name_and_item.extend(item.iter().cloned());

    item.extend([
        only,
        TokenTree::Punct(Punct::new('!', Spacing::Joint)),
        TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            name_and_item.into_iter().collect(),
        )),
        semi.clone(),
    ]);

    item.into_iter().collect()
}

#[proc_macro_attribute]
pub fn reflect(_attr: PMTokenStream, item: PMTokenStream) -> PMTokenStream {
    reflect_impl(_attr.into(), item.into()).into()
}

fn reflect_impl(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let input: syn::ItemTrait = syn::parse2(item).unwrap();
    let attrs: BitFlags<_> = attrs
        .into_iter()
        .filter_map(|x| match x {
            TokenTree::Ident(ident) => ReflectionMode::from_ident(ident),
            _ => None,
        })
        .collect();

    // TODO: report errors if there are invalid attrs

    let maybe_traitable = if attrs.contains(ReflectionMode::Recursive | ReflectionMode::Dyn) {
        let traitable_name = Ident::new(&format!("{}ableValue", input.ident), Span::call_site());
        let trait_name = input.ident.clone();

        let method_proxies = input
            .items
            .iter()
            .filter_map(|item| match item {
                syn::TraitItem::Fn(method) => {
                    let sig = method.sig.clone();
                    let name = sig.ident.clone();
                    let args = sig
                        .inputs
                        .iter()
                        .filter_map(|arg| match arg {
                            syn::FnArg::Receiver(_) => None,                 // skip self
                            syn::FnArg::Typed(arg) => Some(arg.pat.clone()), // TODO: not all patterns are valid here
                        })
                        .collect::<Vec<_>>();
                    Some(quote! {
                        pub #sig {
                            self.0.#name(#(#args),*)
                        }
                    })
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        Some(quote! {
            #[repr(transparent)]
            pub struct #traitable_name(dyn #trait_name);

            impl #traitable_name {
                #(#method_proxies)*
                pub fn __zambaga_from<T: #trait_name>(value: &T) -> &Self {
                    unsafe { std::mem::transmute(value as &dyn #trait_name) }
                }
            }
        })
    } else {
        None
    };

    let is_adt: Option<TokenStream> = None;

    let derive_trait: Option<TokenStream> = None;

    let macro_rules_loophole: Option<TokenStream> = None;

    quote! {
       #input

       #maybe_traitable

       #is_adt

       #derive_trait

       #macro_rules_loophole
    }
}

#[proc_macro]
pub fn impl_adt(input: PMTokenStream) -> PMTokenStream {
    impl_adt_impl(input.into()).into()
}

fn impl_adt_impl(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter();
    let mut attributes = vec![];
    let mut next = input.next().unwrap();
    while !matches!(next, TokenTree::Punct(ref p) if p.as_char() == ';') {
        attributes.push(next);
        next = input.next().unwrap();
    }

    TokenStream::new()
}
