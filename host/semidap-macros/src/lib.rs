#![deny(warnings)]

extern crate proc_macro;

use core::{
    mem,
    sync::atomic::{AtomicUsize, Ordering},
};
use proc_macro::TokenStream;
use std::{borrow::Cow, time::SystemTime};

use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use proc_macro_hack::proc_macro_hack;
use quote::quote;
use syn::{
    parse::{self, Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, LitStr, Token,
};

const THRESHOLD: usize = 4;

#[proc_macro_hack]
pub fn print(input: TokenStream) -> TokenStream {
    write(None, input, false)
}

#[proc_macro_hack]
pub fn println(input: TokenStream) -> TokenStream {
    write(None, input, true)
}

#[proc_macro_hack]
pub fn error(input: TokenStream) -> TokenStream {
    write(Some(Level::Error), input, true)
}

#[proc_macro_hack]
pub fn warn(input: TokenStream) -> TokenStream {
    write(Some(Level::Warn), input, true)
}

#[proc_macro_hack]
pub fn info(input: TokenStream) -> TokenStream {
    write(Some(Level::Info), input, true)
}

#[proc_macro_hack]
pub fn debug(input: TokenStream) -> TokenStream {
    write(Some(Level::Debug), input, true)
}

#[proc_macro_hack]
pub fn trace(input: TokenStream) -> TokenStream {
    write(Some(Level::Trace), input, true)
}

#[derive(Clone, Copy)]
#[repr(u8)]
enum Level {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

impl Level {
    fn needs_debug_assertions(&self) -> bool {
        match self {
            Level::Debug | Level::Trace => true,
            _ => false,
        }
    }
}

fn write(
    level: Option<Level>,
    input: TokenStream,
    newline: bool,
) -> TokenStream {
    let input = parse_macro_input!(input as Input);

    match log_(level, input, newline) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn log_(
    level: Option<Level>,
    input: Input,
    newline: bool,
) -> parse::Result<TokenStream2> {
    let mut lit = input.literal.value();
    if newline {
        lit.push('\n');
    }
    let mut args = input.args.into_iter();
    let pieces = parsel(&lit, input.literal.span())?;

    let mut stmts = vec![];
    if let Some(level) = level {
        stmts.push(quote!(__stdout__.write_timestamp();));
        let sym = level as usize;
        stmts.push(quote!(__stdout__.write_sym(#sym as *const u8);));
    }
    stmts.extend(pieces.into_iter().map(|piece| match piece {
        Piece::Debug { pretty } => {
            let arg = args.next();
            if pretty {
                quote!(let _ = ufmt::uwrite!(__stdout__, "{:#?}", #arg);)
            } else {
                quote!(let _ = ufmt::uwrite!(__stdout__, "{:?}", #arg);)
            }
        }

        Piece::Display => {
            let arg = args.next();
            quote!(let _ = ufmt::uwrite!(__stdout__, "{}", #arg);)
        }

        Piece::Str(s) => {
            if s.as_bytes().len() >= THRESHOLD {
                let i = count();
                // attach fake version to avoid problems with duplicated symbols
                // XXX another solution to symbol duplication could be marking
                // the symbol as "weak"; this would let the linker pick one
                // among the duplicates (I think). In any case, the `#[linkage]`
                // attribute is perma-unstable
                let nanos = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map(|dur| dur.subsec_nanos() as usize)
                    .unwrap_or(i);
                let s = format!("{}@{}", s, nanos);
                let sect = format!(".log.{}", i);
                quote!(__stdout__.write_sym({
                        #[link_section = #sect]
                        #[export_name = #s]
                        static SYM: u8 = 0;

                        &SYM as *const u8
                    });)
            } else {
                quote!(__stdout__.write_str(#s);)
            }
        }
    }));

    let debug_assertions = if level
        .as_ref()
        .map(|level| level.needs_debug_assertions())
        .unwrap_or(false)
    {
        quote!(cfg!(debug_assertions))
    } else {
        quote!(true)
    };

    Ok(quote!(if #debug_assertions {
        if let Some(mut __stdout__) = semidap::stdout() {
            #(#stmts)*
            __stdout__.flush();
        }
    }))
}

fn count() -> usize {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    COUNT.fetch_add(1, Ordering::Relaxed)
}

struct Input {
    literal: LitStr,
    _comma: Option<Token![,]>,
    args: Punctuated<Expr, Token![,]>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let literal = input.parse()?;

        if input.is_empty() {
            Ok(Input {
                literal,
                _comma: None,
                args: Punctuated::new(),
            })
        } else {
            Ok(Input {
                literal,
                _comma: input.parse()?,
                args: Punctuated::parse_terminated(input)?,
            })
        }
    }
}

// Copy-paste from ufmt-macros v0.1.1
fn parsel<'l>(
    mut literal: &'l str,
    span: Span2,
) -> parse::Result<Vec<Piece<'l>>> {
    let mut pieces = vec![];

    let mut buf = String::new();
    loop {
        let mut parts = literal.splitn(2, '{');
        match (parts.next(), parts.next()) {
            // empty string literal
            (None, None) => break,

            // end of the string literal
            (Some(s), None) => {
                if buf.is_empty() {
                    if !s.is_empty() {
                        pieces.push(Piece::Str(unescape(s, span)?));
                    }
                } else {
                    buf.push_str(&unescape(s, span)?);

                    pieces.push(Piece::Str(Cow::Owned(buf)));
                }

                break;
            }

            (head, Some(tail)) => {
                const DEBUG: &str = ":?}";
                const DEBUG_PRETTY: &str = ":#?}";
                const DISPLAY: &str = "}";
                const ESCAPED_BRACE: &str = "{";

                let head = head.unwrap_or("");
                if tail.starts_with(DEBUG)
                    || tail.starts_with(DEBUG_PRETTY)
                    || tail.starts_with(DISPLAY)
                {
                    if buf.is_empty() {
                        if !head.is_empty() {
                            pieces.push(Piece::Str(unescape(head, span)?));
                        }
                    } else {
                        buf.push_str(&unescape(head, span)?);

                        pieces.push(Piece::Str(Cow::Owned(mem::replace(
                            &mut buf,
                            String::new(),
                        ))));
                    }

                    if tail.starts_with(DEBUG) {
                        pieces.push(Piece::Debug { pretty: false });

                        literal = &tail[DEBUG.len()..];
                    } else if tail.starts_with(DEBUG_PRETTY) {
                        pieces.push(Piece::Debug { pretty: true });

                        literal = &tail[DEBUG_PRETTY.len()..];
                    } else {
                        pieces.push(Piece::Display);

                        literal = &tail[DISPLAY.len()..];
                    }
                } else if tail.starts_with(ESCAPED_BRACE) {
                    buf.push_str(&unescape(head, span)?);
                    buf.push('{');

                    literal = &tail[ESCAPED_BRACE.len()..];
                } else {
                    return Err(parse::Error::new(
                        span,
                        "invalid format string: expected `{{`, `{}`, `{:?}` or `{:#?}`",
                    ));
                }
            }
        }
    }

    Ok(pieces)
}

// `}}` -> `}`
fn unescape<'l>(
    mut literal: &'l str,
    span: Span2,
) -> parse::Result<Cow<'l, str>> {
    if literal.contains('}') {
        let mut buf = String::new();

        while literal.contains('}') {
            const ERR: &str = "format string contains an unmatched right brace";
            let mut parts = literal.splitn(2, '}');

            match (parts.next(), parts.next()) {
                (Some(left), Some(right)) => {
                    const ESCAPED_BRACE: &str = "}";

                    if right.starts_with(ESCAPED_BRACE) {
                        buf.push_str(left);
                        buf.push('}');

                        literal = &right[ESCAPED_BRACE.len()..];
                    } else {
                        return Err(parse::Error::new(span, ERR));
                    }
                }

                _ => unreachable!(),
            }
        }

        buf.push_str(literal);

        Ok(buf.into())
    } else {
        Ok(Cow::Borrowed(literal))
    }
}

#[derive(Debug, PartialEq)]
enum Piece<'a> {
    Debug { pretty: bool },
    Display,
    Str(Cow<'a, str>),
}
