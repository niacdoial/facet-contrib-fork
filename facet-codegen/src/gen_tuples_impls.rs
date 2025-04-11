pub(super) fn generate_tuples_impls() -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(65536);

    macro_rules! w {
        ($($t:tt)*) => {
            write!(s, $($t)*).unwrap()
        };
    }

    // Header
    w!("//! GENERATED: DO NOT EDIT — this file is generated from `tuples_impls.rs.j2`\n");
    w!("//! file in the `facet-codegen` crate.\n");
    w!("//!\n");
    w!("//! Edit the template and run `just codegen` to update.\n\n");

    w!("use core::{{alloc::Layout, fmt}};\n\n");
    w!("use crate::{{\n");
    w!(
        "    Characteristic, ConstTypeId, Def, Facet, Field, FieldFlags, MarkerTraits, OpaqueConst, Shape,\n"
    );
    w!("    StructDef, StructKind, TypeNameOpts, ValueVTable,\n");
    w!("}};
\n");

    // Helper functions
    w!("#[inline(always)]\n");
    w!("pub fn write_type_name_list(\n");
    w!("    f: &mut fmt::Formatter<'_>,\n");
    w!("    opts: TypeNameOpts,\n");
    w!("    open: &'static str,\n");
    w!("    delimiter: &'static str,\n");
    w!("    close: &'static str,\n");
    w!("    shapes: &'static [&'static Shape],\n");
    w!(") -> fmt::Result {{\n");
    w!("    f.pad(open)?;\n");
    w!("    if let Some(opts) = opts.for_children() {{\n");
    w!("        for (index, shape) in shapes.iter().enumerate() {{\n");
    w!("            if index > 0 {{\n");
    w!("                f.pad(delimiter)?;\n");
    w!("            }}\n");
    w!("            shape.write_type_name(f, opts)?;\n");
    w!("        }}\n");
    w!("    }} else {{\n");
    w!("        write!(f, \"⋯\")?;\n");
    w!("    }}\n");
    w!("    f.pad(close)?;\n");
    w!("    Ok(())\n");
    w!("}}\n\n");

    w!("macro_rules! field {{\n");
    w!("    ($idx:tt, $ty:ty,) => {{\n");
    w!("        Field::builder()\n");
    w!("            .name(stringify!($idx))\n");
    w!("            .shape($crate::shape_of(&|t: $ty| t.$idx))\n");
    w!("            .offset(core::mem::offset_of!($ty, $idx))\n");
    w!("            .flags(FieldFlags::EMPTY)\n");
    w!("            .build()\n");
    w!("    }};\n");
    w!("}}\n\n");

    // Generate implementations for tuples of different sizes
    let max_tuple_size = 12;

    for n in 1..=max_tuple_size {
        // Generate type parameters and where clauses
        let type_params = (0..n)
            .map(|i| format!("T{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        let where_predicates = (0..n)
            .map(|i| format!("T{}: Facet", i))
            .collect::<Vec<_>>()
            .join(",\n    ");
        let shape_list = (0..n)
            .map(|i| format!("T{}::SHAPE", i))
            .collect::<Vec<_>>()
            .join(", ");

        // Start impl block
        w!(
            "unsafe impl<{}> Facet for {}
",
            type_params,
            // Handle formatting of tuple types correctly
            if n == 1 {
                "(T0,)".to_string()
            } else {
                format!("({})", type_params)
            },
        );
        w!("where\n");
        w!("    {}\n", where_predicates);
        w!("{{\n");
        w!("    const SHAPE: &'static Shape = &const {{\n");

        // type_name function
        w!(
            "        fn type_name<{}>(f: &mut fmt::Formatter, opts: TypeNameOpts) -> fmt::Result\n",
            type_params
        );
        w!("        where\n");
        w!("            {}\n", where_predicates);
        w!("        {{\n");
        if n <= 3 {
            w!(
                "            write_type_name_list(f, opts, \"(\", \", \", \")\", &[{}])\n",
                shape_list
            );
        } else {
            w!("            write_type_name_list(\n");
            w!("                f,\n");
            w!("                opts,\n");
            w!("                \"(\",\n");
            w!("                \", \",\n");
            w!("                \")\",\n");
            w!("                &[\n");
            w!("                    {},\n", shape_list);
            w!("                ],\n");
            w!("            )\n");
        }
        w!("        }}\n\n");

        // Shape builder start
        w!("        Shape::builder()\n");
        w!("            .id(ConstTypeId::of::<");
        if n == 1 {
            w!("(T0,)")
        } else {
            w!("({})", type_params)
        }
        w!(">())\n");
        w!("            .layout(Layout::new::<");
        if n == 1 {
            w!("(T0,)")
        } else {
            w!("({})", type_params)
        }
        w!(">())\n");
        w!("            .vtable(\n");
        w!("                &const {{\n");
        w!("                    let mut builder = ValueVTable::builder()\n");
        w!(
            "                        .type_name(type_name::<{}>)\n",
            type_params
        );
        w!("                        .marker_traits(MarkerTraits::empty());\n\n");

        // Conditional debug and eq implementations
        w!("                    if Characteristic::Eq.all(&[");
        if n <= 5 {
            w!("{}", shape_list);
        } else {
            w!("\n");
            w!("                        {},\n", shape_list);
            w!("                    ");
        }
        w!("]) {{\n");

        // debug implementation
        w!("                        builder = builder.debug(|value, f| {{\n");
        if n == 1 {
            w!("                            let value = unsafe {{ value.as_ref::<(T0,)>() }};\n");
        } else {
            w!(
                "                            let value = unsafe {{ value.as_ref::<({})>() }};\n",
                type_params
            );
        }
        w!("                            write!(f, \"(\")?;\n");

        for i in 0..n {
            if i > 0 {
                w!("                            write!(f, \", \")?;\n");
            }
            w!("                            unsafe {{\n");
            w!(
                "                                let ptr = &value.{0} as *const T{0};\n",
                i
            );
            w!(
                "                                (T{0}::SHAPE.vtable.debug.unwrap_unchecked())(\n",
                i
            );
            w!("                                    OpaqueConst::new(ptr),\n");
            w!("                                    f,\n");
            w!("                                )\n");
            w!("                            }}?;\n");
        }

        w!("                            write!(f, \")\")\n");
        w!("                        }});\n\n");

        // eq implementation
        w!("                        builder = builder.eq(|a, b| {{\n");
        if n == 1 {
            w!("                            let a = unsafe {{ a.as_ref::<(T0,)>() }};\n");
            w!("                            let b = unsafe {{ b.as_ref::<(T0,)>() }};\n\n");
        } else {
            w!(
                "                            let a = unsafe {{ a.as_ref::<({})>() }};\n",
                type_params
            );
            w!(
                "                            let b = unsafe {{ b.as_ref::<({})>() }};\n\n",
                type_params
            );
        }

        // Compare elements except the last one
        for i in 0..n - 1 {
            w!("                            // Compare element {}\n", i);
            w!("                            unsafe {{\n");
            w!(
                "                                let a_ptr = &a.{0} as *const T{0};\n",
                i
            );
            w!(
                "                                let b_ptr = &b.{0} as *const T{0};\n",
                i
            );
            w!(
                "                                if !(T{0}::SHAPE.vtable.eq.unwrap_unchecked())(\n",
                i
            );
            w!("                                    OpaqueConst::new(a_ptr),\n");
            w!("                                    OpaqueConst::new(b_ptr),\n");
            w!("                                ) {{\n");
            w!("                                    return false;\n");
            w!("                                }}\n");
            w!("                            }}\n\n");
        }

        // Special case for the last element
        let last = n - 1;
        w!("                            // Compare last element\n");
        w!("                            unsafe {{\n");
        w!(
            "                                (T{0}::SHAPE.vtable.eq.unwrap_unchecked())(\n",
            last
        );
        w!(
            "                                    OpaqueConst::new(&a.{0} as *const T{0}),\n",
            last
        );
        w!(
            "                                    OpaqueConst::new(&b.{0} as *const T{0}),\n",
            last
        );
        w!("                                )\n");
        w!("                            }}\n");
        w!("                        }});\n");
        w!("                    }}\n\n");

        // Finish vtable builder
        w!("                    builder.build()\n");
        w!("                }},\n");
        w!("            )\n");
        w!("            .def(Def::Struct({{\n");
        w!("                StructDef::builder()\n");
        w!("                    .kind(StructKind::Tuple)\n");
        w!("                    .fields(\n");

        // Generate field array
        if n <= 3 {
            w!("                        &const {{ [");
            for i in 0..n {
                if i > 0 {
                    w!(",\n                        ");
                }
                if n == 1 {
                    w!("field!({}, (T0,),)", i);
                } else {
                    let field_tuple = format!("({},)", type_params);
                    w!("field!({}, {},)", i, field_tuple);
                }
            }
            w!("] }}\n");
        } else {
            w!("                        &const {{\n");
            w!("                            [\n");
            for i in 0..n {
                if n == 1 {
                    w!("                                field!({}, (T0,),)", i);
                } else {
                    let field_tuple = format!("({},)", type_params);
                    w!(
                        "                                field!({}, {},)",
                        i,
                        field_tuple
                    );
                }
                if i < n - 1 {
                    w!(",\n");
                } else {
                    w!("\n");
                }
            }
            w!("                            ]\n");
            w!("                        }},\n");
        }

        // Finish implementation
        w!("                    )\n");
        w!("                    .build()\n");
        w!("            }}))\n");
        w!("            .build()\n");
        w!("    }};\n");
        w!("}}\n");
    }

    s
}
