extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, ItemStruct, LitInt};

#[proc_macro_derive(CodecDecode)]
pub fn decode_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;
    let mut fields = Vec::new();
    if let syn::Data::Struct(data) = input.data {
        if let syn::Fields::Named(fields_named) = data.fields {
            for field in fields_named.named {
                let field_name = field.ident.unwrap();
                fields.push(field_name);
            }
        }
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let expanded = quote! {
        impl #impl_generics gyra_codec::coding::Decoder for #name #ty_generics #where_clause {
            fn decode<R: std::io::Read>(reader: &mut R) -> gyra_codec::error::Result<Self> {
                Ok(Self {
                    #(
                        #fields: <_ as gyra_codec::coding::Decoder>::decode(reader).map_err(|e| gyra_codec::error::CodecError::CantParseField {
                            field: stringify!(#fields).to_string(),
                            source: Box::new(e),
                        })?,
                    )*
                })
            }
        }
    };
    expanded.into()
}

#[proc_macro_derive(CodecEncode)]
pub fn encode_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;
    let mut fields = Vec::new();
    if let syn::Data::Struct(data) = input.data {
        if let syn::Fields::Named(fields_named) = data.fields {
            for field in fields_named.named {
                let field_name = field.ident.unwrap();
                fields.push(field_name);
            }
        }
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let expanded = quote! {
        impl #impl_generics gyra_codec::coding::Encoder for #name #ty_generics #where_clause {
            fn encode<W: std::io::Write>(&self, writer: &mut W) -> gyra_codec::error::Result<usize> {
                let mut bytes_written = 0;
                #(
                    bytes_written += <_ as gyra_codec::coding::Encoder>::encode(&self.#fields, writer)?;
                )*
                Ok(bytes_written)
            }
        }
    };
    expanded.into()
}

#[derive(Clone)]
struct PacketArgs {
    pub id: u32,
    pub when: syn::Ident,
    pub direction: TokenStream,
}

impl Parse for PacketArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut packet_id: Option<LitInt> = None;
        let mut when_id: Option<syn::Ident> = None;
        let mut direction = quote! {ToClient};

        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(syn::Ident) {
                let ident = input.parse::<syn::Ident>()?;
                if ident == "id" {
                    input.parse::<syn::Token![:]>()?;
                    let id = input.parse::<LitInt>()?;
                    packet_id = Some(id);
                }
                if ident == "when" {
                    input.parse::<syn::Token![:]>()?;
                    let when = input.parse::<syn::Ident>()?;
                    when_id = Some(when);
                }
                if ident == "server" {
                    direction = quote! {ToServer};
                }
            } else {
                let _ = input.parse::<proc_macro2::TokenTree>();
            }
        }

        match (packet_id, when_id) {
            (None, _) => Err(syn::Error::new(
                input.span(),
                "Packet attribute requires an id",
            )),
            (_, None) => Err(syn::Error::new(
                input.span(),
                "Packet attribute requires a when",
            )),
            (Some(packet_id), Some(when_id)) => Ok(PacketArgs {
                direction,
                id: packet_id.base10_parse().expect("i expect a number"),
                when: when_id,
            }),
        }
    }
}

#[proc_macro_attribute]
pub fn packet(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if args.is_empty() {
        return quote! {
           compile_error!("Packet attribute requires arguments");
        }
        .into();
    }

    let args = parse_macro_input!(args as PacketArgs);
    let item_struct = parse_macro_input!(input as ItemStruct);
    let ident = item_struct.ident.clone();

    let packet_id = args.id;
    let when = args.when;
    let direction = args.direction;

    quote! {
        #item_struct

        impl gyra_codec::packet::Packet for #ident {
            const ID: gyra_codec::packet::PacketId = #packet_id;
            const WHEN: gyra_codec::packet::When = gyra_codec::packet::When::#when;
            const DIRECTION: gyra_codec::packet::Direction = gyra_codec::packet::Direction::#direction;
        }
    }.into()
}
