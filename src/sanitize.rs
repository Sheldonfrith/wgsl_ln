use proc_macro2::{token_stream::IntoIter, Delimiter, Group, Ident, TokenStream, TokenTree};
/// Find the first instance of `#ident` and rewrite the macro as `__paste!(wgsl!())`.
pub fn sanitize_wgsl(stream: TokenStream) -> (TokenStream, Option<Ident>) {
    let mut result = Vec::new();
    let mut prev_token_was_hash_char = false;
    let mut iter: IntoIter = stream.into_iter();
    let mut first_token = true;
    while let Some(tt) = iter.next() {
        match tt {
            // ifndef
            TokenTree::Group(group) if first_token && group.delimiter() == Delimiter::Bracket => {
                result.push(TokenTree::Group(group));
            }
            // hash char tokens will never be added to the output except as part of a naga oil definition, as seen below.
            TokenTree::Punct(p) if p.as_char() == '#' => {
                prev_token_was_hash_char = true;
            }
            // if is a naga_oil definition, write `#def`
            #[cfg(feature = "naga_oil")]
            TokenTree::Ident(ident) if prev_token_was_hash_char && is_naga_oil_name(&ident) => {
                prev_token_was_hash_char = false;
                result.push(TokenTree::Punct(proc_macro2::Punct::new(
                    '#',
                    Spacing::Joint,
                )));
                result.push(TokenTree::Ident(ident.clone()));
            }
            // If # ident, this is a wgsl_ln import statement, so import it and remove duplicated `#`s if they exist.
            TokenTree::Ident(ident) if prev_token_was_hash_char => {
                result.push(TokenTree::Ident(ident.clone()));
                sanitize_remaining(iter, &ident, &mut result);
                return (TokenStream::from_iter(result), Some(ident));
            }
            // Recursively look for `#`s.
            TokenTree::Group(g) => {
                let delim = g.delimiter();
                let (stream, ident) = sanitize_wgsl(g.stream());
                result.push(TokenTree::Group(Group::new(delim, stream)));
                if let Some(ident) = ident {
                    sanitize_remaining(iter, &ident, &mut result);
                    return (TokenStream::from_iter(result), Some(ident));
                }
            }
            // copy non-special tokens to the results
            tt => {
                prev_token_was_hash_char = false;
                result.push(tt)
            }
        }
        first_token = false
    }
    (TokenStream::from_iter(result), None)
}

/// Remove duplicated `#`s from `# ident`s.
pub fn sanitize_remaining(stream: IntoIter, ident: &Ident, items: &mut Vec<TokenTree>) {
    let mut last_is_hash = false;
    for tt in stream {
        match &tt {
            // ifndef
            TokenTree::Punct(p) if p.as_char() == '#' => {
                last_is_hash = true;
                items.push(tt)
            }
            TokenTree::Ident(i) if last_is_hash && i == ident => {
                last_is_hash = false;
                let _ = items.pop();
                items.push(tt)
            }
            TokenTree::Group(g) => {
                last_is_hash = false;
                let mut stream = Vec::new();
                sanitize_remaining(g.stream().into_iter(), ident, &mut stream);
                items.push(TokenTree::Group(Group::new(
                    g.delimiter(),
                    TokenStream::from_iter(stream),
                )))
            }
            _ => {
                last_is_hash = false;
                items.push(tt)
            }
        }
    }
}

#[allow(dead_code)]
fn is_naga_oil_name(name: &Ident) -> bool {
    name == "define_import_path"
        || name == "import"
        || name == "if"
        || name == "ifdef"
        || name == "ifndef"
        || name == "else"
        || name == "endif"
}
