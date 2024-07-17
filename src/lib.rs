use proc_macro::TokenStream;

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use syn::{Error, Ident, Item, ItemFn, ItemType, parse_macro_input, Result};
use syn::spanned::Spanned;

#[proc_macro_attribute]
pub fn register(_args: TokenStream, input: TokenStream) -> TokenStream {
    let st = parse_macro_input!(input as Item);
    expand(&st).unwrap_or_else(Error::into_compile_error).into()
}

fn expand(st: &Item) -> Result<TokenStream2> {
    if let Item::Type(t) = st {
        let ident = &t.ident;
        let ident_name = ident.to_string();
        let static_ident = get_static_ident(ident);
        let ty = t.ty.clone();

        let vis = &t.vis;

        let impl_func = match *ty.clone() {
            syn::Type::BareFn(bf) => {
                let args_ty = bf.inputs.iter().map(|a| &a.ty);
                let args_name: Vec<_> = bf.inputs.iter().enumerate().map(|(idx, _)| {
                    format_ident!("__arg{}",idx)
                }).collect();

                let output = &bf.output;
                quote! {
                    pub fn unwrap_run(func:&str,#(#args_name:#args_ty),*) #output{
                        // let _register=#static_ident.get().unwrap().lock().unwrap();
                        // let f=_register.get(func).expect(format!("fn {} not found in {}", func,#ident_name).as_str());
                        let f=Self::get_fn(func).expect(format!("fn {} not found in {}", func,#ident_name).as_str());
                        f(#(#args_name),*)
                    }
                    pub fn get_fn(func:&str) -> Option<Box<#ty>>{
                        let _register=#static_ident.get().unwrap().lock().unwrap();
                        _register.get(func).map(|x|x.clone())
                    }
                }
            }
            _ => {
                // eprintln!("{:#?}", n);
                return !unimplemented!();
            }
        };



        return Ok(quote! {
            static #static_ident:std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<String,std::boxed::Box<#ty>>>>=std::sync::OnceLock::new();
            #vis struct #ident;

            impl #ident{
                #impl_func
            }
        });
    }
    Err(Error::new(st.span(), "only support type"))
}

fn expand_run_func(ty: &ItemType) -> Result<TokenStream2> {
    let ret = quote! {};
    Ok(ret)
}

fn get_static_ident(i: &Ident) -> Ident {
    let name = format!("__REGISTER_{}", i.to_string().to_uppercase());
    Ident::new(&name, i.span())
}

#[proc_macro_attribute]
pub fn register_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    let st = parse_macro_input!(input as Item);
    let args = parse_macro_input!(args as Ident);

    register_fn_expand(&st, &args).into()
}

fn register_fn_expand(st: &Item, args: &Ident) -> TokenStream2 {
    let mut ret = TokenStream2::new();
    ret.extend(st.to_token_stream());
    if let Item::Fn(ItemFn { sig, .. }) = st {
        // eprintln!("{:#?}", sig);
        let fn_ident = &sig.ident;
        let fn_name = fn_ident.to_string();
        let static_ident = get_static_ident(&args);
        let register_fn_ident = format_ident!("__register_{}",fn_ident);
        ret.extend(quote! {
            #[small_ctor::ctor]
            fn #register_fn_ident(){
                let _register = #static_ident.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
                _register.lock().unwrap().insert(#fn_name.to_string(), std::boxed::Box::new(#fn_ident));
            }
        });
    } else {
        ret.extend(Error::new(Span::call_site(), "Only support function").to_compile_error())
    }

    ret
}
