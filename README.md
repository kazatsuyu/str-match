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

f("abcdefghi"); // "def"
f("aaabbbccc"); // "bbbccc"
f("{000}"); // "000"
f("xyz"); // "!"
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
        "{x};" => x,
        _ => "!",
    }
}
```