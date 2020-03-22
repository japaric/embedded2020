extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::Span as Span2;
use proc_macro_hack::proc_macro_hack;
use quote::{format_ident, quote};
use syn::{
    parse::{self, Parse, ParseBuffer},
    parse_macro_input,
    punctuated::Punctuated,
    Ident, LitInt, Token,
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

    if ntasks > 4 {
        return parse::Error::new(
            Span2::call_site(),
            "only 4 concurrent tasks are supported at the moment",
        )
        .to_compile_error()
        .into();
    }

    let mut stmts = vec![];
    let mut arms = vec![];

    let krate = format_ident!("executor");

    // check that idents are futures and pin them
    for (i, ident) in idents.iter().enumerate() {
        stmts.push(quote!(
            let mut #ident = #krate::check(#ident);
            // the future will never be moved
            let mut #ident = unsafe { core::pin::Pin::new_unchecked(&mut #ident) };
        ));

        let i = LitInt::new(&i.to_string(), Span2::call_site());
        arms.push(quote!(
            #i => {
                // NOTE this clears the flag
                let waker = #krate::waker(flag);
                let mut cx = core::task::Context::from_waker(&waker);
                // XXX do we want to prevent futures being polled beyond completion?
                drop(#ident.as_mut().poll(&mut cx));
            }
        ));
    }

    // TODO add a drop guard to prevent `panic_unwind` breaking havoc
    if ntasks == 1 {
        let ident = &idents[0];

        stmts.push(quote!(
            static READY: core::sync::atomic::AtomicBool = {
                core::sync::atomic::AtomicBool::new(true)
            };

            loop {
                use core::future::Future as _;

                if READY.load(core::sync::atomic::Ordering::Relaxed) {
                    READY.store(false, core::sync::atomic::Ordering::Relaxed);

                    let waker = #krate::waker(&READY);
                    let mut cx = core::task::Context::from_waker(&waker);
                    drop(#ident.as_mut().poll(&mut cx));
                } else {
                    #krate::wfe();
                }
            }
        ));
    } else {
        if ntasks == 2 {
            stmts.push(quote!(static FLAGS: #krate::Flags2 = #krate::Flags2::new();));
        } else {
            stmts.push(quote!(static FLAGS: #krate::Flags4 = #krate::Flags4::new();));
        }

        stmts.push(quote!(
            loop {
                use core::future::Future as _;

                while let Some((i, flag)) = FLAGS.next() {
                    flag.store(false, core::sync::atomic::Ordering::Relaxed);

                    match i {
                        #(#arms)*
                        _ => unsafe { core::hint::unreachable_unchecked() },
                    }
                }

                #krate::wfe();
            }
        ));
    }

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
