extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{self};

/// 实现了 `struct A(B)` 的 `from_struct(b: B) -> A`
/// 和 `a.into_struct() -> B` 方法。
///
/// ## Examples
///
/// ```rust
///
/// #[derive(FromIntoStruct)]
/// struct A(B);
///
/// let b: B = /* B */
/// let a = A::from_struct(b);
/// let b = a.into_struct();
/// ```
#[proc_macro_derive(FromIntoStruct)]
pub fn from_into_derive(input: TokenStream) -> TokenStream {
  // Construct a representation of Rust code as a syntax tree
  // that we can manipulate
  let ast: syn::DeriveInput = syn::parse(input).unwrap();
  // parse_macro_input!(input as DeriveInput);
  // println!("{:?}", ast.data);

  // Build the trait implementation
  impl_from_into(&ast)
}

fn impl_from_into(ast: &syn::DeriveInput) -> TokenStream {
  let name = &ast.ident;
  // println!("{:?}", ast.data);
  // let mut expr: syn::Stmt = syn::parse_str("assert_eq!(u8::max_value(), 255)").unwrap();
  let mut st = String::from("");
  match &ast.data {
    syn::Data::Struct(data) => match &data.fields {
      syn::Fields::Unnamed(field) => {
        // println!("{:?}", field.unnamed.first());
        let token = field.unnamed.first().unwrap().into_token_stream().to_string();
        // let v: Vec<&str> = token.split(" ").collect();
        // println!("{:?}", v);
        st = token;
        // expr = syn::parse_str(&token).unwrap();
      }
      _ => (),
    },
    _ => (),
  };

  let st = st.trim_start_matches("pub ");
  // println!("{}", s);
  let st: syn::Expr = syn::parse_str(st).expect("FromIntoStruct Macro expanded error: expect a expr");
  // println!("{:#?}", st);

  let gen = quote! {
      impl #name {
        #[doc = concat!("from [", stringify!(#st), "] to [", stringify!(#name), "].")]
        pub fn from_struct(v: #st) -> Self {
          Self(v)
        }

        pub fn into_struct(&self) -> #st {
          self.0.clone()
        }
      }
  };
  gen.into() // 使用 quote! 可以定义我们想要返回的 Rust 代码。由于编译器需要的内容和 quote! 直接返回的不一样，因此还需要使用 .into 方法其转换为 TokenStream。
}
