use trait_def::{Show, __zambaga_show_impl}; // unfortunately we need to import the extra impl module
use zambaga::macros as z;

#[z::derive(Show)]
struct Foo(String);

#[z::derive(Show)]
struct Bar {
    foo: Foo,
    something_else: u64,
}

fn main() {
    println!(
        "{}",
        Show::print(
            &Bar {
                foo: Foo("Hello, world!".to_string()),
                something_else: 42,
            },
            0
        )
    );
}
