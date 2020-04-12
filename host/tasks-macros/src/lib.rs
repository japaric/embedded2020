use proc_macro::TokenStream;

use proc_macro2::{Ident as Ident2, Span as Span2, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use std::time::SystemTime;
use syn::{
    parse::{self, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    Block, Expr, ItemMod, ItemUse, ReturnType, Stmt, Type, Visibility,
};

#[proc_macro_attribute]
pub fn declare(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return parse::Error::new(Span2::call_site(), "this attribute takes no arguments")
            .to_compile_error()
            .into();
    }

    let input = parse_macro_input!(input as Input);

    let mut items = vec![];
    let mut citems = vec![];
    let shared_args = &to_args(&input.statics).collect::<Vec<_>>();
    let shared_params = &to_params(&input.statics, false).collect::<Vec<_>>();

    citems.extend(to_statics(&input.statics));

    for (i, f) in input.fns.iter().enumerate() {
        let name = &f.name;
        let is_init = name.to_string() == "init";
        let params = to_params(&f.locals, is_init);
        let stmts = &f.stmts;
        let output = &f.output;
        let i = to_ident(i);
        items.push(quote!(
            #[inline(always)]
            fn #i(#(#shared_params,)* #(#params),*) #output {
                #(#stmts)*
            }
        ));

        let mut no_mangle = quote!(#[no_mangle]);
        let static_ = if is_init {
            no_mangle = quote!();
            let section = format!(".init.{}.{}", input.name, pseudo_rand());
            quote! (
                #[link_section = #section]
                #[used]
                static INIT: unsafe extern "C" fn() = #name;
            )
        } else {
            quote!()
        };
        let locals = to_statics(&f.locals);
        let args = to_args(&f.locals);
        citems.push(quote!(
            #static_

            #no_mangle
            unsafe extern "C" fn #name() {
                #(#locals)*

                drop(#i(#(#shared_args,)* #(#args),*))
            }
        ))
    }

    let name = &input.name;
    let uses = &input.uses;

    quote!(
        mod #name {
            #(#uses)*

            #(#items)*

            const _C: ()= {
                #(#citems)*
            };
        }
    ).into()
}

struct Input {
    fns: Vec<Fn>,
    name: Ident2,
    statics: Vec<Static>,
    uses: Vec<ItemUse>,
}

impl parse::Parse for Input {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let mod_: ItemMod = input.parse()?;

        if mod_.vis != Visibility::Inherited {
            return Err(parse::Error::new(mod_.span(), "module must be private"));
        }

        if !mod_.attrs.is_empty() {
            return Err(parse::Error::new(
                mod_.span(),
                "module must have no attributes",
            ));
        }

        if let Some((_, items)) = mod_.content {
            let mut fns = vec![];
            let mut statics = vec![];
            let mut uses = vec![];
            for item in items {
                match item {
                    syn::Item::Fn(f) => {
                        if !f.attrs.is_empty() {
                            return Err(parse::Error::new(
                                f.span(),
                                "function must have no attributes",
                            ));
                        }

                        if f.vis != Visibility::Inherited {
                            return Err(parse::Error::new(f.span(), "function must be private"));
                        }

                        let sig = &f.sig;

                        if sig.constness.is_some()
                            || sig.asyncness.is_some()
                            || sig.unsafety.is_some()
                            || sig.abi.is_some()
                            || !sig.generics.params.is_empty()
                            || sig.generics.where_clause.is_some()
                            || !sig.inputs.is_empty()
                        {
                            return Err(parse::Error::new(
                                f.span(),
                                "function must have signature `fn() -> T`",
                            ));
                        }

                        let (locals, stmts) = split(f.block)?;

                        fns.push(Fn {
                            locals,
                            name: f.sig.ident,
                            output: f.sig.output,
                            stmts,
                        });
                    }

                    syn::Item::Static(s) => statics.push(verify(s)?),

                    syn::Item::Use(u) => uses.push(u),

                    _ => {
                        return Err(parse::Error::new(
                            item.span(),
                            "module must only contain functions and static variables",
                        ))
                    }
                }
            }

            Ok(Input {
                fns,
                name: mod_.ident,
                statics,
                uses,
            })
        } else {
            Err(parse::Error::new(mod_.span(), "module must be inline"))
        }
    }
}

fn split(block: Box<Block>) -> parse::Result<(Vec<Static>, Vec<Stmt>)> {
    let mut istmts = block.stmts.into_iter();
    let mut stmts = vec![];
    let mut statics = vec![];

    while let Some(stmt) = istmts.next() {
        if let Stmt::Item(syn::Item::Static(s)) = stmt {
            statics.push(verify(s)?);
        } else {
            stmts.push(stmt);
            break;
        }
    }

    stmts.extend(istmts);

    Ok((statics, stmts))
}

fn verify(s: syn::ItemStatic) -> parse::Result<Static> {
    let span = s.span();
    let mut attrs = s.attrs;

    let mut uninit = false;
    if let Some(pos) = attrs.iter().position(|attr| {
        attr.path
            .get_ident()
            .map(|id| id == "uninit")
            .unwrap_or(false)
    }) {
        let attr = &attrs[pos];
        let span = attr.span();

        if attr.tokens.to_string() == "(unsafe)" {
            attrs.remove(pos);
            uninit = true;
        } else {
            return Err(parse::Error::new(
                span,
                format!(
                    "`#[uninit]` attribute must contain the `unsafe` keyword: `#[uninit(unsafe)]`"
                ),
            ));
        }
    }

    if !attrs.is_empty() {
        return Err(parse::Error::new(
            span,
            "function must have no attributes other than `#[uninit]`",
        ));
    }

    if s.vis != Visibility::Inherited {
        return Err(parse::Error::new(span, "static must be private"));
    }

    if s.mutability.is_none() {
        return Err(parse::Error::new(span, "static must be mutable"));
    }

    Ok(Static {
        expr: s.expr,
        name: s.ident,
        ty: s.ty,
        uninit,
    })
}

fn to_args<'s>(ss: &'s [Static]) -> impl Iterator<Item = TokenStream2> + 's {
    ss.iter().map(|s| {
        let name = &s.name;
        quote!(&mut #name)
    })
}

fn to_ident(i: usize) -> Ident2 {
    format_ident!("_{}", i)
}

fn to_params<'s>(ss: &'s [Static], is_init: bool) -> impl Iterator<Item = TokenStream2> + 's {
    ss.iter().map(move |s| {
        let name = &s.name;
        let ty = &s.ty;
        let lt = if is_init { quote!('static) } else { quote!() };
        quote!(
            #[allow(unused_variables)]
            #[allow(non_snake_case)]
            #name : &#lt mut #ty)
    })
}

fn to_statics<'s>(ss: &'s [Static]) -> impl Iterator<Item = TokenStream2> + 's {
    ss.iter().map(|s| {
        let name = &s.name;
        let ty = &s.ty;
        let expr = &s.expr;
        let attr = if s.uninit {
            let section = format!(".uninit.{}.{}", name, pseudo_rand());
            quote!(#[link_section = #section])
        } else {
            quote!()
        };
        quote!(
            #attr
            static mut #name: #ty = #expr;
        )
    })
}

fn pseudo_rand() -> u32 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos()
}

struct Fn {
    locals: Vec<Static>,
    name: Ident2,
    output: ReturnType,
    stmts: Vec<Stmt>,
}

struct Static {
    expr: Box<Expr>,
    name: Ident2,
    ty: Box<Type>,
    uninit: bool,
}
