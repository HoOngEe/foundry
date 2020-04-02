// Copyright 2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::path_of_single_ident;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::ToTokens;

fn generate_impl_for_a_single_trait(handle: &syn::ItemTrait) -> Result<TokenStream2, TokenStream2> {
    let mut the_impl = syn::parse_str::<syn::ItemImpl>(&format!(
        "
    impl T{} for import::{} {{
    }}
    ",
        handle.ident, handle.ident
    ))
    .unwrap();

    for (i, method) in handle.items.iter().enumerate() {
        let index = syn::Lit::Int(syn::LitInt::new(&format!("{}", i + 1), Span::call_site()));
        let method = match method {
            syn::TraitItem::Method(x) => x,
            _ => continue,
        };
        let mut the_method = syn::parse_str::<syn::ImplItemMethod>("fn dummy() -> () {}").unwrap();
        the_method.sig = method.sig.clone();

        let mut the_tuple = syn::ExprTuple {
            attrs: Vec::new(),
            paren_token: syn::token::Paren(Span::call_site()),
            elems: syn::punctuated::Punctuated::new(),
        };

        for arg in &method.sig.inputs {
            match arg {
                syn::FnArg::Receiver(_) => continue, // &self
                syn::FnArg::Typed(pattern) => {
                    if let syn::Pat::Ident(the_arg) = &*pattern.pat {
                        the_tuple.elems.push(syn::Expr::Path(syn::ExprPath {
                            attrs: Vec::new(),
                            qself: None,
                            path: path_of_single_ident(the_arg.ident.clone()),
                        }));
                    } else {
                        return Err(TokenStream2::from(
                            syn::Error::new_spanned(arg, format!("You must not use a pattern for the argument"))
                                .to_compile_error(),
                        ))
                    }
                }
            }
        }

        let the_call = TokenStream2::from(quote! {
            super::import::call(&self.handle, #index, &#the_tuple)
        });
        let the_call = syn::parse2::<syn::ExprCall>(the_call).unwrap();
        the_method.block.stmts.push(syn::Stmt::Expr(syn::Expr::Call(the_call)));
        the_impl.items.push(syn::ImplItem::Method(the_method));
    }
    Ok(the_impl.to_token_stream())
}

pub fn generate_import(imported_handles: &Vec<&syn::ItemTrait>) -> Result<TokenStream2, TokenStream2> {
    let mut the_uses = TokenStream2::new();
    let mut the_handles = TokenStream2::new();
    let mut the_impls = TokenStream2::new();

    for (i, trait_item) in imported_handles.iter().enumerate() {
        // Add a use statement for each handle trait
        let the_use = syn::parse_str::<syn::ItemUse>(&format!(
            "
        use super::super::handles::{} as T{};
        ",
            trait_item.ident, trait_item.ident
        ))
        .unwrap();
        the_uses.extend(the_use.to_token_stream());

        // Add a concrete sturct for each handle
        let the_struct = syn::parse_str::<syn::ItemStruct>(&format!(
            "
        #[derive(Serialize, Deserialize, Debug)]
        pub struct {} {{
            pub handle: ImportedHandle,
        }}",
            trait_item.ident
        ))
        .unwrap();
        the_handles.extend(the_struct.to_token_stream());
        the_impls.extend(generate_impl_for_a_single_trait(&trait_item));
    }

    let module = TokenStream2::from(quote! {
        pub mod import {
            use super::super::super::get_context;
            #the_uses
            use fml::handle::{ImportedHandle, MethodId};
            use fml::PacketHeader;
            use serde::{Deserialize, Serialize};

            pub fn call<T: serde::Serialize, R: serde::de::DeserializeOwned>(
                handle: &ImportedHandle,
                method: MethodId,
                args: &T,
            ) -> R {
                let mut buffer: Vec<u8> = Vec::new();
                buffer.resize(std::mem::size_of::<PacketHeader>(), 0 as u8);
                serde_cbor::to_writer(&mut buffer[std::mem::size_of::<PacketHeader>()..], &args).unwrap();
                let result =
                    get_context().ports.lock().unwrap().get(&handle.port_id).unwrap().1.call(handle.id, method, buffer);
                serde_cbor::from_reader(&result[std::mem::size_of::<PacketHeader>()..]).unwrap()
            }

            #the_handles
        }
        // we separte this because of `import::` appeared in return type
        pub mod import_impls {
            use super::import as import;
            #the_uses
            #the_impls
        }
    });
    Ok(module)
}
