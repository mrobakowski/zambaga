mod mode;

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

    quote! {
       #input
    }
}
