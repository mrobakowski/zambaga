pub extern crate zambaga as z;

use std::fmt::Write;

#[z::reflect(recursive, dyn)]
pub trait Show {
    fn print(&self, indentation: usize) -> String;
}

/* #region Generated */

trait IsAdtForShow {
    const NAME: &'static str;

    const ADDEND_NAMES: &'static [Option<&'static str>];
    const ADDEND_TYPES: &'static [&'static str];
    const ADDEND_RECURSIVE_IMPLS: &'static [bool];
    fn addend_values(&self) -> Vec<&ShowableValue>;

    const FACTOR_NAMES: &'static [&'static str];
    const FACTOR_TYPES: &'static [&'static str];
    fn factor_values(&self) -> Vec<&ShowableValue>;
}

trait DeriveShow: IsAdtForShow {
    fn print(&self, indentation: usize) -> String;
}

macro_rules! Show {
    ($name:ident; $adt_name:ident; $i:item) => {
        $crate::z::impl_adt!(recursive, dyn; $adt_name; $i);
        impl $crate::Show for $name {
            fn print(&self, indentation: usize) -> String {
                DeriveShow::print(&$adt_name(self), indentation)
            }
        }
    };
}
/* #endregion */

impl<T> DeriveShow for T
where
    T: IsAdtForShow,
{
    fn print(&self, indentation: usize) -> String {
        const STRING_APPEND_ERROR: &str = "Failed to append to string";

        let mut output = String::new();
        writeln!(&mut output, "{} {{", T::NAME).expect(STRING_APPEND_ERROR);
        for (i, (addend, name)) in self
            .addend_values()
            .into_iter()
            .zip(T::ADDEND_NAMES)
            .enumerate()
        {
            writeln!(
                &mut output,
                "{:indentation$}  addend #{i} {name}: {}",
                "",
                addend.print(indentation + 2),
                name = name.unwrap_or("?")
            )
            .expect(STRING_APPEND_ERROR);
        }

        for (i, (factor, name)) in self
            .factor_values()
            .into_iter()
            .zip(T::FACTOR_NAMES)
            .enumerate()
        {
            writeln!(
                &mut output,
                "{:indentation$}  factor #{i} {name}: {}",
                "",
                factor.print(indentation + 2),
                name = name
            )
            .expect(STRING_APPEND_ERROR);
        }
        write!(&mut output, "{:indentation$}}}", "").expect(STRING_APPEND_ERROR);

        output
    }
}

impl Show for String {
    fn print(&self, _indentation: usize) -> String {
        let mut output = String::new();
        write!(&mut output, "{self:?}").expect("Failed to append to string");
        output
    }
}

#[z::derive(Show)]
struct Foo(String);

#[z::derive(Show)]
struct Bar {
    foo: Foo,
}

/* #region Generated */
struct FooAdt<'a>(&'a Foo);

impl IsAdtForShow for FooAdt<'_> {
    const NAME: &'static str = "Foo";

    const ADDEND_NAMES: &'static [Option<&'static str>] = &[None];
    const ADDEND_TYPES: &'static [&'static str] = &["std::string::String"]; // const std::any::type_name is unstable
    const ADDEND_RECURSIVE_IMPLS: &'static [bool] = &[
        const {
            use test_traits::*;
            IsShow::<String>::DOES_IMPLEMENT
        },
        const {
            use test_traits::*;
            IsShow::<i32>::DOES_IMPLEMENT
        },
    ];
    fn addend_values(&self) -> Vec<&ShowableValue> {
        let _0_value = ShowableValue::from(&self.0 .0);
        vec![_0_value]
    }

    const FACTOR_NAMES: &'static [&'static str] = &[];
    const FACTOR_TYPES: &'static [&'static str] = &[];
    fn factor_values(&self) -> Vec<&ShowableValue> {
        vec![]
    }
}

struct BarAdt<'a>(&'a Bar);

impl IsAdtForShow for BarAdt<'_> {
    const NAME: &'static str = "Bar";

    const ADDEND_NAMES: &'static [Option<&'static str>] = &[Some("foo")];
    const ADDEND_TYPES: &'static [&'static str] = &["Foo"];
    const ADDEND_RECURSIVE_IMPLS: &'static [bool] = &[true];
    fn addend_values(&self) -> Vec<&ShowableValue> {
        let _0_value = ShowableValue::from(&self.0.foo);
        vec![_0_value]
    }

    const FACTOR_NAMES: &'static [&'static str] = &[];
    const FACTOR_TYPES: &'static [&'static str] = &[];
    fn factor_values(&self) -> Vec<&ShowableValue> {
        vec![]
    }
}
/* #endregion */

fn main() {
    dbg!(<FooAdt as IsAdtForShow>::ADDEND_RECURSIVE_IMPLS);

    println!(
        "{}",
        Show::print(
            &Bar {
                foo: Foo("Hello, world!".to_string()),
            },
            0
        )
    );

    // test resolution

    {
        use test_traits::*;
        let string = "foo".to_string();
        let x = IsShow(&string).as_dyn();
        let integer = 42;
        let y = IsShow(&integer).as_dyn();
        dbg!(x.is_some());
        dbg!(y.is_none());
    }

}


pub mod test_traits {
    use super::*;

    pub struct IsShow<'a, T>(pub &'a T);

    pub trait DefaultCase<'a> {
        fn as_dyn(&self) -> Option<&'a dyn Show> {
            None
        }
        const DOES_IMPLEMENT: bool = false;
    }

    impl<'a, T> DefaultCase<'a> for IsShow<'a, T> {
    }

    impl<'a, T: Show> IsShow<'a, T> {
        pub fn as_dyn(&self) -> Option<&'a dyn Show> {
            Some(self.0 as &dyn Show)
        }

        pub const DOES_IMPLEMENT: bool = true;
    }

}
