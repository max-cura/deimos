// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use crate::{Input, Safety};
use proc_macro2::TokenStream;
use quote::quote;

pub fn static_block(input: &Input) -> TokenStream {
    let tock_registers = &input.tock_registers;
    // let comments = &input.comments;
    let cfgs = &input.cfgs;
    let visibility = &input.visibility;
    let real_name = &input.real_name;
    let trait_name = &input.name;
    let fields = input.fields.iter().filter_map(|field| {
        let register = field.contents.register()?;
        let cfgs = &field.cfgs;
        let name = &register.name;
        let data_type = &register.data_type;
        let read = match &register.read {
            None => quote![#tock_registers::NoAccess],
            Some(Safety::Safe(_)) => quote![#tock_registers::Safe],
            Some(Safety::Unsafe(_)) => quote![#tock_registers::Unsafe],
        };
        let write = match &register.write {
            None => quote![#tock_registers::NoAccess],
            Some(Safety::Safe(_)) => quote![#tock_registers::Safe],
            Some(Safety::Unsafe(_)) => quote![#tock_registers::Unsafe],
        };
        let offset_ = field
            .offset
            .as_ref()
            // TODO: not clear to me how this should behave; it seems logical to do this as
            //       `packed`, but there's also no unaligned volatile accesses on `*mut T` right now
            //       so it would seem to require repr(C).
            .expect("offset inferment has not been implemented");
        let dyn_register_type = quote![
            #tock_registers::DynamicRegister<#data_type, #read, #write>
        ];
        let comments = &field.comments;
        Some(quote! {
            #(#cfgs)*
            type #name<'s> = #dyn_register_type
                where Self: 's;
            #(#comments)* #(#cfgs)* fn #name(&self) -> Self::#name<'_> {
                #tock_registers::DynamicRegister::from_ptr(
                    ::core::ptr::with_exposed_provenance_mut(
                        self.addr + #offset_
                    )
                )
            }
        })
    });
    quote! {
        // #(#comments)*
        #(#cfgs)*
        #[allow(non_camel_case_types)]
        #visibility struct #real_name {
            addr: usize,
        }
        impl #real_name {
            pub fn from_addr(addr: usize) -> Self {
                Self { addr }
            }
            pub unsafe fn to_addr(&self) -> usize {
                self.addr
            }
        }
        impl #trait_name for #real_name {
            #(#fields)*
        }
    }
}
