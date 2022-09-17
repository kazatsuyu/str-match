#![feature(stmt_expr_attributes, proc_macro_hygiene)]
use str_match::str_match;

fn main() {
    let a = "abcdefgh";
    str_match! {
        match a {
            "abc{x}fgh" => println!("{x}"),
            _ => {}
        }
    }
}
