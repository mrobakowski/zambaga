use zambaga::macros as z;

#[z::reflect]
pub trait Zerde {
    // TODO: support `output: impl std::io::Write`
    fn to_json(&self, output: &mut dyn std::io::Write) -> std::io::Result<()>;
}

pub struct NoOpVisitor;

zerde_field_visitor!(NoOpVisitor);

#[z::derivation(Zerde)]
pub mod ZerdeDerivation {
    use super::*;

    pub fn to_json<S>(zelf: &S, output: &mut dyn std::io::Write) -> std::io::Result<()> {
        output.write_all(b"{}")
    }

    pub fn visit_implemented<T: Zerde>(
        output: &mut dyn std::io::Write,
        field_value: &T,
        field_name: Option<zambaga::FieldName>,
        _field_type: zambaga::TypeName,
    ) {
        field_value.to_json(output).unwrap();
    }
}
