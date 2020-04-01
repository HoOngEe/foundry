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

extern crate proc_macro;
extern crate proc_macro_crate;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro2;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::ToTokens;
use syn::Ident;

use syn::parse_macro_input;
use syn::{DeriveInput, ItemImpl};

const MODULE_NAME: &'static str = "handles";

fn debugmark() {
    println!("-----------------");
    println!("STEPTSTPETPETPETP");
    println!("-----------------");
}

fn generate_dispatch_for_a_single_trait(
    methods: Vec<&syn::TraitItemMethod>,
    num: u32,
) -> Result<TokenStream2, TokenStream2> {
    let fnname = syn::Ident::new(&format!("dispatch_{}", num), Span::call_site());
    let code = TokenStream2::from(quote! {
        #[allow(clippy::let_unit_value)]
        fn #fnname(
            mut buffer: Cursor<&mut Vec<u8>>,
            object: Arc<dyn handles::Customer + Send + Sync>,
            method: MethodId,
            data: &[u8],
        ) {
        }
    });
    let mut function = syn::parse2::<syn::ItemFn>(code).unwrap();
    let mut the_match = syn::parse_str::<syn::ExprMatch>("match method {}").unwrap();

    // Make a match arm for each handle's method
    for (i, m) in methods.iter().enumerate() {
        let index = syn::Lit::Int(syn::LitInt::new(&format!("{}", i + 1), Span::call_site()));
        let the_arm = TokenStream::from(quote! {
            #index => {
            }
        });
        let mut the_arm = syn::parse::<syn::Arm>(the_arm).unwrap();

        let argnum = m.sig.inputs.len() - 1;
        if let syn::FnArg::Typed(_) = m.sig.inputs[0] {
            return Err(TokenStream2::from(
                syn::Error::new_spanned(m, format!("All methods must have &self"))
                    .to_compile_error(),
            ));
        }

        // statement #1
        let mut the_let_pattern = syn::PatTuple {
            attrs: Vec::new(),
            paren_token: syn::token::Paren(Span::call_site()),
            elems: syn::punctuated::Punctuated::new(),
        };
        for j in 1..(argnum + 1) {
            let the_iden = syn::Ident::new(&format!("a{}", j), Span::call_site());
            the_let_pattern.elems.push(syn::Pat::Ident(syn::PatIdent {
                attrs: Vec::new(),
                by_ref: None,
                mutability: None,
                ident: the_iden,
                subpat: None,
            }));
        }
        let stmt_deserialize = TokenStream::from(quote! {
            let #the_let_pattern = serde_cbor::from_reader(&data[std::mem::size_of::<PacketHeader>()..]).unwrap();
        });

        // statement #2
        let mut the_args: syn::punctuated::Punctuated<syn::Expr, syn::token::Comma> =
            syn::punctuated::Punctuated::new();
        for j in 1..(argnum + 1) {
            let the_iden = proc_macro2::TokenStream::from(quote! {a#j});
            the_args.push(syn::Expr::Verbatim(the_iden));
        }
        let method_name = m.sig.ident.clone();
        let stmt_call = TokenStream::from(quote! {
            let result = object.#method_name(#the_args);
        });

        // statement #3
        let stmt_serialize = TokenStream::from(quote! {
            serde_cbor::to_writer(&mut buffer, &result).unwrap();
        });

        let mut stmts = proc_macro2::TokenStream::new();
        stmts.extend(proc_macro2::TokenStream::from(stmt_deserialize));
        stmts.extend(proc_macro2::TokenStream::from(stmt_call));
        stmts.extend(proc_macro2::TokenStream::from(stmt_serialize));

        the_arm.body = Box::new(syn::Expr::Verbatim(stmts));
        the_match.arms.push(the_arm);
    }

    function
        .block
        .stmts
        .push(syn::Stmt::Expr(syn::Expr::Match(the_match)));
    Ok(function.to_token_stream())
}

