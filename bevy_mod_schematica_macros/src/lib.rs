use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput, GenericParam, Generics,
    Index,
};

#[proc_macro_derive(Schematic)]
pub fn dervie_schematic(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let instantiate = instantiate_impl(&input.data);

    let expanded = quote! {
        impl #impl_generics ::bevy_mod_schematica::Schematic for #name #ty_generics #where_clause {
            fn instantiate(self, mut ctx: &mut ::bevy_mod_schematica::SchematicContext) -> ::bevy_mod_schematica::SchematicResult {
                #instantiate
            }
        }
    };

    expanded.into()
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param
                .bounds
                .push(parse_quote!(::bevy_mod_schematica::Schematic));
        }
    }

    generics
}

fn instantiate_impl(data: &Data) -> TokenStream2 {
    match data {
        Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => {
                let operations = fields.named.iter().map(|f| {
                    let name = &f.ident;

                    quote_spanned! {f.span() =>
                        self.#name.instantiate(&mut ctx)?;
                    }
                });

                quote! {
                    #(#operations)*
                    Ok(())
                }
            }
            syn::Fields::Unnamed(fields) => {
                let operations = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let index = Index::from(i);

                    quote_spanned! {f.span() =>
                        self.#index.instantiate(ctx)?;
                    }
                });

                quote! {
                    #(#operations)*
                    Ok(())
                }
            }
            syn::Fields::Unit => unimplemented!(),
        },
        Data::Union(_) | Data::Enum(_) => unimplemented!(),
    }
}
