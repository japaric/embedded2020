use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{Ident, LitInt};

use crate::{
    fmt::Hex,
    ir::{Bitfield, Register, Width},
};

pub fn ident(s: &str) -> Ident {
    format_ident!(
        "{}",
        match s {
            // keywords
            "in" => "in_",
            _ => s,
        }
    )
}

pub fn bitwidth2ty(width: u8) -> TokenStream2 {
    if width <= 8 {
        quote!(u8)
    } else if width <= 16 {
        quote!(u16)
    } else if width <= 32 {
        quote!(u32)
    } else if width <= 64 {
        quote!(u64)
    } else {
        unreachable!()
    }
}

pub fn hex(val: u64) -> LitInt {
    LitInt::new(&Hex(val).to_string(), Span2::call_site())
}

pub fn r2wmask(reg: &Register<'_>) -> u64 {
    let mut mask = 0;
    for field in &reg.r_fields {
        if !reg.w_fields.contains(field) {
            mask |= field.mask() << field.offset;
        }
    }
    mask
}

pub fn unsuffixed(val: u8) -> LitInt {
    LitInt::new(&val.to_string(), Span2::call_site())
}

pub fn width2ty(width: Width) -> TokenStream2 {
    match width {
        Width::U8 => quote!(u8),
        Width::U16 => quote!(u16),
        Width::U32 => quote!(u32),
        Width::U64 => quote!(u64),
    }
}

pub fn field_docs(field: &Bitfield<'_>) -> String {
    let mut doc = if field.width == 1 {
        format!("(Bit {})", field.offset)
    } else {
        format!("(Bits {}..={})", field.offset, field.offset + field.width)
    };
    if let Some(desc) = field.description.as_ref() {
        doc.push(' ');
        doc.push_str(desc);
    }
    doc
}
