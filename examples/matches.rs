use str_match::str_matches;

fn f(a: &str) -> bool {
    str_matches!(a, "abc{_}ghi" | "aaa{_}" | "{{{_}}}")
}

fn main() {
    assert!(f("abcdefghi"));
    assert!(f("aaabbbccc"));
    assert!(f("{000}"));
    assert!(!f("xyz"));
}
