use crate::{
    datamodel::{
        attributes::FieldAttribute, model::index_field_input::IndexFieldOptions, DefaultValue, FieldType,
        IdFieldDefinition, Relation,
    },
    value::{Constant, Documentation, Function, Text},
};
use psl::dml;
use std::{borrow::Cow, collections::HashMap, fmt};

/// A field in a model block.
#[derive(Debug)]
pub struct ModelField<'a> {
    name: Constant<Cow<'a, str>>,
    commented_out: bool,
    r#type: FieldType<'a>,
    documentation: Option<Documentation<'a>>,
    updated_at: Option<FieldAttribute<'a>>,
    unique: Option<FieldAttribute<'a>>,
    id: Option<IdFieldDefinition<'a>>,
    default: Option<DefaultValue<'a>>,
    map: Option<FieldAttribute<'a>>,
    relation: Option<Relation<'a>>,
    native_type: Option<FieldAttribute<'a>>,
    ignore: Option<FieldAttribute<'a>>,
}

impl<'a> ModelField<'a> {
    /// Create a new required model field declaration.
    ///
    /// ```ignore
    /// model User {
    ///   name String
    /// //     ^^^^^^ type_name
    /// //^^^^ name
    /// }
    /// ```
    pub fn new(name: impl Into<Cow<'a, str>>, type_name: impl Into<Cow<'a, str>>) -> Self {
        let name = Constant::new_no_validate(name.into());

        Self {
            name,
            commented_out: false,
            r#type: FieldType::required(type_name),
            map: None,
            documentation: None,
            updated_at: None,
            unique: None,
            id: None,
            default: None,
            relation: None,
            native_type: None,
            ignore: None,
        }
    }

    /// Sets the field as optional.
    ///
    /// ```ignore
    /// model Address {
    ///   street String?
    /// //             ^ this
    /// }
    /// ```
    pub fn optional(&mut self) {
        self.r#type.into_optional();
    }

    /// Sets the field to be an array.
    ///
    /// ```ignore
    /// model Address {
    ///   street String[]
    /// //             ^^ this
    /// }
    /// ```
    pub fn array(&mut self) {
        self.r#type.into_array();
    }

    /// Sets the field to be unsupported.
    ///
    /// ```ignore
    /// model Address {
    ///   street Unsupported("foo")
    /// //       ^^^^^^^^^^^^^^^^^^ this
    /// }
    /// ```
    pub fn unsupported(&mut self) {
        self.r#type.into_unsupported();
    }

    /// Sets the field map attribute.
    ///
    /// ```ignore
    /// model Address {
    ///   street String @map("Straße")
    ///                       ^^^^^^ value
    /// }
    /// ```
    pub fn map(&mut self, value: impl Into<Cow<'a, str>>) {
        let mut map = Function::new("map");
        map.push_param(value.into());

        self.map = Some(FieldAttribute::new(map));
    }

    /// Documentation of the field.
    ///
    /// ```ignore
    /// model Foo {
    ///   /// This is the documentation.
    ///   bar Int
    /// }
    /// ```
    pub fn documentation(&mut self, documentation: impl Into<Cow<'a, str>>) {
        match self.documentation.as_mut() {
            Some(docs) => docs.push(documentation),
            None => self.documentation = Some(Documentation(documentation.into())),
        }
    }

    /// Sets the field default attribute.
    ///
    /// ```ignore
    /// model Address {
    ///   street String @default("Prenzlauer Allee")
    ///                           ^^^^^^^^^^^^^^^^ value
    /// }
    /// ```
    pub fn default(&mut self, value: DefaultValue<'a>) {
        self.default = Some(value);
    }

