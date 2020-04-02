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

pub mod dispatch;
pub mod export;
pub mod handles;
pub mod import;
pub mod types;

pub use self::dispatch::generate_dispatch;
pub use self::export::generate_export;
pub use self::handles::generate_handles;
pub use self::import::generate_import;

pub fn path_of_single_ident(ident: syn::Ident) -> syn::Path {
    syn::Path {
        leading_colon: None,
        segments: {
            let mut punc = syn::punctuated::Punctuated::new();
            punc.push(syn::PathSegment {
                ident,
                arguments: syn::PathArguments::None,
            });
            punc
        },
    }
}
