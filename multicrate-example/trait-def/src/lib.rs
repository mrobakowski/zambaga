use std::{any::Any, fmt::Write};
use zambaga::macros as z;

#[z::reflect(recursive, dyn)]
pub trait Show {
    fn print(&self, indentation: usize) -> String;
}

/* #region Generated */
use ::zambaga::*;
pub struct MDTShow;
impl MakeDynTrait for MDTShow {
    type DynTrait<'a> = dyn Show + 'a;
    type IsTrait<'a, T: 'a> = test_traits::IsShow<'a, T>;
    type FieldVisitor = DeriveShowVisitor;
}

pub trait VisitShowField {
    fn visit_implemented<T: Show>(
        &mut self,
        field_value: &T,
        field_name: Option<FieldName>,
        field_type: TypeName,
    );

    fn visit_any(
        &mut self,
        _field_value: &dyn Any,
        _field_name: Option<FieldName>,
        _field_type: TypeName,
    ) {
        panic!(
            "This should not be called. If this is being called it means that you turned off \
            the validator but didn't override the visit_any method."
        );
    }
}

pub trait DeriveShow: WithMirror<MDTShow> {
    const VALIDATION: Validation = Validation::all_fields_impl(&Self::MIRROR);
    type FieldVisitor: VisitShowField;
    fn print(&self, indentation: usize) -> String;

    fn visit_fields(&self, mut visitor: Self::FieldVisitor) -> Self::FieldVisitor {
        Self::MIRROR
            .field_visitor
            .accept::<Self, Self::FieldVisitor>(&mut visitor, self);

        visitor
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
        const VISITOR_ACCEPTOR: AnyVisitorAcceptor = AnyVisitorAcceptor;
    }

    pub struct AnyVisitorAcceptor;

    impl AnyVisitorAcceptor {
        pub fn accept<Visitor: VisitShowField>(
            self,
            field_value: &dyn Any,
            field_name: Option<FieldName>,
            field_type: TypeName,
            visitor: &mut Visitor,
        ) {
            visitor.visit_any(field_value, field_name, field_type);
        }
    }

    impl<'a, T> DefaultCase<'a> for IsShow<'a, T> {}

    impl<'a, T: Show> IsShow<'a, T> {
        pub fn as_dyn(&self) -> Option<&'a dyn Show> {
            Some(self.0 as &dyn Show)
        }

        pub const DOES_IMPLEMENT: bool = true;

        pub const VISITOR_ACCEPTOR: ImplementedVisitorAcceptor = ImplementedVisitorAcceptor;
    }

    pub struct ImplementedVisitorAcceptor;

    impl ImplementedVisitorAcceptor {
        pub fn accept<T: Show, Visitor: VisitShowField>(
            self,
            field_value: &T,
            field_name: Option<FieldName>,
            field_type: TypeName,
            visitor: &mut Visitor,
        ) {
            visitor.visit_implemented(field_value, field_name, field_type);
        }
    }
}

#[macro_export]
macro_rules! ShowMacro {
    ($name:ident; $adt_name:ident; $i:item) => {
        ::zambaga::macros::impl_adt!(recursive, dyn; $adt_name; $i);
        impl $crate::Show for $name {
            fn print(&self, indentation: usize) -> String {
                let _ = <$name as DeriveShow>::VALIDATION;
                DeriveShow::print(self, indentation)
            }
        }
    };
}

pub use ShowMacro as Show;
/* #endregion */

const STRING_APPEND_ERROR: &str = "We should be able to append to a string";

impl<T> DeriveShow for T
where
    T: WithMirror<MDTShow>,
{
    const VALIDATION: Validation = Validation::ok();

    fn print(&self, indentation: usize) -> String {
        let mirror = T::MIRROR;

        let mut output = String::new();
        writeln!(&mut output, "{} {{", mirror.name.runtime()).expect(STRING_APPEND_ERROR);

        let DeriveShowVisitor {
            mut output,
            ..
        } = self.visit_fields(DeriveShowVisitor {
            output,
            indentation: indentation + 2,
            index: 1,
        });

        write!(&mut output, "{:indentation$}}}", "").expect(STRING_APPEND_ERROR);

        for (i, (field_name, field_type, recursive_instance)) in self.fields().enumerate() {
            println!(
                "#{i} {field_name:?} {field_type:?} has recursive instance {:?}",
                recursive_instance.is_some()
            );
        }

        output
    }

    type FieldVisitor = DeriveShowVisitor;
}

pub struct DeriveShowVisitor {
    output: String,
    indentation: usize,
    index: usize,
}

impl VisitShowField for DeriveShowVisitor {
    fn visit_implemented<T: Show>(
        &mut self,
        field_value: &T,
        field_name: Option<FieldName>,
        _field_type: TypeName,
    ) {
        writeln!(
            self.output,
            "{:indentation$}field #{index} {name}: {}",
            "",
            field_value.print(self.indentation),
            indentation = self.indentation,
            name = field_name.map(|x| x.0).unwrap_or("?"),
            index = self.index
        )
        .expect(STRING_APPEND_ERROR);
        self.index += 1;
    }

    fn visit_any(
        &mut self,
        _field_value: &dyn Any,
        field_name: Option<FieldName>,
        field_type: TypeName,
    ) {
        writeln!(
            self.output,
            "{:indentation$}field #{index} {name}: <value of type `{typ}` which does not implement Show>",
            "",
            indentation = self.indentation,
            name = field_name.map(|x| x.0).unwrap_or("?"),
            typ = field_type.runtime(),
            index = self.index
        )
        .expect(STRING_APPEND_ERROR);
        self.index += 1;
    }
}

impl Show for String {
    fn print(&self, _indentation: usize) -> String {
        self.clone()
    }
}
