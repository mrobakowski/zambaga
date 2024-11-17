use zambaga::macros as z;
use zambaga::*;

use std::any::Any;

use test_traits::{DefaultCase, IsShow};
use trait_def::*;

#[z::derive(Show)]
struct Foo(String);

#[z::derive(Show)]
struct Bar {
    foo: Foo,
    something_else: u64,
}

/* #region Generated */
impl WithMirror<MDTShow> for Foo {
    const MIRROR: Mirror<MDTShow> = Mirror {
        name: TypeName::from_source::<Foo>("Foo"),
        field_visitor: {
            fn __zambaga_visitor(
                foo: &Foo,
                visitor: &mut <MDTShow as MakeDynTrait>::FieldVisitor,
            ) {
                <MDTShow as MakeDynTrait>::IsTrait::<'_, String>::VISITOR_ACCEPTOR.accept(
                    &foo.0,
                    None,
                    TypeName::from_source::<String>("String"),
                    visitor
                );
            }
            TypeIdBasedVisitor::new(__zambaga_visitor)
        },
        fields_or_variants: FieldsOrVariants::TupleStruct {
            fields: &[(
                TypeName::from_source::<String>("String"),
                FieldExtractor::new({
                    fn __zambaga_extractor(value: &dyn Any) -> Result<&dyn Any, TypeError> {
                        let value = value.downcast_ref::<Foo>().ok_or(TypeError)?;
                        Ok(&value.0)
                    }
                    __zambaga_extractor
                }),
                ImplExtractor {
                    extractor_fn: {
                        type DynTrait<'a> = dyn Show + 'a;
    
                        fn __zambaga_extractor(
                            value: &dyn Any,
                        ) -> Result<Option<&DynTrait>, TypeError> {
                            let value = value.downcast_ref::<String>().ok_or(TypeError)?;
                            Ok(IsShow(value).as_dyn())
                        }
                        __zambaga_extractor
                    },
                    has_impl: {
                        use test_traits::*;
                        IsShow::<String>::DOES_IMPLEMENT
                    },
                },
            )],
        },
    };
}

impl WithMirror<MDTShow> for Bar {
    const MIRROR: Mirror<MDTShow> = Mirror {
        name: TypeName::from_source::<Bar>("Bar"),
        field_visitor: {
            fn __zambaga_visitor(
                bar: &Bar,
                visitor: &mut <MDTShow as MakeDynTrait>::FieldVisitor,
            ) {
                IsShow::<Foo>::VISITOR_ACCEPTOR.accept(
                    &bar.foo,
                    Some(FieldName("foo")),
                    TypeName::from_source::<Foo>("Foo"),
                    visitor,
                );
    
                IsShow::<u64>::VISITOR_ACCEPTOR.accept(
                    &bar.something_else,
                    Some(FieldName("something_else")),
                    TypeName::from_source::<u64>("u64"),
                    visitor,
                );
            }
            TypeIdBasedVisitor::new(__zambaga_visitor)
        },
        fields_or_variants: FieldsOrVariants::Struct {
            fields: &[(
                FieldName("foo"),
                TypeName::from_source::<Foo>("Foo"),
                FieldExtractor::new({
                    fn __zambaga_extractor(this_value: &dyn Any) -> Result<&dyn Any, TypeError> {
                        let value = this_value.downcast_ref::<Bar>().ok_or(TypeError)?;
                        Ok(&value.foo)
                    }
                    __zambaga_extractor
                }),
                ImplExtractor {
                    extractor_fn: {
                        fn __zambaga_extractor(
                            field_value: &dyn Any,
                        ) -> Result<Option<&dyn Show>, TypeError> {
                            let value = field_value.downcast_ref::<Foo>().ok_or(TypeError)?;
                            Ok(IsShow(value).as_dyn())
                        }
    
                        let x: for<'a> fn(&'a dyn Any) -> Result<Option<&'a dyn Show>, TypeError> =
                            __zambaga_extractor;
                        x
                    },
                    has_impl: {
                        use test_traits::*;
                        IsShow::<Foo>::DOES_IMPLEMENT
                    },
                },
            ),
            (
                FieldName("something_else"),
                TypeName::from_source::<u64>("u64"),
                FieldExtractor::new({
                    fn __zambaga_extractor(this_value: &dyn Any) -> Result<&dyn Any, TypeError> {
                        let value = this_value.downcast_ref::<Bar>().ok_or(TypeError)?;
                        Ok(&value.something_else)
                    }
                    __zambaga_extractor
                }),
                ImplExtractor {
                    extractor_fn: {
                        fn __zambaga_extractor(
                            field_value: &dyn Any,
                        ) -> Result<Option<&dyn Show>, TypeError> {
                            let value = field_value.downcast_ref::<u64>().ok_or(TypeError)?;
                            Ok(IsShow(value).as_dyn())
                        }
    
                        let x: for<'a> fn(&'a dyn Any) -> Result<Option<&'a dyn Show>, TypeError> =
                            __zambaga_extractor;
                        x
                    },
                    has_impl: {
                        use test_traits::*;
                        IsShow::<u64>::DOES_IMPLEMENT
                    },
                },
            )],
        },
    };
}

/* #endregion */

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
