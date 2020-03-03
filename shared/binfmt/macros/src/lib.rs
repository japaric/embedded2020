use proc_macro::TokenStream;

use proc_macro2::Span as Span2;
use proc_macro_hack::proc_macro_hack;
use quote::{format_ident, quote};
use syn::{
    parse::{self, Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    Data, DeriveInput, Expr, Fields, GenericParam, LitInt, LitStr, Token,
};

#[proc_macro_derive(binDebug)]
pub fn debug(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut generics = input.generics;

    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(parse_quote!(binfmt::binDebug));
        }
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut stmts = vec![];
    let ident = &input.ident;
    let ts = match input.data {
        Data::Struct(data) => {
            let ident_s = ident.to_string();

            let footprint = match data.fields {
                Fields::Named(fields) => {
                    let mut fields_s = vec![];
                    for field in &fields.named {
                        let ident = field.ident.as_ref().expect("UNREACHABLE");
                        let name = ident.to_string();
                        let ty = &field.ty;

                        fields_s.push(format!("{}: {{}}", name));
                        stmts.push(quote!(
                            <#ty as binfmt::binDebug>::fmt(&self.#ident, f)
                        ));
                    }

                    format!("{} {{{{ {} }}}}", ident_s, fields_s.join(", "))
                }

                Fields::Unnamed(fields) => {
                    let mut fields_s = vec![];
                    for (i, field) in fields.unnamed.iter().enumerate() {
                        let ty = &field.ty;

                        let i = LitInt::new(&i.to_string(), Span2::call_site());
                        fields_s.push("{}");
                        stmts.push(quote!(
                            <#ty as binfmt::binDebug>::fmt(&self.#i, f)
                        ));
                    }

                    format!("{}({})", ident_s, fields_s.join(", "))
                }

                Fields::Unit => ident_s,
            };

            let section = format!(".binfmt.{}", footprint);
            quote!(
                impl #impl_generics binfmt::binDebug for #ident #ty_generics
                    #where_clause
                {
                    fn fmt(&self, f: &mut impl binfmt::binWrite) {
                        #[export_name = #footprint]
                        #[link_section = #section]
                        static SYM: u8 = 0;
                        f.write_footprint(&SYM);
                        #(#stmts;)*
                    }
                }

            )
        }

        _ => todo!(),
    };

    ts.into()
}

// #[proc_macro_hack]
// pub fn binwrite(input: TokenStream) -> TokenStream {
//     write_(parse_macro_input!(input as Input), false)
//         .unwrap_or_else(|e| e.to_compile_error().into())
// }

#[proc_macro_hack]
pub fn binwriteln(input: TokenStream) -> TokenStream {
    write_(parse_macro_input!(input as Input), true)
        .unwrap_or_else(|e| e.to_compile_error().into())
}

fn write_(input: Input, newline: bool) -> parse::Result<TokenStream> {
    let mut footprint = input.footprint.value();
    if newline {
        footprint.push('\n');
    }

    let span = input.footprint.span();
    let fargs = count_args(&footprint, span)?;
    let iargs = input.args.len();

    if fargs != iargs {
        return Err(parse::Error::new(
            span,
            &format!(
                "supplied args (n={}) don't match footprint args (n={})",
                iargs, fargs,
            ),
        ));
    }

    let section = format!(".binfmt.{}", footprint);
    let write = if input.args.is_empty() {
        quote!(
            #[export_name = #footprint]
            #[link_section = #section]
            static SYM: u8 = 0;
            <_ as binfmt::binWrite>::write_footprint(__f__, &SYM);
        )
    } else {
        let args = input.args.iter();
        let expr = quote!((#(&(#args),)*));

        let mut stmts = vec![];
        let mut pats = vec![];
        for i in 0..input.args.len() {
            let arg = format_ident!("arg{}", i);
            stmts.push(quote!(<_ as binfmt::binDebug>::fmt(#arg, __f__)));
            pats.push(arg);
        }
        quote!(
            match #expr {
                (#(#pats,)*) => {
                    #[export_name = #footprint]
                    #[link_section = #section]
                    static SYM: u8 = 0;
                    <_ as binfmt::binWrite>::write_footprint(__f__, &SYM);
                    #(#stmts;)*
                }
            }
        )
    };

    let formatter = &input.formatter;
    Ok(quote!(match #formatter {
        __f__ => {
            #write
        }
    })
    .into())
}

fn count_args(footprint: &str, span: Span2) -> parse::Result<usize> {
    let mut chars = footprint.chars().peekable();

    let mut nargs = 0;
    while let Some(c) = chars.next() {
        if c == '{' {
            let next = chars.peek();

            if next == Some(&'}') {
                drop(chars.next());

                nargs += 1;
            } else if next == Some(&'{') {
                // escaped brace
                drop(chars.next());
            } else {
                return Err(parse::Error::new(
                    span,
                    "unmatched `{`; use `{{` to escape it",
                ));
            }
        } else if c == '}' {
            let next = chars.peek();

            if next == Some(&'}') {
                // escaped brace
                drop(chars.next());
            } else {
                return Err(parse::Error::new(
                    span,
                    "unmatched `}`; use `}}` to escape it",
                ));
            }
        } else {
            // OK
        }
    }
    Ok(nargs)
}

struct Input {
    formatter: Expr,
    _comma1: Option<Token![,]>,
    footprint: LitStr,
    _comma2: Option<Token![,]>,
    args: Punctuated<Expr, Token![,]>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let formatter = input.parse()?;
        let _comma1 = input.parse()?;
        let footprint = input.parse()?;

        if input.is_empty() {
            Ok(Input {
                formatter,
                _comma1,
                footprint,
                _comma2: None,
                args: Punctuated::new(),
            })
        } else {
            Ok(Input {
                formatter,
                _comma1,
                footprint,
                _comma2: input.parse()?,
                args: Punctuated::parse_terminated(input)?,
            })
        }
    }
}
