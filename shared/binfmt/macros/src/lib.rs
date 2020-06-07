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
    let (tag, footprint) = match input.data {
        Data::Enum(data) => {
            if data
                .variants
                .iter()
                .all(|variant| variant.fields == Fields::Unit)
                && data.variants.len() < 256
            {
                let mut variants = vec![];
                let mut arms = vec![];
                for (i, variant) in data.variants.iter().enumerate() {
                    variants.push(variant.ident.to_string());
                    let vident = &variant.ident;
                    arms.push(quote!(#ident::#vident => #i as u8))
                }

                stmts.push(quote!(
                    f.write_byte(match self { #(#arms),* });
                ));

                let footprint = variants.join(",");
                (quote!(CLikeEnum), footprint)
            } else {
                return parse::Error::new(ident.span(), "this data type is not supported")
                    .to_compile_error()
                    .into();
            }
        }

        Data::Union(_) => {
            return parse::Error::new(ident.span(), "this data type is not supported")
                .to_compile_error()
                .into();
        }

        Data::Struct(data) => {
            let ident_s = ident.to_string();

            match data.fields {
                Fields::Named(fields) => {
                    // TODO implement bitfield compression for structs that
                    // contain non-bool fields
                    if fields.named.iter().all(|f| f.ty == parse_quote!(bool)) {
                        let mut fields_s = vec![];
                        let mut exprs = vec![];
                        let n = fields.named.len();
                        for (i, field) in fields.named.iter().enumerate() {
                            let ident = field.ident.as_ref().expect("UNREACHABLE");
                            let name = ident.to_string();

                            fields_s.push(format!("{}: {{{}}}", name, i));
                            exprs.push(quote!(if self.#ident {{ 1 }} else {{ 0 }} << #i));
                        }

                        if n <= 8 {
                            stmts.push(quote!(f.write(&u8::to_le_bytes(#(#exprs)|*))));
                        } else if n <= 16 {
                            stmts.push(quote!(f.write(&u16::to_le_bytes(#(#exprs)|*))));
                        } else {
                            todo!()
                        }

                        (
                            quote!(Register),
                            format!("{} {{{{ {} }}}}", ident_s, fields_s.join(", ")),
                        )
                    } else {
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

                        (
                            quote!(Footprint),
                            format!("{} {{{{ {} }}}}", ident_s, fields_s.join(", ")),
                        )
                    }
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

                    (
                        quote!(Footprint),
                        format!("{}({})", ident_s, fields_s.join(", ")),
                    )
                }

                Fields::Unit => (quote!(Footprint), ident_s),
            }
        }
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
                f.write_byte(binfmt::Tag::#tag as u8);
                f.write_sym(&SYM);
                #(#stmts;)*
            }
        }
    )
    .into()
}

#[proc_macro_hack]
pub fn binwrite(input: TokenStream) -> TokenStream {
    write_(parse_macro_input!(input as Input), false, false)
        .unwrap_or_else(|e| e.to_compile_error().into())
}

fn write_(input: Input, newline: bool, tag: bool) -> parse::Result<TokenStream> {
    let mut footprint = input.footprint.value();

    let span = input.footprint.span();
    if footprint.contains('@') {
        return Err(parse::Error::new(span, "`@` character is not allowed"));
    }

    if newline {
        footprint.push('\n');
    }

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
    // add random version to the symbol to avoid linker error due to duplicates
    let footprint = format!("{}@{}", footprint, rand::random::<u64>());
    let tag = if tag {
        Some(quote!(<_ as binfmt::binWrite>::write_byte(
            __f__,
            binfmt::Tag::Footprint as u8,
        );))
    } else {
        None
    };
    let write = if input.args.is_empty() {
        quote!(
            #[export_name = #footprint]
            #[link_section = #section]
            static SYM: u8 = 0;
            #tag
            <_ as binfmt::binWrite>::write_sym(__f__, &SYM);
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
                    #tag
                    <_ as binfmt::binWrite>::write_sym(__f__, &SYM);
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
                let _ = chars.next();

                nargs += 1;
            } else if next == Some(&'{') {
                // escaped brace
                let _ = chars.next();
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
                let _ = chars.next();
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
