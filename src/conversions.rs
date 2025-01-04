use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::quote;
use syn::{parse2, Type};

pub fn convert_all_rust_types_in_stream_to_wgsl_types(input: TokenStream) -> TokenStream {
    // First, parse the TokenStream into a syn::Type
    match parse2::<Type>(input.clone()) {
        Ok(ty) => {
            // If it's a single type, convert it directly
            convert_rust_type_to_wgsl(&ty)
        }
        Err(_) => {
            // If it's not a single type, it might be a larger syntax item (like a struct or function)
            // Parse it into the appropriate syn type based on your use case
            match parse2::<syn::ItemStruct>(input.clone()) {
                Ok(mut item_struct) => {
                    // Convert types in struct fields
                    for field in &mut item_struct.fields {
                        let new_type = convert_rust_type_to_wgsl(&field.ty);
                        field.ty = parse2(new_type).unwrap_or(field.ty.clone());
                    }
                    quote! { #item_struct }
                }
                Err(_) => {
                    // Handle other cases (functions, impl blocks, etc.)
                    // This is just an example for handling functions
                    match parse2::<syn::ItemFn>(input.clone()) {
                        Ok(mut item_fn) => {
                            // Convert return type if it exists
                            if let syn::ReturnType::Type(arrow, ty) = &item_fn.sig.output {
                                let new_return_type = convert_rust_type_to_wgsl(ty);
                                item_fn.sig.output = syn::ReturnType::Type(
                                    arrow.clone(),
                                    Box::new(parse2(new_return_type).unwrap_or(*ty.clone())),
                                );
                            }

                            // Convert parameter types
                            for input in &mut item_fn.sig.inputs {
                                if let syn::FnArg::Typed(pat_type) = input {
                                    let new_type = convert_rust_type_to_wgsl(&pat_type.ty);
                                    pat_type.ty =
                                        Box::new(parse2(new_type).unwrap_or(*pat_type.ty.clone()));
                                }
                            }
                            quote! { #item_fn }
                        }
                        Err(_) => {
                            // If we can't parse it as any known type, return the original stream
                            input
                        }
                    }
                }
            }
        }
    }
}

/// recursive
fn convert_rust_type_to_wgsl(ty: &Type) -> TokenStream {
    println!("convert_rust_type_to_wgsl");
    match ty {
        #![cfg_attr(test, deny(non_exhaustive_omitted_patterns))]
        Type::Array(_) => rust_array_to_wgsl_array(ty),
        Type::Slice(_) => {
            print!("slice");
            abort!(
                Span::call_site(),
                "WGSL does not support slices or arrays with dynamic length."
            );
        }
        Type::Path(_) => rust_handle_path_type(ty),
        Type::Group(g) => convert_rust_type_to_wgsl(g.elem.as_ref()),
        Type::Tuple(_) => {
            print!("tuple");
            abort!(
                Span::call_site(),
                "WGSL does not support Tuple types, use arrays instead."
            );
        }
        _ => {
            abort!(Span::call_site(), "Unsupported type");
        }
    }
}

fn rust_handle_path_type(ty: &Type) -> TokenStream {
    println!("path");
    if let Type::Path(type_path) = ty {
        let last_segment = type_path
            .path
            .segments
            .last()
            .expect("Type path should have at least one segment");
        return rust_handle_path_segment(last_segment);
    } else {
        abort!(
            Span::call_site(),
            "rust_handle_type_path was given a non-Type::Path type"
        );
    }
}

fn rust_handle_path_segment(segment: &syn::PathSegment) -> TokenStream {
    println!("segment: {:?}", segment.ident);
    match segment.ident.to_string().as_str() {
        "f32" => quote!(f32),
        "f64" | "f8" | "u8" | "u16" | "u64" | "u128" | "i8" | "i16" | "i64" | "i128" | "usize" => {
            abort!(
                Span::call_site(),
                "WGSL only supports numeric types f32, f16, i32, and u32.",
            );
        }
        "i32" => quote!(i32),
        "u32" => quote!(u32),
        "bool" => quote!(bool),
        "Vec2" | "Vec3" | "Vec4" => {
            // Handle generic parameters for vector types
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                if let Some(arg) = args.args.first() {
                    if let syn::GenericArgument::Type(type_arg) = arg {
                        let inner_type = convert_rust_type_to_wgsl(type_arg);
                        let vec_type = match segment.ident.to_string().as_str() {
                            "Vec2" => quote!(vec2),
                            "Vec3" => quote!(vec3),
                            "Vec4" => quote!(vec4),
                            _ => unreachable!(),
                        };
                        return quote!(#vec_type<#inner_type>);
                    }
                }
            }
            abort!(
                Span::call_site(),
                "Vector types must specify their element type",
            );
        }
        "Mat2" => quote!(mat2x2),
        "Mat3" => quote!(mat3x3),
        "Mat4" => quote!(mat4x4),
        "Vec" => {
            abort!(Span::call_site(), "WGSL does not support Vec types.");
        }
        other => {
            abort!(Span::call_site(), "Unsupported type: {}", other);
        }
    }
}

fn rust_array_to_wgsl_array(ty: &Type) -> TokenStream {
    print!("array");
    match ty {
        Type::Array(array) => {
            let ty = convert_rust_type_to_wgsl(&array.elem);
            let len = &array.len;
            quote!(array<#ty, #len>)
        }
        _ => {
            abort!(Span::call_site(), "Expected array type");
        }
    }
}
