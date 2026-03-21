use proc_macro::TokenStream;
use quote::quote;
use syn::fold::{self, Fold};
use syn::{parse_macro_input, BinOp, Expr, ExprBinary, ExprLit, FnArg, Ident, Lit, Pat};

/// A very basic macro to evaluate a math expression at compile time.
#[proc_macro]
pub fn eval_math(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    fn eval(expr: &Expr) -> i64 {
        match expr {
            Expr::Lit(ExprLit {
                lit: Lit::Int(i), ..
            }) => i.base10_parse().unwrap(),
            Expr::Binary(ExprBinary {
                left, op, right, ..
            }) => {
                let l = eval(left);
                let r = eval(right);
                match op {
                    BinOp::Add(_) => l + r,
                    BinOp::Sub(_) => l - r,
                    BinOp::Mul(_) => l * r,
                    BinOp::Div(_) => l / r,
                    _ => panic!("Unsupported operator"),
                }
            }
            Expr::Group(g) => eval(&g.expr),
            Expr::Paren(p) => eval(&p.expr),
            _ => panic!("Unsupported expression"),
        }
    }

    let result = eval(&expr);
    quote! { #result }.into()
}

struct Interpolator {
    arg_names: Vec<Ident>,
}

impl Fold for Interpolator {
    fn fold_expr(&mut self, i: Expr) -> Expr {
        if let Expr::Path(ref p) = i {
            if let Some(ident) = p.path.get_ident() {
                if self.arg_names.contains(ident) {
                    // Verbatim is a way to tell syn "don't parse this, just emit these tokens".
                    // Here we emit '#' then the identifier, which quote! in the codegen
                    // function will interpret as interpolation.
                    let pound = proc_macro2::Punct::new('#', proc_macro2::Spacing::Alone);
                    let mut tokens = proc_macro2::TokenStream::new();
                    tokens.extend(quote!(#pound #ident));
                    return Expr::Verbatim(tokens);
                }
            }
        }
        fold::fold_expr(self, i)
    }
}

/// The fsdr_instantiate macro.
#[proc_macro_attribute]
pub fn fsdr_instantiate(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as syn::ItemFn);

    // 1. Keep the original function (for RuntimeBackend)
    let original_fn = &input_fn;

    // 2. Identify function arguments
    let mut arg_names = Vec::new();
    for arg in &input_fn.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                arg_names.push(pat_ident.ident.clone());
            }
        }
    }

    // 3. Transform the function body for codegen
    let mut interpolator = Interpolator { arg_names };
    let transformed_block = interpolator.fold_block(*input_fn.block.clone());

    // 4. Generate the Codegen version
    let fn_name = &input_fn.sig.ident;
    let codegen_fn_name = quote::format_ident!("{}_codegen", fn_name);
    let fn_args = &input_fn.sig.inputs;
    let stmts = &transformed_block.stmts;

    let expanded = quote! {
        #original_fn

        #[allow(unused_variables)]
        pub fn #codegen_fn_name(#fn_args) -> proc_macro2::TokenStream {
            // By using transformed_block, which contains #ident sequences,
            // this quote! will interpolate the values of the arguments.
            quote::quote! { #(#stmts)* }
        }
    };

    expanded.into()
}
