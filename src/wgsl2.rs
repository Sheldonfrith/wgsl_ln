use naga::valid::{Capabilities, ValidationFlags, Validator};
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{format_ident, quote};

use crate::{
    conversions::convert_all_rust_types_in_stream_to_wgsl_types, sanitize::sanitize_wgsl,
    to_wgsl_string::to_wgsl_string,
};

// fn insert_imported_wgsl(name: proc_macro2::Ident) -> TokenStream {
//     let full_paste_name = format_ident!("__wgsl_paste_{}", name);
//     // recursive
//     return quote! {{use crate::*; #full_past_name!(wgsl!(#stream))}};
// }
pub fn wgsl2(stream: TokenStream) -> TokenStream {
    let (stream, pastes) = sanitize_wgsl(stream);
    if let Some(paste) = pastes {
        let paste = format_ident!("__wgsl_paste_{}", paste);
        // recursive
        return quote! {{use crate::*; #paste!(wgsl!(#stream))}};
    }

    let mut spans = Vec::new();
    let mut source = String::new();
    #[allow(unused_variables)]
    let uses_naga_oil = to_wgsl_string(stream, &mut spans, &mut source);
    #[cfg(feature = "naga_oil")]
    if uses_naga_oil {
        return quote! {#source};
    }
    match naga::front::wgsl::parse_str(&source) {
        Ok(module) => {
            match Validator::new(ValidationFlags::all(), Capabilities::all()).validate(&module) {
                Ok(_) => quote! {#source},
                Err(e) => {
                    if let Some((span, _)) = e.spans().next() {
                        let location = span.location(&source);
                        let pos = match spans
                            .binary_search_by_key(&(location.offset as usize), |x| x.0)
                        {
                            Ok(x) => x,
                            Err(x) => x.saturating_sub(1),
                        };
                        abort!(spans[pos].1, "Wgsl Error: {}", e);
                    }
                    let e_str = e.to_string();
                    quote! {compile_error!(#e_str)}
                }
            }
        }
        Err(e) => {
            if let Some((span, _)) = e.labels().next() {
                let location = span.location(&source);
                let pos = match spans.binary_search_by_key(&(location.offset as usize), |x| x.0) {
                    Ok(x) => x,
                    Err(x) => x.saturating_sub(1),
                };
                abort!(spans[pos].1, "Wgsl Error: {}", e);
            }
            let e_str = e.to_string();
            quote! {compile_error!(#e_str)}
        }
    }
}
pub fn rust_to_wgsl2(rust_stream: TokenStream) -> TokenStream {
    let wgsl_stream = convert_all_rust_types_in_stream_to_wgsl_types(rust_stream.clone());
    let (wgsl_stream, wgsl_pastes) = sanitize_wgsl(wgsl_stream);
    if let Some(paste) = wgsl_pastes {
        let paste = format_ident!("__rust_to_wgsl_paste_{}", paste);
        return quote! {{use crate::*; #paste!(_wgsl!(#wgsl_stream))}};
    }
    let mut spans = Vec::new();
    let mut source = String::new();
    #[allow(unused_variables)]
    let uses_naga_oil = to_wgsl_string(wgsl_stream, &mut spans, &mut source);
    #[cfg(feature = "naga_oil")]
    if uses_naga_oil {
        return quote! {#source};
    }
    match naga::front::wgsl::parse_str(&source) {
        Ok(module) => {
            match Validator::new(ValidationFlags::all(), Capabilities::all()).validate(&module) {
                Ok(_) => quote! {#source},
                Err(e) => {
                    if let Some((span, _)) = e.spans().next() {
                        let location = span.location(&source);
                        let pos = match spans
                            .binary_search_by_key(&(location.offset as usize), |x| x.0)
                        {
                            Ok(x) => x,
                            Err(x) => x.saturating_sub(1),
                        };
                        abort!(spans[pos].1, "Wgsl Error: {}", e);
                    }
                    let e_str = e.to_string();
                    quote! {compile_error!(#e_str)}
                }
            }
        }
        Err(e) => {
            if let Some((span, _)) = e.labels().next() {
                let location = span.location(&source);
                let pos = match spans.binary_search_by_key(&(location.offset as usize), |x| x.0) {
                    Ok(x) => x,
                    Err(x) => x.saturating_sub(1),
                };
                abort!(spans[pos].1, "Wgsl Error: {}", e);
            }
            let e_str = e.to_string();
            quote! {compile_error!(#e_str)}
        }
    }
}