    /// Sets the native type of the field.
    ///
    /// ```ignore
    /// model Address {
    ///   street String @db.VarChar(255)
    /// //                          ^^^ param
    /// //                  ^^^^^^^ type_name
    /// //               ^^ prefix
    /// }
    /// ```
    ///
    /// TODO: `params` as `&[&str]` when we get rid of the DML.
    pub fn native_type(
        &mut self,
        prefix: impl Into<Cow<'a, str>>,
        r#type: impl Into<Cow<'a, str>>,
        params: Vec<String>,
    ) {
        let mut native_type = FieldAttribute::new(Function::new(r#type));

        for param in params {
            native_type.push_param(Constant::new_no_validate(param));
        }

        native_type.prefix(prefix);

        self.native_type = Some(native_type);
    }

    /// Marks the field to hold the update timestamp.
    ///
    /// ```ignore
    /// model Address {
    ///   street String @updatedAt
    /// //              ^^^^^^^^^^ adds this
    /// }
    /// ```
    pub fn updated_at(&mut self) {
        self.updated_at = Some(FieldAttribute::new(Function::new("updatedAt")));
    }

    /// Marks the field to hold a unique constraint.
    ///
    /// ```ignore
    /// model Address {
    ///   street String @unique(sort: Asc, length: 11)
    /// //              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ this
    /// }
    /// ```
    pub fn unique(&mut self, options: IndexFieldOptions<'a>) {
        let mut fun = Function::new("unique");

        if let Some(map) = options.map {
            fun.push_param(("map", Text::new(map)));
        }

        if let Some(sort_order) = options.sort_order {
            fun.push_param(("sort", Constant::new_no_validate(sort_order)));
        }

        if let Some(length) = options.length {
            fun.push_param(("length", Constant::new_no_validate(length)));
        }

        if let Some(clustered) = options.clustered {
            fun.push_param(("clustered", Constant::new_no_validate(clustered)));
        }

        self.unique = Some(FieldAttribute::new(fun));
    }

    /// Marks the field to be the id of the model.
    ///
    /// ```ignore
    /// model Address {
    ///   street String @id
    /// //              ^^^ this
    /// }
    /// ```
    pub fn id(&mut self, definition: IdFieldDefinition<'a>) {
        self.id = Some(definition);
    }

    /// Set the field to be a relation.
    ///
    /// ```ignore
    /// model Address {
    ///   street Street @relation(...)
    /// //              ^^^^^^^^^^^^^^ this
    /// }
    /// ```
    pub fn relation(&mut self, relation: Relation<'a>) {
        self.relation = Some(relation);
    }

    /// Ignores the field.
    ///
    /// ```ignore
    /// model Address {
    ///   street Street @ignore
    /// //              ^^^^^^^ this
    /// }
    /// ```
    pub fn ignore(&mut self) {
        self.ignore = Some(FieldAttribute::new(Function::new("ignore")));
    }

    /// Comments the field out.
    pub fn commented_out(&mut self) {
        self.commented_out = true;
    }

    /// Generate a model field rendering from the deprecated DML structure.
    ///
    /// Remove when destroying the DML. This API cannot really be
    /// public, because we need info from the model and it doesn't
    /// make much sense to call this from outside of the module.
    pub(super) fn from_dml(
        datasource: &'a psl::Datasource,
        _dml_model: &dml::Model,
        dml_field: &dml::Field,
        uniques: &HashMap<&str, IndexFieldOptions<'static>>,
        id: Option<IdFieldDefinition<'static>>,
    ) -> ModelField<'a> {
        match dml_field {
            dml::Field::ScalarField(ref sf) => {
                let (r#type, native_type): (String, _) = match sf.field_type {
                    dml::FieldType::Enum(ref ct) => (ct.clone(), None),
                    dml::FieldType::Relation(ref info) => (info.referenced_model.clone(), None),
                    dml::FieldType::Unsupported(ref s) => (s.clone(), None),
                    dml::FieldType::Scalar(ref st, ref nt) => {
                        (st.as_ref().to_owned(), nt.as_ref().map(|nt| (nt.name(), nt.args())))
                    }
                    dml::FieldType::CompositeType(ref ct) => (ct.clone(), None),
                };

                let mut field = Self::new(sf.name.clone(), r#type);

                match sf.arity {
                    dml::FieldArity::Optional => {
                        field.optional();
                    }
                    dml::FieldArity::List => {
                        field.array();
                    }
                    _ => (),
                };

                if sf.field_type.is_unsupported() {
                    field.unsupported();
                }

                if let Some(ref docs) = sf.documentation {
                    field.documentation(docs.clone());
                }

                if let Some(dv) = sf.default_value() {
                    field.default(DefaultValue::from_dml(dv));
                }

                if let Some((name, args)) = native_type {
                    field.native_type(&datasource.name, name, args);
                }

                if sf.is_updated_at {
                    field.updated_at();
                }

                if let Some(unique) = uniques.get(sf.name.as_str()) {
                    field.unique(unique.clone());
                }

                if sf.is_ignored {
                    field.ignore();
                }

                if sf.is_commented_out {
                    field.commented_out();
                }

                if let Some(ref map) = sf.database_name {
                    field.map(map.clone());
                }

                if let Some(id) = id {
                    field.id(id);
                }

                field
            }
            dml::Field::RelationField(rf) => {
                let field_name = rf.name.clone();
                let referenced_model = rf.relation_info.referenced_model.clone();

                let mut field = Self::new(field_name, referenced_model);

                match rf.arity {
                    dml::FieldArity::Optional => field.optional(),
                    dml::FieldArity::List => field.array(),
                    dml::FieldArity::Required => (),
                }

                if let Some(ref docs) = rf.documentation {
                    field.documentation(docs.clone());
                }

                if rf.is_ignored {
                    field.ignore();
                }

                let dml_info = &rf.relation_info;
                let relation_name = dml_info.name.as_str();

                // :(
                if !relation_name.is_empty() || (!dml_info.fields.is_empty() || !dml_info.references.is_empty()) {
                    let mut relation = Relation::new();

                    if !relation_name.is_empty() {
                        relation.name(relation_name.to_owned());
                    }

                    relation.fields(dml_info.fields.iter().map(Clone::clone).map(Cow::Owned));
                    relation.references(dml_info.references.iter().map(Clone::clone).map(Cow::Owned));

                    if let Some(ref action) = dml_info.on_delete {
                        relation.on_delete(action.as_ref().to_owned());
                    }

                    if let Some(ref action) = dml_info.on_update {
                        relation.on_update(action.as_ref().to_owned());
                    }

                    if let Some(ref map) = &dml_info.fk_name {
                        relation.map(map.clone());
                    }

                    field.relation(relation);
                }

                field
            }
            dml::Field::CompositeField(cf) => {
                let name = cf.name.clone();
                let ct = cf.composite_type.clone();

                let mut field = Self::new(name, ct);

                match cf.arity {
                    dml::FieldArity::Required => (),
                    dml::FieldArity::Optional => field.optional(),
                    dml::FieldArity::List => field.array(),
                }

                if let Some(ref docs) = cf.documentation {
                    field.documentation(docs.clone());
                }

                if let Some(ref map) = cf.database_name {
                    field.map(map.clone());
                }

                if cf.is_commented_out {
                    field.commented_out();
                }

                if cf.is_ignored {
                    field.ignore();
                }

                if let Some(ref dv) = cf.default_value {
                    field.default(DefaultValue::from_dml(dv));
                }

                field
            }
        }
    }
}

impl<'a> fmt::Display for ModelField<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref docs) = self.documentation {
            docs.fmt(f)?;
        }

        if self.commented_out {
            f.write_str("// ")?;
        }

        write!(f, "{} {}", self.name, self.r#type)?;

        if let Some(ref updated_at) = self.updated_at {
            write!(f, " {updated_at}")?;
        }

        if let Some(ref unique) = self.unique {
            write!(f, " {unique}")?;
        }

        if let Some(ref id) = self.id {
            write!(f, " {id}")?;
        }

        if let Some(ref def) = self.default {
            write!(f, " {def}")?;
        }

        if let Some(ref map) = self.map {
            write!(f, " {map}")?;
        }

        if let Some(ref relation) = self.relation {
            write!(f, " {relation}")?;
        }

        if let Some(ref nt) = self.native_type {
            write!(f, " {nt}")?;
        }

        if let Some(ref ignore) = self.ignore {
            write!(f, " {ignore}")?;
        }

        Ok(())
    }
}
