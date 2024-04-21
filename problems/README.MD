# Problems

Raynner Schnneider Carvalho

## Who is the owner?

The following code fails to compile:

```rust
fn ret_string() -> String {
    String::from("  A String object  ")
}

fn main() {
    let s = ret_string().trim();
    assert_eq!(s, "A String object");
}
```

- Why? Ask yourself: what is the lifetime of s? Who is the owner of the underlying string with spaces (every object has an owner)?

### Solution

The `trim` method returns a string slice, which is a reference to the original string. The original string is owned by the `ret_string` function, and it is dropped at the end of the function. The reference returned by `trim` is invalid after the function returns, and the program will panic when it tries to access it.

- Fix the code so that it compiles (and the s variable represents the trimmed string). Note that you can reuse the same variable name.

### Solution

```rust
fn main() {
    let s = ret_string();
    let s = s.trim();
    assert_eq!(s, "A String object");
}
```

## Select between alternatives

Add the most appropriate lifetimes to the following function:

```rust
fn choose_str(s1: &str, s2: &str, select_s1: bool) -> &str {
    if select_s1 { s1 } else { s2 }
}
```

At call time, s1 and s2 may have different lifetimes and we don't want any constraint between the lifetimes of those two strings. 

### Solution

The returned reference should have the same lifetime as the selected string. The function signature should be:

```rust
fn choose_str<'a>(s1: &'a str, s2: &'a str, select_s1: bool) -> &'a str {
    if select_s1 { s1 } else { s2 }
}
```

## Write a OOR (owned or ref) type

We want to create a `OOR` type which can store either a `String` or a `&str` to avoid copying a string which already exists in the environment.

- Write a `OOR` enum with two alternatives: `Owned` which stored a `String` and `Borrowed` which stores a `&str`.

It will require using a generic parameter. What does it represent?

### Solution

The generic parameter represents the lifetime of the borrowed string. The `OOR` enum is defined as:

```rust
enum OOR<'a> {
    Owned(String),
    Borrowed(&'a str),
}
```

- Implement the Deref trait for the OOR structure so that it dereferences into an a &str. What is the lifetime of the resulting &str (note that you have no choice there)? Why is that always appropriate?

### Solution

The `Deref` trait is implemented as:

```rust
use std::ops::Deref;

impl<'a> Deref for OOR<'a> {
    type Target = str;

    fn deref(&self) -> &str {
        match self {
            OOR::Owned(s) => s,
            OOR::Borrowed(s) => s,
        }
    }
}
```

The lifetime of the resulting `&str` is the same as the lifetime of the `OOR` object. This is always appropriate because the `OOR` object is the owner of the string, and the lifetime of the string should be linked to the lifetime of the owner.

- Write a DerefMut trait for the OOR structure. If you have not stored a String, you will have to mutate and store a String before you can hand out a &mut str because you can't transform your inner &str into &mut str.

### Solution

The `DerefMut` trait is implemented as:

```rust
use std::ops::DerefMut;

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
```