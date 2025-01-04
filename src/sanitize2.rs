// use proc_macro2::{
//     token_stream::IntoIter, Delimiter, Group, Ident, Punct, Spacing, TokenStream, TokenTree,
// };

// /// Preprocessor for WGSL code that handles special directives and imports.
// /// Returns the processed token stream and optionally an identifier from the first #ident found.
// pub fn identify_special_directives(stream: TokenStream) -> (TokenStream, Option<Ident>) {
//     let mut processor = WGSLPreprocessor::new(stream);
//     processor.process()
// }

// struct WGSLPreprocessor {
//     processed_tokens: Vec<TokenTree>,
//     prev_token_was_hash: bool,
//     first_token: bool,
//     unprocessed_tokens: IntoIter,
// }

// impl WGSLPreprocessor {
//     fn new(stream: TokenStream) -> Self {
//         Self {
//             processed_tokens: Vec::new(),
//             prev_token_was_hash: false,
//             first_token: true,
//             unprocessed_tokens: stream.into_iter(),
//         }
//     }

//     fn process(mut self) -> (TokenStream, Option<Ident>) {
//         while let Some(token) = self.unprocessed_tokens.next() {
//             if let Some(ident) = self.process_token(token) {
//                 return (TokenStream::from_iter(self.processed_tokens), Some(ident));
//             }
//             self.first_token = false;
//         }
//         (TokenStream::from_iter(self.processed_tokens), None)
//     }

//     fn process_token(&mut self, token: TokenTree) -> Option<Ident> {
//         match token {
//             // Special case: Handle ifndef brackets at the start
//             TokenTree::Group(group)
//                 if self.first_token && group.delimiter() == Delimiter::Bracket =>
//             {
//                 self.processed_tokens.push(TokenTree::Group(group));
//                 None
//             }

//             // Handle hash character (#)
//             TokenTree::Punct(p) if p.as_char() == '#' => {
//                 self.prev_token_was_hash = true;
//                 None
//             }

//             // Handle naga_oil definitions (#define_import_path, #import, etc.)
//             #[cfg(feature = "naga_oil")]
//             TokenTree::Ident(ident)
//                 if self.prev_token_was_hash && is_naga_oil_directive(&ident) =>
//             {
//                 self.prev_token_was_hash = false;
//                 self.result
//                     .push(TokenTree::Punct(Punct::new('#', Spacing::Joint)));
//                 self.result.push(TokenTree::Ident(ident.clone()));
//                 None
//             }

//             // Handle WGSL import statements (#ident)
//             TokenTree::Ident(ident) if self.prev_token_was_hash => {
//                 self.processed_tokens.push(TokenTree::Ident(ident.clone()));
//                 process_remaining(self.unprocessed_tokens, &ident, &mut self.processed_tokens);
//                 Some(ident)
//             }

//             // Recursively process nested groups
//             TokenTree::Group(group) => {
//                 let delim = group.delimiter();
//                 let (processed_stream, found_ident) = identify_special_directives(group.stream());
//                 self.processed_tokens
//                     .push(TokenTree::Group(Group::new(delim, processed_stream)));

//                 if let Some(ident) = found_ident {
//                     process_remaining(self.unprocessed_tokens, &ident, &mut self.processed_tokens);
//                     Some(ident)
//                 } else {
//                     None
//                 }
//             }

//             // Handle all other tokens
//             other => {
//                 self.prev_token_was_hash = false;
//                 self.processed_tokens.push(other);
//                 None
//             }
//         }
//     }
// }

// /// Processes the remaining tokens after finding a #ident, removing duplicate #ident occurrences
// pub fn process_remaining(
//     unprocessed_tokens: IntoIter,
//     ident: &Ident,
//     processed_tokens: &mut Vec<TokenTree>,
// ) {
//     let mut processor = RemainingTokenProcessor::new(processed_tokens);
//     processor.process(unprocessed_tokens, ident);
// }

// struct RemainingTokenProcessor<'a> {
//     processed_tokens: &'a mut Vec<TokenTree>,
//     prev_token_wash_hash: bool,
// }

// impl<'a> RemainingTokenProcessor<'a> {
//     fn new(processed_tokens: &'a mut Vec<TokenTree>) -> Self {
//         Self {
//             processed_tokens,
//             prev_token_wash_hash: false,
//         }
//     }

//     fn process(&mut self, unprocessed_tokens: IntoIter, ident: &Ident) {
//         for token in unprocessed_tokens {
//             self.process_token(token, ident);
//         }
//     }

//     fn process_token(&mut self, token: TokenTree, ident: &Ident) {
//         match &token {
//             // Handle hash character
//             TokenTree::Punct(p) if p.as_char() == '#' => {
//                 self.prev_token_wash_hash = true;
//                 self.processed_tokens.push(token);
//             }

//             // Handle duplicate #ident
//             TokenTree::Ident(i) if self.prev_token_wash_hash && i == ident => {
//                 self.prev_token_wash_hash = false;
//                 self.processed_tokens.pop(); // Remove the previous #
//                 self.processed_tokens.push(token);
//             }

//             // Handle nested groups
//             TokenTree::Group(g) => {
//                 self.prev_token_wash_hash = false;
//                 let mut nested_items = Vec::new();
//                 let mut processor = RemainingTokenProcessor::new(&mut nested_items);
//                 processor.process(g.stream().into_iter(), ident);

//                 self.processed_tokens.push(TokenTree::Group(Group::new(
//                     g.delimiter(),
//                     TokenStream::from_iter(nested_items),
//                 )));
//             }

//             // Handle all other tokens
//             _ => {
//                 self.prev_token_wash_hash = false;
//                 self.processed_tokens.push(token);
//             }
//         }
//     }
// }

// #[cfg(feature = "naga_oil")]
// fn is_naga_oil_directive(name: &Ident) -> bool {
//     matches!(
//         name.to_string().as_str(),
//         "define_import_path" | "import" | "if" | "ifdef" | "ifndef" | "else" | "endif"
//     )
// }
