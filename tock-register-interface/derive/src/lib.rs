// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

mod definition;
mod deref_impl;
mod parsing;
mod static_block;

#[cfg(test)]
mod test_util;

use definition::definition;
use deref_impl::deref_impl;
use proc_macro2::Span;
use quote::quote;
use static_block::static_block;
use syn::{parse_macro_input, Attribute, Ident, LitInt, Path, Token, Type, Visibility};

#[proc_macro]
pub fn peripheral(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Input);
    let definition = definition(&input);
    let deref_impl = deref_impl(&input);
    let static_block = static_block(&input);
    quote! {
        #definition
        #deref_impl
        #static_block
    }
    .into()
}

#[cfg_attr(test, derive(Debug, PartialEq))]
struct Input {
    #[allow(dead_code)] // TODO: Remove
    pub allow_bus_adapter: bool,
    pub cfgs: Vec<Attribute>,
    pub comments: Vec<Attribute>,
    pub fields: Vec<Field>,
    pub name: Ident,
    pub real_name: Ident,
    pub tock_registers: Path,
    pub visibility: Visibility,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
struct Field {
    pub cfgs: Vec<Attribute>,
    pub comments: Vec<Attribute>,
    pub contents: FieldContents,
    pub offset: Option<LitInt>,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
enum FieldContents {
    Padding(Token![_]),
    Register(Register),
}

impl FieldContents {
    pub fn padding(&self) -> Option<&Token![_]> {
        match self {
            FieldContents::Padding(underscore) => Some(underscore),
            FieldContents::Register(_) => None,
        }
    }

    pub fn register(&self) -> Option<&Register> {
        match self {
            FieldContents::Padding(_) => None,
            FieldContents::Register(register) => Some(register),
        }
    }
}

#[cfg_attr(test, derive(Debug, PartialEq))]
struct Register {
    pub data_type: Type,
    pub name: Ident,
    pub read: Option<Safety>,
    pub write: Option<Safety>,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
enum Safety {
    Safe(Ident),
    Unsafe(Ident),
}

impl Safety {
    pub fn span(&self) -> Span {
        match self {
            Safety::Safe(ident) => ident.span(),
            Safety::Unsafe(ident) => ident.span(),
        }
    }
}
