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

mod helpers;

use proc_macro::TokenStream;
use syn::parse_macro_input;
const MODULE_NAME: &str = "handles";

use helpers::*;

fn fml_macro_core(args: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::Item);
    if !args.is_empty() {
        return TokenStream::from(
            syn::Error::new_spanned(ast, "#[fml_macro] doesn't take any arguments").to_compile_error(),
        )
    }

    let the_module = match ast {
        syn::Item::Mod(x) => x,
        item => {
            return TokenStream::from(
                syn::Error::new_spanned(item, format!("Use #[fml_macro] only once, on the `{}` module", MODULE_NAME))
                    .to_compile_error(),
            )
        }
    };

    if the_module.ident != MODULE_NAME {
        return TokenStream::from(
            syn::Error::new_spanned(the_module, format!("Use #[fml_macro] only once, on the `{}` module", MODULE_NAME))
                .to_compile_error(),
        )
    }

    if let Some(item) = the_module.content.as_ref().expect("Your module is empty!").1.iter().find(|x| {
        if let syn::Item::Trait(_) = x {
            return false
        }
        true
    }) {
        return TokenStream::from(
            syn::Error::new_spanned(item, "Your module contains a non-trait item.").to_compile_error(),
        )
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

    let attribute_error = "Handle trait must have only one of either #[exported] or #[imported] as an attribute.";
    let mut exported = Vec::new();
    let mut imported = Vec::new();
    for t in trait_items {
        if t.attrs.len() != 1 {
            return TokenStream::from(syn::Error::new_spanned(t, attribute_error).to_compile_error())
        }
        if *t.attrs[0].path.get_ident().expect(attribute_error) == syn::parse_str::<syn::Ident>("exported").unwrap() {
            exported.push(t);
        } else if *t.attrs[0].path.get_ident().expect(attribute_error)
            == syn::parse_str::<syn::Ident>("imported").unwrap()
        {
            imported.push(t);
        } else {
            return TokenStream::from(syn::Error::new_spanned(t, attribute_error).to_compile_error())
        }
    }

    let handles = {
        let result = generate_handles(&exported, &imported);
        match result {
            Ok(x) => x,
            Err(x) => return TokenStream::from(x),
        }
    };
    let dispatch = {
        let result = generate_dispatch(&exported);
        match result {
            Ok(x) => x,
            Err(x) => return TokenStream::from(x),
        }
    };
    let export = {
        let result = generate_export(&exported);
        match result {
            Ok(x) => x,
            Err(x) => return TokenStream::from(x),
        }
    };
    let import = {
        let result = generate_import(&imported);
        match result {
            Ok(x) => x,
            Err(x) => return TokenStream::from(x),
        }
    };

    let result = quote! {
        #handles
        pub mod generated {
            #dispatch
            #export
            #import
        }
    };
    TokenStream::from(result)
}

#[proc_macro_attribute]
pub fn fml_macro(args: TokenStream, input: TokenStream) -> TokenStream {
    fml_macro_core(args, input)
}

#[proc_macro_attribute]
pub fn fml_macro_debug(args: TokenStream, input: TokenStream) -> TokenStream {
    println!("{}", fml_macro_core(args, input));
    TokenStream::new()
}
