use std::{
    any::{Any, TypeId},
    fmt::Debug,
};

pub mod macros {
    pub use zambaga_macros::*;
}

#[derive(Clone, Copy, Debug)]
pub struct FieldName(pub &'static str);
#[derive(Clone, Copy)]
pub struct TypeName {
    in_source: &'static str,
    from_type_id: fn() -> &'static str,
}

impl Debug for TypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TypeName {{ in_source: {:?} }}", self.in_source)
    }
}

impl TypeName {
    pub const fn from_source<T>(source_name: &'static str) -> Self {
        TypeName {
            in_source: source_name,
            from_type_id: || std::any::type_name::<T>(),
        }
    }

    pub fn runtime(&self) -> &'static str {
        (self.from_type_id)()
    }

    pub const fn compiletime(&self) -> &'static str {
        self.in_source
    }
}

#[derive(Debug)]
pub struct TypeError;

pub trait MakeDynTrait {
    type DynTrait<'a>: ?Sized;
    type IsTrait<'a, T: 'a>: ?Sized;
    type FieldVisitor;
    const TRAIT_NAME: &'static str;
}

type ExtractorFn<MDT> =
    fn(&dyn Any) -> Result<Option<&<MDT as MakeDynTrait>::DynTrait<'_>>, TypeError>;

pub struct ImplExtractor<MDT: MakeDynTrait> {
    pub extractor_fn: ExtractorFn<MDT>,
    pub has_impl: bool,
}

impl<MDT: MakeDynTrait> ImplExtractor<MDT> {
    pub const fn new(extractor_fn: ExtractorFn<MDT>, has_impl: bool) -> Self {
        Self {
            extractor_fn,
            has_impl,
        }
    }

    pub fn extract<'a, T: Any>(
        &'_ self,
        value: &'a T,
    ) -> Result<Option<&'a MDT::DynTrait<'a>>, TypeError> {
        (self.extractor_fn)(value)
    }
}

pub struct FieldExtractor {
    pub extractor_fn: fn(&dyn Any) -> Result<&dyn Any, TypeError>,
}

impl FieldExtractor {
    pub const fn new(extractor_fn: fn(&dyn Any) -> Result<&dyn Any, TypeError>) -> Self {
        Self { extractor_fn }
    }
}

pub struct Validation;

impl Validation {
    pub const fn ok() -> Self {
        Validation
    }
    pub const fn all_fields_impl<MDT: MakeDynTrait>(mirror: &Mirror<MDT>) -> Validation {
        match mirror.fields_or_variants {
            FieldsOrVariants::Struct { fields } => {
                let mut i = 0;
                while i < fields.len() {
                    let (field_name, field_type, _, show_impl_extractor) = &fields[i];
                    i += 1;
                    if !show_impl_extractor.has_impl {
                        return Validation::doesnt_implement(
                            field_name,
                            field_type,
                            MDT::TRAIT_NAME,
                        );
                    }
                }
            }
            FieldsOrVariants::TupleStruct { fields } => {
                let mut i = 0;
                while i < fields.len() {
                    let (_, _, show_impl_extractor) = &fields[i];
                    i += 1;
                    if !show_impl_extractor.has_impl {
                        return Validation::err();
                    }
                }
            }
            FieldsOrVariants::Enum { variants } => {
                let mut i = 0;
                while i < variants.len() {
                    let (_, _, _, show_impl_extractor) = &variants[i];
                    i += 1;
                    if !show_impl_extractor.has_impl {
                        return Validation::err();
                    }
                }
            }
        }
        Validation
    }
    pub const fn err() -> Self {
        panic!("Validation failed")
    }

    pub const fn doesnt_implement(
        field_name: &FieldName,
        field_type: &TypeName,
        trait_name: &str,
    ) -> Validation {
        const_panic::concat_panic!(const_panic::FmtArg::DISPLAY;
            "\nField `", field_name.0, "` of type `", field_type.compiletime(), "` does not implement `", trait_name, "`\n"
        )
    }
}

