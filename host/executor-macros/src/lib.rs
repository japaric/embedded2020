#![deny(warnings)]

extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::Span as Span2;
use proc_macro_hack::proc_macro_hack;
use quote::{format_ident, quote};
use syn::{
    parse::{self, Parse, ParseBuffer},
    parse_macro_input,
    punctuated::Punctuated,
    Ident, Token,
};

#[proc_macro_hack]
pub fn run(input: TokenStream) -> TokenStream {
    let idents = parse_macro_input!(input as Input).idents;
    let ntasks = idents.len();

    if ntasks == 0 {
        return parse::Error::new(Span2::call_site(), "expected at least one task")
            .to_compile_error()
            .into();
    }

    let mut stmts = vec![];
    let mut polls = vec![];

    let krate = format_ident!("executor");

    // check that idents are futures and pin them
    for ident in idents.iter() {
        stmts.push(quote!(
            let mut #ident = #krate::check(#ident);
            // the future will never be moved
            let mut #ident = unsafe { core::pin::Pin::new_unchecked(&mut #ident) };
        ));

        polls.push(quote!(
            // XXX do we want to prevent futures being polled beyond completion?
            let _ = #ident.as_mut().poll(&mut cx);
        ));
    }

    stmts.push(quote!(
        let waker = #krate::waker();
        let mut cx = core::task::Context::from_waker(&waker);

        loop {
            use core::future::Future as _;

            #(#polls)*
            #krate::wfe();
        }
    ));

    quote!({
        #(#stmts)*
    })
    .into()
}

struct Input {
    idents: Punctuated<Ident, Token![,]>,
}

impl Parse for Input {
    fn parse(input: &ParseBuffer) -> parse::Result<Self> {
        Ok(Self {
            idents: Punctuated::parse_separated_nonempty(input)?,
        })
    }
}
