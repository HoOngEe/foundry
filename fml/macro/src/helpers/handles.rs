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

use super::types::{traverse_type, TypeKind};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::ToTokens;

fn test_type(test: TypeKind, handle: &syn::ItemTrait, msg1: &str, msg2: &str) -> Result<(), TokenStream2> {
    for method in &handle.items {
        let method = match method {
            syn::TraitItem::Method(x) => x,
            _ => continue,
        };
        for arg in &method.sig.inputs {
            if let syn::FnArg::Typed(p) = arg {
                match traverse_type(&*p.ty)? {
                    TypeKind::Free => (),
                    _ => return Err(TokenStream2::from(syn::Error::new_spanned(arg, msg1).to_compile_error())),
                }
            }
        }

        if let syn::ReturnType::Type(_, t) = &method.sig.output {
            let traverse_result = traverse_type(t)?;
            if traverse_result != test && traverse_result != TypeKind::Free {
                return Err(TokenStream2::from(syn::Error::new_spanned(&method.sig.output, msg2).to_compile_error()))
            }
        }
    }
    Ok(())
}

pub fn generate_handles(
    exported_handles: &Vec<&syn::ItemTrait>,
    imported_handles: &Vec<&syn::ItemTrait>,
) -> Result<TokenStream2, TokenStream2> {
    let mut the_exporteds = TokenStream2::new();
    let mut the_importeds = TokenStream2::new();

    for handle in exported_handles {
        let mut the_trait = (*handle).clone();
        the_trait.attrs.clear();
        test_type(
            TypeKind::Exported,
            &handle,
            "You must not use Handle type as an argument",
            "You must not use imported Handle as a return type of method in exported Handle",
        )?;
        the_exporteds.extend(the_trait.to_token_stream());
    }

    for handle in imported_handles {
        let mut the_trait = (*handle).clone();
        the_trait.attrs.clear();
        test_type(
            TypeKind::Imported,
            &handle,
            "You must not use Handle type as an argument",
            "You must not use exported Handle as a return type of method in imported Handle",
        )?;
        the_importeds.extend(the_trait.to_token_stream());
    }

    let module = TokenStream2::from(quote! {
        pub mod handles {
            pub use super::generated::{export, import};
            #the_exporteds
            #the_importeds
        }
    });
    Ok(module)
}
