#![feature(stmt_expr_attributes, proc_macro_hygiene)]
use str_match::str_match;

fn f(a: &str) -> &str {
    #[str_match]
    match a {
        "abc{x}ghi" => x,
        "aaa{x}" => x,
        "{{{x}}}" => x,
        _ => "!",
    }
}

fn main() {
    assert_eq!(f("abcdefghi"), "def");
    assert_eq!(f("aaabbbccc"), "bbbccc");
    assert_eq!(f("{000}"), "000");
    assert_eq!(f("xyz"), "!");
}
