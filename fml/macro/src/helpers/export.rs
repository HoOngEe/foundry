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

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::ToTokens;

pub fn generate_export(exported_handles: &[&syn::ItemTrait]) -> Result<TokenStream2, TokenStream2> {
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
        // Add a concrete sturct for each handle
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
            // Sadly, syn can't parse a field.
            fields.named.push({
                syn::Field {
                    attrs: Vec::new(),
                    vis: syn::Visibility::Public(syn::VisPublic {
                        pub_token: syn::token::Pub(Span::call_site()),
                    }),
                    ident: Some(syn::parse_str::<syn::Ident>(&format!("handles_trait{}", i + 1)).unwrap()),
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

        // Add a handle creation method in impl of struct ExportedHandles
        let big_case = format!("{}", trait_item.ident);
        let small_case = big_case.to_lowercase();
        let the_method = syn::parse_str::<syn::ImplItemMethod>(&format!(
            "
        pub fn create_handle_{}<T: handles::{} + Send + Sync + 'static>(&self, x: T) -> {} {{
            let trait_id = {} as u16;
            let index = self.handles_trait{}.create(Arc::new(x)) as u16;
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
            i + 1,
            big_case
        ))
        .unwrap();
        the_methods.extend(the_method.to_token_stream());
    }
    let the_handles = syn::Item::Verbatim(the_handles);
    let the_methods = syn::Item::Verbatim(the_methods);
    let module = quote! {
        pub mod export {
            use super::super::types::*;
            use super::super::super::get_context;
            use super::super::handles;
            use fml::handle::pool::HandlePool;
            use fml::handle::{ExportedHandle, HandleInstanceId};
            use fml::port::PortId;
            use serde::{Deserialize, Serialize};
            use std::sync::Arc;

            pub fn get_handle_pool(port_id: PortId) -> Arc<ExportedHandles> {
                get_context().ports.read().unwrap().get(&port_id).unwrap().1.dispatcher_get()
            }
            #the_handles
            #the_exported_handles

            impl ExportedHandles {
                #the_methods
            }

        }
    };
    Ok(module)
}
