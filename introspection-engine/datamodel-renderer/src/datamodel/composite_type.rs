mod field;

use crate::value::{Constant, Documentation};
pub use field::CompositeTypeField;
use psl::dml;
use std::{borrow::Cow, fmt};

/// A type block in a PSL file.
#[derive(Debug)]
pub struct CompositeType<'a> {
    name: Constant<Cow<'a, str>>,
    documentation: Option<Documentation<'a>>,
    fields: Vec<CompositeTypeField<'a>>,
}

impl<'a> CompositeType<'a> {
    /// Create a new type declaration block. Will not be valid without
    /// adding at least one field.
    ///
    /// ```ignore
    /// type Address {
    /// //   ^^^^^^^ name
    /// }
    /// ```
    pub fn new(name: impl Into<Cow<'a, str>>) -> Self {
        let name = Constant::new_no_validate(name.into());

        Self {
            name,
            documentation: None,
            fields: Vec::new(),
        }
    }

    /// Documentation of the type.
    ///
    /// ```ignore
    /// /// This is the documentation.
    /// type Foo {
    ///   ....
    /// }
    /// ```
    pub fn documentation(&mut self, documentation: impl Into<Cow<'a, str>>) {
        self.documentation = Some(Documentation(documentation.into()));
    }

    /// Add a new field to the type.
    ///
    /// ```ignore
    /// type Foo {
    ///     bar String
    /// //  ^^^^^^^^^^ this
    /// }
    /// ```
    pub fn push_field(&mut self, field: CompositeTypeField<'a>) {
        self.fields.push(field);
    }

    /// Generate a composite type rendering from the deprecated DML structure.
    ///
    /// Remove when destroying the DML.
    pub fn from_dml(datasource: &'a psl::Datasource, dml_ct: &dml::CompositeType) -> Self {
        let mut composite_type = CompositeType::new(dml_ct.name.clone());

        for dml_field in dml_ct.fields.iter() {
            composite_type.push_field(CompositeTypeField::from_dml(datasource, dml_field));
        }

        composite_type
    }
}

impl<'a> fmt::Display for CompositeType<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref docs) = self.documentation {
            docs.fmt(f)?;
        }

        writeln!(f, "type {} {{", self.name)?;

        for field in self.fields.iter() {
            writeln!(f, "{field}")?;
        }

        f.write_str("}\n")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::datamodel::*;

    #[test]
    fn kitchen_sink() {
        let mut composite_type = CompositeType::new("Address");
        composite_type.documentation("...so many tears 🎵");

        let mut field = CompositeTypeField::new("Street", "String");
        field.native_type("db", "VarChar", vec!["255".into()]);
        field.default(DefaultValue::text("Prenzlauer Allee 193"));
        field.map("Shield");
        composite_type.push_field(field);

        let field = CompositeTypeField::new("Number", "Int");
        composite_type.push_field(field);

        let mut field = CompositeTypeField::new("City", "String");
        field.optional();
        field.documentation("...soooooooo many tears 🎵");
        composite_type.push_field(field);

        let mut field = CompositeTypeField::new("Other", "String");
        field.array();
        composite_type.push_field(field);

        let mut field = CompositeTypeField::new("Invalid", "Float");
        field.map("1Invalid");
        composite_type.push_field(field);

        let mut field = CompositeTypeField::new("11111", "Float");
        field.commented_out();
        field.map("11111");
        composite_type.push_field(field);

        let expected = expect![[r#"
            /// ...so many tears 🎵
            type Address {
              Street  String   @default("Prenzlauer Allee 193") @map("Shield") @db.VarChar(255)
              Number  Int
              /// ...soooooooo many tears 🎵
              City    String?
              Other   String[]
              Invalid Float    @map("1Invalid")
              // 11111 Float @map("11111")
            }
        "#]];

        let rendered = psl::reformat(&format!("{composite_type}"), 2).unwrap();
        expected.assert_eq(&rendered);
    }
}
