use super::{
    Mutate,
    impl_unique_arg
};
use proc_macro2::TokenStream;
use crate::common::{
    render_let_stmt,
    greedy_parse_with
};
use quote::{
    format_ident,
    ToTokens, TokenStreamExt,
};
use syn::{
    Ident, Expr,
    Type, FnArg,
    Item, ItemFn,
    Result, Token,
    parse::{
        Parse, ParseStream
    }
};

#[derive(Clone)]
pub struct ArgName(pub Ident);

impl Parse for ArgName {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self(input.parse::<Ident>()?))
    }
}

impl Mutate for ArgName {
    fn mutate(&self, target: &mut Item) {
        if let Item::Fn(function) = target {
            function.sig.ident = format_ident!("{}_{}", function.sig.ident, self.0);

            return;
        }

        panic!(
            "ArgName.print(): Expected syn::ItemFn, received syn::Item {:?}",
            core::mem::discriminant(target)
        );
    }
}

impl_unique_arg!(ArgName);

#[derive(Clone)]
pub(crate) struct ArgWith(Vec<Expr>);

impl Parse for ArgWith {
    fn parse(input: ParseStream) -> Result<Self> {
        let items: Vec<Expr> = greedy_parse_with(input, | input_after: ParseStream | {
            if !input_after.is_empty() {
                input_after.parse::<Token![,]>()?;
            }

            Ok(())
        })?;

        Ok(Self(items))
    }
}

impl Mutate for ArgWith {
    fn mutate(&self, target: &mut Item) {
        if let Item::Fn(function) = target {
            let mut new_fn_def: TokenStream = TokenStream::new();

            // Append existing attributes and visibility modifiers
            new_fn_def.append_all(function.attrs.iter());
            function.vis.to_tokens(&mut new_fn_def);

            // Steal inputs from signature, leaving the original function sig inputs empty
            let inputs = core::mem::take(&mut function.sig.inputs);
            function.sig.to_tokens(&mut new_fn_def);

            // Extract fn inputs, preserving their order, mapping such to their ident & type
            let mut input_map = inputs.iter().map(| arg | {
                if let FnArg::Typed(item) = arg {
                    if let syn::Pat::Ident(decl) = item.pat.as_ref() {
                        return (&decl.ident, &item.ty)
                    }
                }

                panic!("Unexpected function arg: syn::FnArg {:?}", core::mem::discriminant(arg))
            });
    
            // Insert input value statements parsed from attribute directly into fn body
            function.block.brace_token.surround(&mut new_fn_def, | test_body | {
                for stmt in &self.0 {
                    let stmt_meta: (&Ident, &Box<Type>) = input_map.next().expect(
                        &format!(
                            "No corresponding input argument defined on test function signature for {:?}",
                            stmt.to_token_stream()
                        )
                    );

                    render_let_stmt(&stmt_meta.0, &stmt_meta.1, stmt, test_body);
                }

                test_body.append_all(function.block.stmts.iter());
            });

            let new_fn: ItemFn = syn::parse2::<ItemFn>(new_fn_def)
                .expect("ArgWith.mutate(): Error creating new function def");

            // Replace the old function with the new mutation
            *target = Item::Fn(new_fn);

            return;
        }

        panic!("ArgWith.mutate(): expected function, received syn::Item {:?}", core::mem::discriminant(target));
    }
}

impl_unique_arg!(ArgWith);