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

use path_of_single_ident;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::ToTokens;

fn generate_dispatch_for_a_single_trait(
    trait_name: &str,
    methods: &[&syn::TraitItemMethod],
    num: u32,
) -> Result<TokenStream2, TokenStream2> {
    let mut function = syn::parse_str::<syn::ItemFn>(&format!(
        "
    #[allow(clippy::let_unit_value)]
    fn dispatch_{}(
        mut buffer: Cursor<&mut Vec<u8>>,
        object: Arc<dyn handles::{} + Send + Sync>,
        method: MethodId,
        data: &[u8],
    ) {{}}
    ",
        num + 1,
        trait_name
    ))
    .unwrap();

    let mut the_match = syn::parse_str::<syn::ExprMatch>("match method {}").unwrap();

    // Make a match arm for each handle's method
    for (i, m) in methods.iter().enumerate() {
        if let syn::FnArg::Typed(_) = m.sig.inputs[0] {
            return Err(syn::Error::new_spanned(m, "All methods must have &self").to_compile_error())
        }

        // statement #1
        let mut the_let_pattern = syn::PatTuple {
            attrs: Vec::new(),
            paren_token: syn::token::Paren(Span::call_site()),
            elems: syn::punctuated::Punctuated::new(),
        };
        for j in 1..m.sig.inputs.len() {
            let the_iden = syn::Ident::new(&format!("a{}", j), Span::call_site());
            the_let_pattern.elems.push(syn::Pat::Ident(syn::PatIdent {
                attrs: Vec::new(),
                by_ref: None,
                mutability: None,
                ident: the_iden,
                subpat: None,
            }));
        }
        let stmt_deserialize = quote! {
            let #the_let_pattern = serde_cbor::from_reader(&data[std::mem::size_of::<PacketHeader>()..]).unwrap();
        };

        // statement #2
        let mut the_args: syn::punctuated::Punctuated<syn::Expr, syn::token::Comma> =
            syn::punctuated::Punctuated::new();
        for j in 1..m.sig.inputs.len() {
            let the_arg = syn::parse_str::<syn::Ident>(&format!("a{}", j)).unwrap();
            the_args.push(syn::Expr::Path(syn::ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: path_of_single_ident(the_arg),
            }));
        }
        let method_name = m.sig.ident.clone();
        let stmt_call = quote! {
            let result = object.#method_name(#the_args);
        };

        let index = syn::Lit::Int(syn::LitInt::new(&format!("{}", i + 1), Span::call_site()));
        let the_arm = quote! {
            #index => {
                #stmt_deserialize
                #stmt_call
                serde_cbor::to_writer(&mut buffer, &result).unwrap();
            }
        };
        the_match.arms.push(syn::parse2::<syn::Arm>(the_arm).unwrap());
    }

    the_match.arms.push(syn::parse_str("_ => panic!()").unwrap());
    function.block.stmts.push(syn::Stmt::Expr(syn::Expr::Match(the_match)));
    Ok(function.to_token_stream())
}

pub fn generate_dispatch(exported_handles: &[&syn::ItemTrait]) -> Result<TokenStream2, TokenStream2> {
    let mut the_match = syn::parse_str::<syn::ExprMatch>("match handle.trait_id {}").unwrap();
    for i in 0..exported_handles.len() {
        let code = format!(
            "{} => {{
            dispatch_{}(buffer, self.handles_trait{}.get(handle.index as usize), method, data);
        }}",
            i + 1,
            i + 1,
            i + 1
        );
        let the_arm = syn::parse_str::<syn::Arm>(&code).unwrap();
        the_match.arms.push(the_arm);
        the_match.arms.push(syn::parse_str("_ => panic!()").unwrap());
    }

    let mut the_struct = syn::parse_str::<syn::ExprStruct>("ExportedHandles {port_id,}").unwrap();
    for i in 0..exported_handles.len() {
        the_struct.fields.push(
            syn::parse_str::<syn::FieldValue>(&format!("handles_trait{}: HandlePool::new(size)", i + 1)).unwrap(),
        );
    }

    let mut dispatch = TokenStream2::new();
    for (i, source_trait) in exported_handles.iter().enumerate() {
        let methods: Vec<&syn::TraitItemMethod> = source_trait
            .items
            .iter()
            .filter_map(|trait_item| {
                if let syn::TraitItem::Method(method) = trait_item {
                    Some(method)
                } else {
                    None // type, const, ... will be ignored
                }
            })
            .collect();
        dispatch.extend(generate_dispatch_for_a_single_trait(&format!("{}", source_trait.ident), &methods, i as u32)?);
    }
    let result = syn::Item::Verbatim(dispatch);

    let module = quote! {
        pub mod dispatch {
            use super::super::types::*;
            use super::super::handles;
            use super::export::ExportedHandles;
            use crate::fml::handle::Dispatcher;
            use crate::fml::handle::{HandleInstanceId, MethodId};
            use crate::fml::port::PortId;
            use fml::handle::pool::HandlePool;
            use fml::PacketHeader;
            use std::io::Cursor;
            use std::sync::Arc;

            #result

            impl Dispatcher for ExportedHandles {
                fn new(port_id: PortId, size: usize) -> Self {
                    #the_struct
                }

                fn dispatch_and_call(
                    &self,
                    buffer: Cursor<&mut Vec<u8>>,
                    handle: HandleInstanceId,
                    method: MethodId,
                    data: &[u8],
                ) {
                    #the_match
                }
            }
        }
    };
    Ok(module)
}
