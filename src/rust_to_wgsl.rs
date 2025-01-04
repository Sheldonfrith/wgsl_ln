use proc_macro2::TokenStream;
use proc_macro_error::set_dummy;
use quote::quote;

use crate::{conversions::convert_all_rust_types_in_stream_to_wgsl_types, wgsl2::wgsl2};

pub fn rust_to_wgsl(stream: TokenStream) -> TokenStream {
    let stream_converted = convert_all_rust_types_in_stream_to_wgsl_types(stream);
    wgsl2(stream_converted)
}