pub struct FieldTraverser<MDT: MakeDynTrait> {
    f: fn(&(), &mut <MDT as MakeDynTrait>::FieldVisitor),
    expected_type_id: fn() -> TypeId,
}

impl<MDT> FieldTraverser<MDT>
where
    MDT: MakeDynTrait,
{
    pub const fn new<T: ?Sized + 'static>(
        f: fn(&T, &mut <MDT as MakeDynTrait>::FieldVisitor),
    ) -> Self {
        Self {
            f: unsafe {
                std::mem::transmute::<
                    fn(&T, &mut <MDT as MakeDynTrait>::FieldVisitor),
                    fn(&(), &mut <MDT as MakeDynTrait>::FieldVisitor),
                >(f)
            },
            expected_type_id: || TypeId::of::<T>(),
        }
    }

    pub fn accept<T: ?Sized + 'static>(
        &self,
        field_visitor: &mut <MDT as MakeDynTrait>::FieldVisitor,
        this: &T,
    ) {
        if (self.expected_type_id)() != TypeId::of::<T>() {
            panic!("TypeId mismatch");
        }
        (unsafe {
            std::mem::transmute::<
                fn(&(), &mut <MDT as MakeDynTrait>::FieldVisitor),
                fn(&T, &mut <MDT as MakeDynTrait>::FieldVisitor),
            >(self.f)
        })(this, field_visitor);
    }
}

pub struct Mirror<MDT: MakeDynTrait + 'static> {
    pub name: TypeName,
    pub field_traverser: FieldTraverser<MDT>,
    pub fields_or_variants: FieldsOrVariants<MDT>,
}

pub enum FieldsOrVariants<MDT: MakeDynTrait + 'static> {
    Struct {
        fields: &'static [(FieldName, TypeName, FieldExtractor, ImplExtractor<MDT>)],
    },
    TupleStruct {
        fields: &'static [(TypeName, FieldExtractor, ImplExtractor<MDT>)],
    },
    Enum {
        variants: &'static [(FieldName, TypeName, FieldExtractor, ImplExtractor<MDT>)],
    },
}

pub trait WithMirror<MDT: MakeDynTrait + 'static>: Sized + 'static {
    const MIRROR: Mirror<MDT>;

    fn fields(
        &self,
    ) -> impl Iterator<Item = (Option<FieldName>, TypeName, Option<&MDT::DynTrait<'_>>)> {
        enum OneOfThree<T, U, V> {
            One(T),
            Two(U),
            Three(V),
        }
        impl<T, U, V, I> Iterator for OneOfThree<T, U, V>
        where
            T: Iterator<Item = I>,
            U: Iterator<Item = I>,
            V: Iterator<Item = I>,
        {
            type Item = T::Item;
            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    Self::One(x) => x.next(),
                    Self::Two(x) => x.next(),
                    Self::Three(x) => x.next(),
                }
            }
        }

        match Self::MIRROR.fields_or_variants {
            FieldsOrVariants::Struct { fields } => OneOfThree::One(fields.iter().map(
                |(name, type_name, extractor, show_impl_extractor)| {
                    let value = (extractor.extractor_fn)(self).unwrap();
                    let value = (show_impl_extractor.extractor_fn)(value).unwrap();
                    (Some(*name), *type_name, value)
                },
            )),
            FieldsOrVariants::TupleStruct { fields } => OneOfThree::Two(fields.iter().map(
                |(type_name, extractor, show_impl_extractor)| {
                    let value = (extractor.extractor_fn)(self).unwrap();
                    let value = (show_impl_extractor.extractor_fn)(value).unwrap();
                    (None, *type_name, value)
                },
            )),
            FieldsOrVariants::Enum { variants } => OneOfThree::Three(variants.iter().map(
                |(_name, _type_name, _extractor, _show_impl_extractor)| {
                    unimplemented!("this is wrong");
                },
            )),
        }
    }
}
