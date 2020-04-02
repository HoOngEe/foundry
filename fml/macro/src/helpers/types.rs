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

#[derive(PartialEq)]
pub enum TypeKind {
    Free,
    Exported,
    Imported,
    Mixed,
}

pub fn traverse_type(the_type: &syn::Type) -> Result<TypeKind, TokenStream2> {
    match the_type {
        syn::Type::Path(p) => {
            // TODO
            Ok(TypeKind::Free)
        }
        syn::Type::Tuple(t) => {
            // TODO
            Ok(TypeKind::Free)
        }
        _ => Err(TokenStream2::from(
            syn::Error::new_spanned(the_type, format!("This type is not allowed")).to_compile_error(),
        )),
    }
}
