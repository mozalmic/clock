#![recursion_limit = "512"]

extern crate proc_macro;
use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(SnafuDebug)]
pub fn derive_parser(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    quote::quote! {
        impl ::std::fmt::Debug for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                ::std::writeln!(f, "{}", self)?;

                let mut error: &dyn ::std::error::Error = self;
                while let Some(source) = error.source() {
                    ::std::writeln!(f, "caused by: {}", source)?;
                    error = source;
                }

                if let Some(backtrace) = ::snafu::ErrorCompat::backtrace(&self) {
                    ::std::writeln!(f, "{}", backtrace)?;
                }

                Ok(())
            }
        }
    }
    .into()
}