fn generate_dispatch(
    exported_handles: &Vec<&syn::ItemTrait>,
) -> Result<TokenStream2, TokenStream2> {
    let mut the_match = syn::parse_str::<syn::ExprMatch>("match handle.trait_id {}").unwrap();
    for (i, h) in exported_handles.iter().enumerate() {
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
    }

    let mut the_struct = syn::parse_str::<syn::ExprStruct>("ExportedHandles {port_id,}").unwrap();
    for i in 0..exported_handles.len() {
        the_struct.fields.push(
            syn::parse_str::<syn::FieldValue>(&format!(
                "handles_trait{}: HandlePool::new(size)",
                i + 1
            ))
            .unwrap(),
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
        dispatch.extend(generate_dispatch_for_a_single_trait(methods, i as u32)?);
    }
    let result = syn::Item::Verbatim(dispatch);

    let module = TokenStream2::from(quote! {
        pub mod dispatch {
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
    });
    Ok(module)
}

fn gnerate_export(exported_handles: &Vec<&syn::ItemTrait>) -> Result<TokenStream2, TokenStream2> {
    let mut the_handles = TokenStream2::new();
    let mut the_exported_handles = syn::parse_str::<syn::ItemStruct>(
        "
    pub struct ExportedHandles {
        pub port_id: PortId,
    }",
    )
    .unwrap();
    let mut the_methods = TokenStream2::new();

    for (i, trait_item) in exported_handles.iter().enumerate() {
        // Add concrete sturct for each handle
        let the_struct = syn::parse_str::<syn::ItemStruct>(&format!(
            "
        #[derive(Serialize, Deserialize, Debug)]
        pub struct {} {{
            pub handle: ExportedHandle,
        }}",
            trait_item.ident
        ))
        .unwrap();
        the_handles.extend(the_struct.to_token_stream());

        // Add fields in struct ExportedHandles
        if let syn::Fields::Named(fields) = &mut the_exported_handles.fields {
            fields.named.push({
                syn::Field {
                    attrs: Vec::new(),
                    vis: syn::Visibility::Public(syn::VisPublic {
                        pub_token: syn::token::Pub(Span::call_site()),
                    }),
                    ident: Some(
                        syn::parse_str::<syn::Ident>(&format!("handles_trait{}", i + 1)).unwrap(),
                    ),
                    colon_token: Some(syn::token::Colon(Span::call_site())),
                    ty: syn::parse_str::<syn::Type>(&format!(
                        "HandlePool<dyn handles::{} + Send + Sync>",
                        trait_item.ident
                    ))
                    .unwrap(),
                }
            })
        } else {
            panic!();
        }

        // Add handle creation methods in impl of struct ExportedHandles
        let big_case = format!("{}", trait_item.ident);
        let small_case = big_case.to_lowercase();
        let the_method = syn::parse_str::<syn::TraitItemMethod>(&format!(
            "
        pub fn create_handle_{}<T: handles::{} + Send + Sync + 'static>(&self, x: T) -> {} {{
            let trait_id = {} as u16;
            let index = self.handles_trait1.create(Arc::new(x)) as u16;
            {} {{
                handle: ExportedHandle {{
                    port_id: self.port_id,
                    id: HandleInstanceId {{
                        trait_id,
                        index,
                    }},
                }},
            }}
        }}
        ",
            small_case,
            big_case,
            big_case,
            i + 1,
            big_case
        ))
        .unwrap();
        the_methods.extend(the_method.to_token_stream());
    }
    let the_handles = syn::Item::Verbatim(the_handles);
    let the_methods = syn::Item::Verbatim(the_methods);
    let module = TokenStream2::from(quote! {
        pub mod dispatch {
            use super::super::super::get_context;
            use super::super::handles;
            use fml::handle::pool::HandlePool;
            use fml::handle::{ExportedHandle, HandleInstanceId};
            use fml::port::PortId;
            use serde::{Deserialize, Serialize};
            use std::sync::Arc;

            pub fn get_handle_pool(port_id: PortId) -> Arc<ExportedHandles> {
                get_context().ports.lock().unwrap().get(&port_id).unwrap().1.dispatcher_get()
            }
            #the_handles
            #the_exported_handles

            impl ExportedHandles {
                #the_methods
            }

        }
    });
    Ok(module)
}

#[proc_macro_attribute]
pub fn fml_macro(args: TokenStream, input: TokenStream) -> TokenStream {
    assert!(args.is_empty(), "#[fml_macro] doesn't take any arguments");

    let input_copy = input.clone(); // parse_macro_input! take only single identifier
    let ast = parse_macro_input!(input_copy as syn::Item);
    let the_module = match ast {
        syn::Item::Mod(x) => x,
        item => {
            return TokenStream::from(
                syn::Error::new_spanned(
                    item,
                    format!(
                        "Use #[fml_macro] only once, on the `{}` module",
                        MODULE_NAME
                    ),
                )
                .to_compile_error(),
            );
        }
    };

    if the_module.ident != MODULE_NAME {
        return TokenStream::from(
            syn::Error::new_spanned(
                the_module,
                format!(
                    "Use #[fml_macro] only once, on the `{}` module",
                    MODULE_NAME
                ),
            )
            .to_compile_error(),
        );
    }

    if let Some(item) = the_module
        .content
        .as_ref()
        .expect("Your module is empty!")
        .1
        .iter()
        .find(|x| {
            if let syn::Item::Trait(_) = x {
                return false;
            }
            true
        })
    {
        return TokenStream::from(
            syn::Error::new_spanned(item, "Your module contains a non-trait item.")
                .to_compile_error(),
        );
    }

    let trait_items: Vec<&syn::ItemTrait> = the_module
        .content
        .as_ref()
        .expect("Your module is empty!")
        .1
        .iter()
        .map(|x| {
            if let syn::Item::Trait(item_trait) = x {
                item_trait
            } else {
                panic!();
            }
        })
        .collect();

    let attribute_error =
        "Handle trait must have only one of either #[exported] or #[imported] as an attribute.";
    let mut exported = Vec::new();
    let mut imported = Vec::new();
    for t in trait_items {
        if t.attrs.len() != 1 {
            return TokenStream::from(
                syn::Error::new_spanned(t, attribute_error).to_compile_error(),
            );
        }
        if *t.attrs[0].path.get_ident().expect(attribute_error)
            == syn::parse_str::<syn::Ident>("exported").unwrap()
        {
            exported.push(t);
        } else if *t.attrs[0].path.get_ident().expect(attribute_error)
            == syn::parse_str::<syn::Ident>("imported").unwrap()
        {
            imported.push(t);
        } else {
            return TokenStream::from(
                syn::Error::new_spanned(t, attribute_error).to_compile_error(),
            );
        }
    }

    let dispatch = {
        let result = generate_dispatch(&exported);
        match result {
            Ok(x) => x,
            Err(x) => return TokenStream::from(x),
        }
    };
    println!("{}", dispatch);
    println!("{}", gnerate_export(&exported).unwrap());

    let mut result = input.clone();
    println!("{}", result);

    TokenStream::new()
}
