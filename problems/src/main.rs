use std::ops::Deref;
use std::ops::DerefMut;

/// Who is the owner?
fn ret_string() -> String {
    String::from("  A String object  ")
}

/*
fn main() {
    let s = ret_string();
    let s = s.trim();
    assert_eq!(s, "A String object");
}
 */


/// Select between alternatives
fn choose_str<'a>(s1: &'a str, s2: &'a str, select_s1: bool) -> &'a str {
    if select_s1 {
        s1
    } else {
        s2
    }
}

/// Write a OOR
enum OOR<'a> {
    Owned(String),
    Borrowed(&'a str),
}

impl<'a> Deref for OOR<'a> {
    type Target = str;

    fn deref(&self) -> &str {
        match self {
            OOR::Owned(s) => s,
            OOR::Borrowed(s) => s,
        }
    }
}

impl<'a> DerefMut for OOR<'a> {
    fn deref_mut(&mut self) -> &mut str {
        match self {
            OOR::Owned(s) => s,
            OOR::Borrowed(s) => {
                let s = s.to_string();
                *self = OOR::Owned(s);
                match self {
                    OOR::Owned(s) => s,
                    OOR::Borrowed(_) => unreachable!(),
                }
            }
        }
    }
}


fn main() {
    let s = OOR::Owned(ret_string());
    assert_eq!(s.trim(), "A String object");

    // Check Deref for both variants of OOR
    let s1 = OOR::Owned(String::from("  Hello, world.  "));
    assert_eq!(s1.trim(), "Hello, world.");
    let mut s2 = OOR::Borrowed("  Hello, world!  ");
    assert_eq!(s2.trim(), "Hello, world!");

    // Check choose
    let s = choose_str(&s1, &s2, true);
    assert_eq!(s.trim(), "Hello, world.");
    let s = choose_str(&s1, &s2, false);
    assert_eq!(s.trim(), "Hello, world!");

    // Check DerefMut, a borrowed string should become owned
    assert!(matches!(s1, OOR::Owned(_)));
    assert!(matches!(s2, OOR::Borrowed(_)));
    unsafe {
        for c in s2.as_bytes_mut() {
            if *c == b'!' {
                *c = b'?';
            }
        }
    }
    assert!(matches!(s2, OOR::Owned(_)));
    assert_eq!(s2.trim(), "Hello, world?");
}
