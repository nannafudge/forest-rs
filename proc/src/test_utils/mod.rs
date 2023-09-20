use crate::common::error_spanned;
use std::collections::BTreeSet;
use quote::ToTokens;
use syn::{
    Item, Result,
    spanned::Spanned
};

mod mocks;
mod test_case;
mod test_suite;

pub use mocks::get_mock;
pub use test_case::{
    TestCase, render_test_case
};
pub use test_suite::{
    TestSuite, render_test_suite
};

type Mutators<T> = BTreeSet<T>;

impl<T: Ord + Spanned + ToTokens> InsertUnique<T> for Mutators<T> {
    fn insert_unique(&mut self, item: T) -> Result<()> {
        let err = Err(error_spanned!("Duplicate argument: {}", &item));
        if !self.insert(item) {
            return err;
        }

        Ok(())
    }
}

trait Mutate {
    fn mutate(self, target: &mut Item);
}

trait InsertUnique<T> {
    fn insert_unique(&mut self, item: T) -> Result<()>;
}

#[macro_use]
mod macros {
    macro_rules! impl_unique_arg {
        ($target:ident $(< $generic:tt $(, $generics:tt)? >)?) => {
            impl $(< $generic $(, $generics)? >)? PartialEq for $target $(<$generic $(, $generics)?>)? {
                fn eq(&self, _: &Self) -> bool { true }
            }
            
            impl $(<$generic $(, $generics)?>)? Eq for $target $(<$generic $(, $generics)?>)? {

            }

            impl $(<$generic $(, $generics)?>)? PartialOrd for $target $(<$generic $(, $generics)?>)? {
                fn partial_cmp(&self, _: &Self) -> Option<core::cmp::Ordering> {
                    Some(core::cmp::Ordering::Equal)
                }
            }
            
            impl $(<$generic $(, $generics)?>)? Ord for $target $(<$generic $(, $generics)?>)? {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    self.partial_cmp(other).expect(
                        stringify!($target, ": Unexpected ord result")
                    )
                }
            }
        };
        ($target:ident $(< $generic:tt $(, $generics:tt)? >)?, $field:ident $(. $subfields:ident )?) => {
            impl $(< $generic $(, $generics)? >)? PartialEq for $target $(<$generic $(, $generics)?>)? {
                fn eq(&self, other: &Self) -> bool {
                    self.$field $(. $subfields)?.eq(&other.$field $(. $subfields)?)
                }
            }
            
            impl $(<$generic $(, $generics)?>)? Eq for $target $(<$generic $(, $generics)?>)? {

            }

            impl $(<$generic $(, $generics)?>)? PartialOrd for $target $(<$generic $(, $generics)?>)? {
                fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                    self.$field $(. $subfields)?.partial_cmp(&other.$field $(. $subfields)?)
                }
            }
            
            impl $(<$generic $(, $generics)?>)? Ord for $target $(<$generic $(, $generics)?>)? {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    self.partial_cmp(other).expect(
                        stringify!($target, ": Unexpected ord result")
                    )
                }
            }
        };
    }

    macro_rules! impl_to_tokens_wrapped {
        ($target:ty: collection) => {
            impl quote::ToTokens for $target {
                fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                    self.0.iter().for_each(| item | item.to_tokens(tokens));
                }
            }
        };
        ($target:ty, $field:ident $(. $subfields:ident )?: collection) => {
            impl quote::ToTokens for $target {
                fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                    self.$field $(. $subfields)?.iter().for_each(| item | item.to_tokens(tokens));
                }
            }
        };
        ($target:ty, $field:ident $(. $subfields:ident )?) => {
            impl quote::ToTokens for $target {
                fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                    self.$field $(. $subfields)?.to_tokens(tokens);
                }
            }
        };
        ($target:ty) => {
            impl quote::ToTokens for $target {
                fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                    self.0.to_tokens(tokens);
                }
            }
        };
    }

    pub(crate) use impl_unique_arg;
    pub(crate) use impl_to_tokens_wrapped;
}