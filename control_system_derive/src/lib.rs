use core::panic;

use proc_macro2::{TokenStream, TokenTree, Ident};
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, Meta, MetaList};

#[derive(Clone, Debug)]
enum BlockIOAttribute {
    Name,
    Input { name: Option<String>, is_arr: bool },
    Output { name: Option<String>, is_arr: bool },
}

impl BlockIOAttribute {
    fn from_attribute(attr: Attribute) -> Option<Self> {
        match attr.meta {
            Meta::List(MetaList { path, tokens, .. }) => {
                if let Some(seg) = path.segments.first() {
                    if seg.ident == "blockio" {
                        Self::parse_tokens(tokens)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn parse_tokens(tokens: TokenStream) -> Option<Self> {
        let mut out: Option<Self> = None;
        let set = |dest: &mut Option<Self>, v: Self| {
            if dest.is_some() {
                panic!("Invalid tokens provided to 'blockio' attribute")
            }
            *dest = Some(v);
        };

        enum State {
            Ident,
            IOField(IOType, IOFieldState),
        }

        #[derive(Clone, Copy)]
        enum IOType {
            Input,
            InputArr,
            Output,
            OutputArr,
        }

        #[derive(PartialEq, Eq, Clone, Copy)]
        enum IOFieldState {
            Sep,
            NameKey,
            Equals,
            Literal,
            Done,
        }

        let mut state = State::Ident;

        for token in tokens {
            match state {
                State::Ident => match token {
                    TokenTree::Ident(ident) => match ident.to_string().as_str() {
                        "block_name" => set(&mut out, BlockIOAttribute::Name),
                        "input" => {
                            state = State::IOField(IOType::Input, IOFieldState::Sep);
                            set(
                                &mut out,
                                BlockIOAttribute::Input {
                                    name: None,
                                    is_arr: false,
                                },
                            );
                        }
                        "output" => {
                            state = State::IOField(IOType::Output, IOFieldState::Sep);
                            set(
                                &mut out,
                                BlockIOAttribute::Output {
                                    name: None,
                                    is_arr: false,
                                },
                            );
                        }
                        "input_arr" => {
                            state = State::IOField(IOType::InputArr, IOFieldState::Sep);
                            set(
                                &mut out,
                                BlockIOAttribute::Input {
                                    name: None,
                                    is_arr: true,
                                },
                            );
                        }
                        "output_arr" => {
                            state = State::IOField(IOType::OutputArr, IOFieldState::Sep);
                            set(
                                &mut out,
                                BlockIOAttribute::Output {
                                    name: None,
                                    is_arr: true,
                                },
                            );
                        }
                        _ => panic!("Unrecognized identifier in 'blockio' attribute: {}", ident),
                    },
                    _ => {
                        panic!("Missing identifier in 'blockio' attribute")
                    }
                },
                State::IOField(io, iostate) => {
                    match iostate {
                        IOFieldState::Sep => match token {
                            TokenTree::Punct(punct) => {
                                if punct.as_char() == ',' {
                                    state = State::IOField(io, IOFieldState::NameKey);
                                } else {
                                    panic!("Unexpected separator in 'blockio' attribute. Expeting ','.");
                                }
                            }
                            _ => panic!(
                                "Unexpected token in 'blockio' attribute. Expecting Punct(',')."
                            ),
                        },
                        IOFieldState::NameKey => match token {
                            TokenTree::Ident(ident) => {
                                if ident == "name" {
                                    state = State::IOField(io, IOFieldState::Equals);
                                } else {
                                    panic!("Unexpected ident in 'blockio' attribute '{}'. Expeting 'name'.", ident);
                                }
                            }
                            _ => {
                                panic!("Unexpected token in 'blockio' attribute. Expeting 'name'.")
                            }
                        },
                        IOFieldState::Equals => match token {
                            TokenTree::Punct(punct) => {
                                if punct.as_char() == '=' {
                                    state = State::IOField(io, IOFieldState::Literal);
                                } else {
                                    panic!("Unexpected separator in 'blockio' attribute. Expeting '='.");
                                }
                            }
                            _ => panic!(
                                "Unexpected token in 'blockio' attribute. Expecting Punct('=')."
                            ),
                        },
                        IOFieldState::Literal => match token {
                            TokenTree::Literal(literal) => match io {
                                IOType::Input => {
                                    out = Some(BlockIOAttribute::Input {
                                        name: Some(literal.to_string()),
                                        is_arr: false,
                                    })
                                }
                                IOType::Output => {
                                    out = Some(BlockIOAttribute::Output {
                                        name: Some(literal.to_string()),
                                        is_arr: false,
                                    })
                                }
                                IOType::InputArr => {
                                    out = Some(BlockIOAttribute::Input {
                                        name: Some(literal.to_string()),
                                        is_arr: true,
                                    })
                                }
                                IOType::OutputArr => {
                                    out = Some(BlockIOAttribute::Output {
                                        name: Some(literal.to_string()),
                                        is_arr: true,
                                    })
                                }
                            },
                            _ => panic!(
                                "Unexpected token in 'blockio' attribute. Expecting Literal."
                            ),
                        },
                        _ => {}
                    }
                }
            }
        }

        if let State::IOField(_, state) = state {
            if state != IOFieldState::Sep && state != IOFieldState::Done {
                panic!("Incorrect syntax for 'blockio' attribute");
            }
        }

        out
    }
}

fn parse_attributes(attrs: &[Attribute]) -> Option<BlockIOAttribute> {
    let mut out: Option<BlockIOAttribute> = None;
    for attr in attrs {
        let parsed = BlockIOAttribute::from_attribute(attr.clone());
        if parsed.is_some() && out.is_some() {
            panic!("Conflicting 'blockio' attributes found");
        }
        out = parsed;
    }
    out
}

fn quote_map_insert(ident: Ident, name: String, is_arr: bool) -> TokenStream {
    if is_arr {
        quote! {
            for (i, s) in self.#ident.iter_mut().enumerate() {
                assert!(hm.insert(format!("{}{}", #name, i + 1).to_string(), s.get_signal_mut()).is_none(), "Duplicate IO name: {}", #name);
            }
        }
    }else{
        quote! {
            assert!(hm.insert(#name.to_string(), self.#ident.get_signal_mut()).is_none(), "Duplicate IO name: {}", #name);
        }
    }
}

#[proc_macro_derive(BlockIO, attributes(blockio))]
pub fn derive(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(tokens as DeriveInput);
    
    let datastruct = match ast.data {
        Data::Struct(s) => s,
        Data::Enum(..) => panic!("Enums are not supported!"),
        Data::Union(..) => panic!("Unions are not supported!"),
    };

    let fields: Vec<_> = match datastruct.fields {
        Fields::Named(named_fields) => named_fields.named.iter().cloned().collect(),
        _ => panic!("Only named struct fields are supported"),
    };

    let mut name: Option<TokenStream> = None;
    let mut input_map: Vec<TokenStream> = vec![];
    let mut output_map: Vec<TokenStream> = vec![];

    for field in fields {
        let ident = field.ident.unwrap();
        if let Some(attr) = parse_attributes(&field.attrs) {
            match attr {
                BlockIOAttribute::Name => {
                    if name.is_some() {
                        panic!("Duplicate field with attribute 'name'");
                    }

                    name = Some(quote! {
                        fn name(&self) -> String {
                            self.#ident.to_string()
                        }
                    });
                }
                BlockIOAttribute::Input { name, is_arr } => {
                    let name = name.unwrap_or(ident.to_string());
                    
                    input_map.push(quote_map_insert(ident, name, is_arr));
                }
                BlockIOAttribute::Output { name, is_arr } => {
                    let name = name.unwrap_or(ident.to_string());
                    output_map.push(quote_map_insert(ident, name, is_arr));
                }
            }
        }
    }

    let struct_ident = ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let tokens = quote! {
        impl #impl_generics BlockIO for #struct_ident #ty_generics #where_clause {
            #name

            fn input_signals(&mut self) -> ::std::collections::HashMap<::std::string::String, &mut ::std::option::Option<::control_system::io::AnySignal>> {
                #![allow(unused_mut, clippy::let_and_return)]
                let mut hm = ::std::collections::HashMap::new();

                #( #input_map )*

                hm
            }

            fn output_signals(&mut self) -> ::std::collections::HashMap<::std::string::String, &mut ::control_system::io::AnySignal> {
                #![allow(unused_mut, clippy::let_and_return)]
                let mut hm = ::std::collections::HashMap::new();

                #( #output_map )*

                hm
            }
        }
    };

    tokens.into()
}
