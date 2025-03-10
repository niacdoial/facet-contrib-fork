use std::alloc::Layout;

use crate::*;

macro_rules! impl_shapely_for_integer {
    ($type:ty, $scalar:expr) => {
        impl Shapely for $type {
            fn shape() -> Shape {
                Shape {
                    name: stringify!($type),
                    layout: Layout::new::<$type>(),
                    innards: Innards::Scalar($scalar),
                    display: Some(|addr: *const u8, f: &mut std::fmt::Formatter| unsafe {
                        write!(f, "{}", *(addr as *const $type))
                    }),
                    debug: Some(|addr: *const u8, f: &mut std::fmt::Formatter| unsafe {
                        write!(f, "{:?}", *(addr as *const $type))
                    }),
                    set_to_default: Some(|addr: *mut ()| unsafe {
                        *(addr as *mut $type) = 0;
                    }),
                    // integers don't need to drop
                    drop_in_place: None,
                }
            }
        }
    };
}

impl_shapely_for_integer!(u8, Scalar::U8);
impl_shapely_for_integer!(u16, Scalar::U16);
impl_shapely_for_integer!(u32, Scalar::U32);
impl_shapely_for_integer!(u64, Scalar::U64);
impl_shapely_for_integer!(i8, Scalar::I8);
impl_shapely_for_integer!(i16, Scalar::I16);
impl_shapely_for_integer!(i32, Scalar::I32);
impl_shapely_for_integer!(i64, Scalar::I64);

macro_rules! impl_schematic_for_float {
    ($type:ty, $scalar:expr) => {
        impl Shapely for $type {
            fn shape() -> Shape {
                Shape {
                    name: stringify!($type),
                    layout: Layout::new::<$type>(),
                    innards: Innards::Scalar($scalar),
                    display: Some(|addr: *const u8, f: &mut std::fmt::Formatter| unsafe {
                        write!(f, "{}", *(addr as *const $type))
                    }),
                    debug: Some(|addr: *const u8, f: &mut std::fmt::Formatter| unsafe {
                        write!(f, "{:?}", *(addr as *const $type))
                    }),
                    set_to_default: Some(|addr: *mut ()| unsafe {
                        *(addr as *mut $type) = 0.0;
                    }),
                    // floats don't need to drop
                    drop_in_place: None,
                }
            }
        }
    };
}

impl_schematic_for_float!(f32, Scalar::F32);
impl_schematic_for_float!(f64, Scalar::F64);

impl Shapely for String {
    fn shape() -> Shape {
        Shape {
            name: "String",
            layout: Layout::new::<String>(),
            innards: Innards::Scalar(Scalar::String),
            display: Some(|addr: *const u8, f: &mut std::fmt::Formatter| unsafe {
                write!(f, "{}", *(addr as *const String))
            }),
            debug: Some(|addr: *const u8, f: &mut std::fmt::Formatter| unsafe {
                write!(f, "{:?}", *(addr as *const String))
            }),
            set_to_default: Some(|addr: *mut ()| unsafe {
                *(addr as *mut String) = String::new();
            }),
            drop_in_place: Some(|addr: *mut ()| unsafe {
                std::ptr::drop_in_place(addr as *mut String);
            }),
        }
    }
}

impl Shapely for bool {
    fn shape() -> Shape {
        Shape {
            name: "bool",
            layout: Layout::new::<bool>(),
            innards: Innards::Scalar(Scalar::Boolean),
            display: Some(|addr: *const u8, f: &mut std::fmt::Formatter| unsafe {
                write!(f, "{}", *(addr as *const bool))
            }),
            debug: Some(|addr: *const u8, f: &mut std::fmt::Formatter| unsafe {
                write!(f, "{:?}", *(addr as *const bool))
            }),
            set_to_default: Some(|addr: *mut ()| unsafe {
                *(addr as *mut bool) = false;
            }),
            // bool doesn't need to drop
            drop_in_place: None,
        }
    }
}

impl Shapely for () {
    fn shape() -> Shape {
        Shape {
            name: "()",
            layout: Layout::new::<()>(),
            innards: Innards::Scalar(Scalar::Nothing),
            display: Some(|_addr: *const u8, f: &mut std::fmt::Formatter| write!(f, "()")),
            debug: Some(|_addr: *const u8, f: &mut std::fmt::Formatter| write!(f, "()")),
            set_to_default: Some(|_addr: *mut ()| {}),
            drop_in_place: None,
        }
    }
}
