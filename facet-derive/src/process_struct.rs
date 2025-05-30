use super::*;

/// Processes a regular struct to implement Facet
///
/// Example input:
/// ```rust
/// struct Blah {
///     foo: u32,
///     bar: String,
/// }
/// ```
pub(crate) fn process_struct(parsed: Struct) -> proc_macro::TokenStream {
    let struct_name = parsed.name.to_string();

    // Generate field definitions
    let (generics_def, generics_use) = generics_split_for_impl(parsed.generics.as_ref());
    let kind;
    let where_clauses;

    let fields = match &parsed.kind {
        StructKind::Struct { clauses, fields } => {
            kind = "::facet::StructKind::Struct";
            where_clauses = clauses.as_ref();
            fields
                .content
                .0
                .iter()
                .map(|field| {
                    let field_name = field.value.name.to_string();
                    gen_struct_field(
                        &field_name,
                        &struct_name,
                        &generics_use,
                        &field.value.attributes,
                    )
                })
                .collect::<Vec<String>>()
        }
        StructKind::TupleStruct {
            fields,
            clauses,
            semi: _,
        } => {
            kind = "::facet::StructKind::TupleStruct";
            where_clauses = clauses.as_ref();
            fields
                .content
                .0
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    let field_name = format!("{index}");
                    gen_struct_field(
                        &field_name,
                        &struct_name,
                        &generics_use,
                        &field.value.attributes,
                    )
                })
                .collect::<Vec<String>>()
        }
        StructKind::UnitStruct { clauses, semi: _ } => {
            kind = "::facet::StructKind::Unit";
            where_clauses = clauses.as_ref();
            vec![]
        }
    }
    .join(", ");

    let static_decl = if parsed.generics.is_none() {
        generate_static_decl(&struct_name)
    } else {
        String::new()
    };
    let maybe_container_doc = build_maybe_doc(&parsed.attributes);
    let where_clauses = where_clauses.map_or(String::new(), ToString::to_string);

    let mut invariant_maybe = "".to_string();
    let invariant_attrs = parsed
        .attributes
        .iter()
        .filter_map(|attr| match &attr.body.content {
            AttributeInner::Facet(facet_attr) => match &facet_attr.inner.content {
                FacetInner::Invariants(invariant_inner) => Some(invariant_inner),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>();

    if !invariant_attrs.is_empty() {
        let tests = invariant_attrs
            .iter()
            .map(|invariant| {
                let invariant_name = invariant.value.as_str();
                format!(
                    r#"
                    if !value.{invariant_name}() {{
                        return false;
                    }}
                    "#
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let invariant_fn = format!(
            r#"
            unsafe fn invariants<'mem>(value: ::facet::OpaqueConst<'mem>) -> bool {{
                let value = value.get::<{struct_name}<{generics_use}>>();
                {tests}
                true
            }}
            "#
        );

        invariant_maybe = format!(
            r#"
            {invariant_fn}

            vtable.invariants = Some(invariants);
            "#
        );
    }

    // Generate the impl
    let output = format!(
        r#"
{static_decl}

#[automatically_derived]
unsafe impl<{generics_def}> ::facet::Facet for {struct_name}<{generics_use}> {where_clauses} {{
    const SHAPE: &'static ::facet::Shape = &const {{
        let fields: &'static [::facet::Field] = &const {{[{fields}]}};

        let vtable = &const {{
            let mut vtable = ::facet::value_vtable_inner!(
                Self,
                |f, _opts| ::core::fmt::Write::write_str(f, "{struct_name}")
            );
            {invariant_maybe}
            vtable
        }};

        ::facet::Shape::builder()
            .id(::facet::ConstTypeId::of::<Self>())
            .layout(::core::alloc::Layout::new::<Self>())
            .vtable(vtable)
            .def(::facet::Def::Struct(::facet::Struct::builder()
                .kind({kind})
                .fields(fields)
                .build()))
            {maybe_container_doc}
            .build()
    }};
}}
        "#
    );

    output.into_token_stream().into()
}
