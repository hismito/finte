use std::borrow::Cow;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

#[proc_macro_derive(IntEnum)]
pub fn derive_int_enum(input: TokenStream) -> TokenStream {
    let input = match parse_input(input) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error(),
    };

    match generate_impl(&input) {
        Ok(impls) => impls,
        Err(err) => err.to_compile_error(),
    }
}

fn parse_input(input: TokenStream) -> Result<DeriveInput, Error> {
    let mut tt_iter = input.into_iter().peekable();

    let mut repr = None;
    loop {
        match tt_iter.next().expect("`#` in `#[repr(_)]`") {
            TokenTree::Punct(punct) => {
                assert_eq!(punct.as_char(), '#');
            }
            TokenTree::Ident(ident) => match ident.to_string().as_str() {
                "pub" => {
                    if let Some(TokenTree::Group(_)) = tt_iter.peek() {
                        tt_iter.next().expect("vis path");
                    }
                    continue;
                }
                "enum" => {
                    break;
                }
                _ => {
                    return Err(Error::new(
                        ident.span(),
                        "unsupported type: try using an enum instead",
                    ));
                }
            },
            _ => panic!("expect outer meta"),
        }

        let stream = match tt_iter.next().expect("`repr` in #[repr(_)]`") {
            TokenTree::Group(group) => {
                assert_eq!(group.delimiter(), Delimiter::Bracket);
                group.stream()
            }
            _ => panic!("expect attr in #[repr(_)]"),
        };

        let mut tt_iter = stream.into_iter();
        let repr_group_stream = match tt_iter.next().expect("attr") {
            TokenTree::Ident(ident) => {
                if ident.to_string() == "repr" {
                    match tt_iter.next().expect("attr list") {
                        TokenTree::Group(group) => {
                            assert_eq!(group.delimiter(), Delimiter::Parenthesis);
                            group.stream()
                        }
                        _ => {
                            panic!("repr attr child should be group");
                        }
                    }
                } else {
                    continue;
                }
            }
            _ => continue,
        };

        let mut tt_iter = repr_group_stream.into_iter();
        match tt_iter.next().expect("repr type") {
            TokenTree::Ident(ident) => {
                if let Some(_) = repr.replace(ident) {
                    panic!("more than repr found");
                }
            }
            _ => {
                panic!("repr child shuld be ident");
            }
        }
        assert!(tt_iter.next().is_none(), "repr should have only child");
    }

    let repr = match repr {
        Some(repr) => repr,
        None => {
            return Err(Error::new(
                Span::call_site(),
                "no #[repr(_)] found: try adding one to specify the type for `IntEnum::Int`",
            ));
        }
    };

    let name = match tt_iter.next().expect("enum name") {
        TokenTree::Ident(ident) => ident,
        tt => return Err(Error::new(tt.span(), "expect enum name")),
    };

    let enum_item_tt = tt_iter.next().expect("enum definition body");
    let enum_item_group = match enum_item_tt {
        TokenTree::Group(group) => {
            assert_eq!(group.delimiter(), Delimiter::Brace);
            group.stream()
        }
        _ => panic!("enum items should reside in a group"),
    };

    let variants = {
        let mut variants = Vec::new();
        let mut tt_iter = enum_item_group.into_iter();
        loop {
            let variant_name = match tt_iter.by_ref().find_map(|tt| match tt {
                TokenTree::Ident(ident) => Some(ident),
                _ => None,
            }) {
                Some(ident) => ident,
                None => break,
            };

            match tt_iter.next().expect("`=`") {
                TokenTree::Punct(punct) => match punct.as_char() {
                    '=' => (),
                    _ => return Err(Error::new(punct.span(), "expect discriminant")),
                },
                tt => return Err(Error::new(tt.span(), "expect Punct(_) after variant name")),
            }

            let variant_value = match tt_iter.next().expect("variant value") {
                TokenTree::Literal(literal) => literal,
                tt => return Err(Error::new(tt.span(), "expect discriminant value after `=`")),
            };

            variants.push((variant_name, variant_value));
        }
        variants
    };

    if variants.is_empty() {
        Err(Error::new(Span::call_site(), "no variants"))
    } else {
        Ok(DeriveInput {
            int: repr,
            name,
            variants,
        })
    }
}

fn generate_impl(input: &DeriveInput) -> Result<TokenStream, Error> {
    use std::fmt::Write;

    let mut ts = String::new();

    writeln!(ts, "impl finte::IntEnum for {} {{", input.name).unwrap();

    writeln!(ts, "    type Int = {};", input.int).unwrap();

    {
        writeln!(
            ts,
            "    fn try_from_int(value: Self::Int) -> Option<Self> {{"
        )
        .unwrap();
        writeln!(ts, "        match value {{").unwrap();
        for (name, value) in input.variants.iter() {
            writeln!(ts, "            {} => Some(Self::{}),", value, name).unwrap();
        }
        writeln!(ts, "            _ => None,").unwrap();
        writeln!(ts, "        }}").unwrap();
        writeln!(ts, "    }}").unwrap();
    }

    {
        writeln!(ts, "    fn int_value(&self) -> Self::Int {{").unwrap();
        writeln!(ts, "        match self {{").unwrap();
        for (name, value) in input.variants.iter() {
            writeln!(ts, "            Self::{} => {},", name, value).unwrap();
        }
        writeln!(ts, "        }}").unwrap();
        writeln!(ts, "    }}").unwrap();
    }

    writeln!(ts, "}}").unwrap();

    Ok(ts.parse().unwrap())
}

#[derive(Debug)]
struct DeriveInput {
    int: Ident,
    name: Ident,
    variants: Vec<(Ident, Literal)>,
}

struct Error {
    span: Span,
    message: Cow<'static, str>,
}

impl Error {
    fn new(span: Span, message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            span,
            message: message.into(),
        }
    }

    fn to_compile_error(&self) -> TokenStream {
        fn with_span(span: Span, tt: impl Into<TokenTree>) -> TokenTree {
            let mut tt = tt.into();
            tt.set_span(span);
            tt
        }

        let mut stream: Vec<TokenTree> = Vec::new();

        stream.push(with_span(self.span, Ident::new("compile_error", self.span)));
        stream.push(with_span(self.span, Punct::new('!', Spacing::Alone)));
        stream.push(with_span(
            self.span,
            with_span(
                self.span,
                Group::new(
                    Delimiter::Parenthesis,
                    TokenTree::from(Literal::string(&self.message)).into(),
                ),
            ),
        ));
        stream.push(with_span(self.span, Punct::new(';', Spacing::Alone)));

        stream.into_iter().collect()
    }
}
