//! Exports procedural macros for compile-time string literal hashing.

use {
    proc_macro::{Literal, TokenStream, TokenTree},
    std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    },
};

trait ToLiteral {
    fn to_literal(&self) -> Literal;
}

impl ToLiteral for u64 {
    fn to_literal(&self) -> Literal {
        Literal::u64_unsuffixed(*self)
    }
}

impl ToLiteral for u32 {
    fn to_literal(&self) -> Literal {
        Literal::u32_unsuffixed(*self)
    }
}

/// Hashes the string to a `u64` using the Rust's default hasher (i.e. one used in the `HashMap`).
fn string_hash_default(string: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    string.hash(&mut hasher);
    hasher.finish()
}

/// Hashes the string to a `u32` using FNV1a hash.
fn string_hash_fnv1a(string: &str) -> u32 {
    const FNV1A_PRIME: u32 = 0x0100_0193;
    const FNV1A_SEED: u32 = 0x811C_9DC5;

    let mut hash = FNV1A_SEED;

    for byte in string.as_bytes() {
        hash = (hash ^ *byte as u32).wrapping_mul(FNV1A_PRIME);
    }

    hash
}

fn str_hash_impl<H: ToLiteral>(item: TokenStream, hash: fn(&str) -> H) -> TokenStream {
    let mut iter = item.into_iter();

    let string = iter.next().expect("`string_hash` macro takes one non-empty quoted string literal argument - none were provided");

    let result: TokenStream;

    match string {
        TokenTree::Literal(string) => {
            // At least [" "].
            let string = string.to_string();
            assert!(string.len() >= 3, "`string_hash` macro takes one non-empty quoted string literal argument - `{}` was provided", string);

            let first_char = &string[0..1];
            assert!(first_char == "\"", "`string_hash` macro takes one non-empty quoted string literal argument - `{}` does not start with a quote", string);

            let last_char = &string[string.len() - 1..string.len()];
            assert!(last_char == "\"", "`string_hash` macro takes one non-empty quoted string literal argument - `{}` does not end with a quote", string);

            // Trim quotes: ["asdf"] -> [asdf].
            let string = &string[1..string.len() - 1];

            let hash_literal = hash(&string).to_literal();

            result = TokenStream::from(TokenTree::Literal(hash_literal));
        }

        TokenTree::Group(group) => {
            result = str_hash_impl(group.stream(), hash);
        }

        TokenTree::Ident(ident) => {
            panic!("`string_hash` macro takes one non-empty quoted string literal argument - ident `{}` was provided", ident);
        }

        TokenTree::Punct(punct) => {
            panic!("`string_hash` macro takes one non-empty quoted string literal argument - punct `{}` was provided", punct);
        }
    }

    assert!(iter.next().is_none(), "`string_hash` macro takes one non-empty quoted string literal argument - multiple were provided");

    result
}

/// Hashes the string to a `u64` using the Rust's default hasher (i.e. one used in the `HashMap`).
#[proc_macro]
pub fn str_hash_default(item: TokenStream) -> TokenStream {
    str_hash_impl(item, string_hash_default)
}

/// Hashes the string to a `u32` using FNV1a hash.
#[proc_macro]
pub fn str_hash_fnv1a(item: TokenStream) -> TokenStream {
    str_hash_impl(item, string_hash_fnv1a)
}
