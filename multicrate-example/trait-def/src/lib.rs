use std::{any::Any, fmt::Write};
use zambaga::macros as z;

#[z::reflect]
pub trait Show {
    fn print(&self, indentation: usize) -> String;
}

// `#[z::reflect]` generates a few items for you:
// 1. Trait called `Derive${TraitName}`, in this example `DeriveShow`. You need to provide a 
//  blanket implementation of this trait.
// 2. Trait called `Visit${TraitName}Field`, in this example `VisitShowField`. You need to 
//  provide an implementation of this trait for a concrete visitor type.
// 3. Macro called `${trait_name}_field_visitor`, which you need to invoke passing the visitor 
//  type into it.
impl<T> DeriveShow for T
where
    T: zambaga::WithMirror<MDTShow>,
{
    // The VALIDATION constant is used to check if the type conforms to the requirements of the 
    //  derivation.
    const VALIDATION: zambaga::Validation = zambaga::Validation::ok();
    // The default behavior is to reject the structs with fields that don't recursively implement
    //  the trait. Try commenting out the line above.
    // const VALIDATION: zambaga::Validation = zambaga::Validation::all_fields_impl(&Self::MIRROR);

    fn print(&self, indentation: usize) -> String {
        // The MIRROR structure provides metadata about the type.
        let mirror = T::MIRROR;

        let mut output = String::new();
        writeln!(&mut output, "{} {{", mirror.name.runtime()).unwrap();

        // You can visit the fields generically using the `visit_fields` method.
        //  The types of the fields are provided to the visitor as actual concrete types plugged 
        //  into the generic parameters.
        let DeriveShowVisitor { mut output, .. } = self.visit_fields(DeriveShowVisitor {
            output,
            indentation: indentation + 2,
            index: 1,
        });

        write!(&mut output, "{:indentation$}}}", "").unwrap();

        // You can also iterate over the fields and get `Option<&dyn TraitName>` for each field 
        //  if the trait is object safe. The option will be empty if the field type does not 
        //  implement the trait.
        for (i, (field_name, field_type, recursive_instance)) in self.fields().enumerate() {
            eprintln!(
                "#{i} {field_name:?} {field_type:?} has recursive instance? {:?}",
                recursive_instance.is_some()
            );
        }

        output
    }
}

pub struct DeriveShowVisitor {
    output: String,
    indentation: usize,
    index: usize,
}

show_field_visitor!(DeriveShowVisitor);

impl VisitShowField for DeriveShowVisitor {
    fn visit_implemented<T: Show>(
        &mut self,
        field_value: &T,
        field_name: Option<zambaga::FieldName>,
        _field_type: zambaga::TypeName,
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
        .unwrap();
        self.index += 1;
    }

    fn visit_any(
        &mut self,
        _field_value: &dyn Any,
        field_name: Option<zambaga::FieldName>,
        field_type: zambaga::TypeName,
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
        .unwrap();
        self.index += 1;
    }
}

impl Show for String {
    fn print(&self, _indentation: usize) -> String {
        self.clone()
    }
}
