# str-match

Match str with pattern like `format!`.

## Usage

```rust
use str_match::str_match;

fn f(a: &str) -> &str{
    str_match! {
        match a {
            "abc{a}ghi" => a,
            "aaa{bb}" => bb,
            "{{{x}}}" => x,
            _ => "!",
        }
    }
}

assert_eq!(f("abcdefghi"), "def");
assert_eq!(f("aaabbbccc"), "bbbccc");
assert_eq!(f("{000}"), "000");
assert_eq!(f("xyz"), "!");
```

You can use `"attribute"` features in nightly.

```rust
// with `str-match.features = ["attribute"]` in Cargo.toml
#![feature(stmt_expr_attributes, proc_macro_hygiene)]
use str_match::str_match;

fn f(a: &str) -> &str{
    #[str_match]
    match a {
        "abc{x}ghi" => x,
        "aaa{x}" => x,
        "{{{x}}}" => x,
        _ => "!",
    }
}
```

## Limitations

This macro converts `&str` to `&[u8]` and use match slice pattern.
For example, `"abc{x}ghi"` pattern is converted to `[b'a', b'b', b'c', x @ .., b'g', b'h', b'i' ]`.
Because two or more variadic patterns are not allowed in slice pattern, only zero or one placeholder in str pattern is also allowed.

This macro can use single `&str` matching, complex pattern (like `(&str, &str)`) is not supported.
