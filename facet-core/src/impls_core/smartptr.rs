use core::alloc::Layout;

use crate::{
    ConstTypeId, Def, Facet, KnownSmartPointer, OpaqueConst, SmartPointerDef, SmartPointerFlags,
    SmartPointerVTable, value_vtable,
};

unsafe impl<T: Facet> Facet for core::ptr::NonNull<T> {
    const SHAPE: &'static crate::Shape = &const {
        crate::Shape::builder()
            .id(ConstTypeId::of::<Self>())
            .layout(Layout::new::<Self>())
            .def(Def::SmartPointer(
                SmartPointerDef::builder()
                    .pointee(T::SHAPE)
                    .flags(SmartPointerFlags::EMPTY)
                    .known(KnownSmartPointer::NonNull)
                    .vtable(
                        &const {
                            SmartPointerVTable::builder()
                                .borrow_fn(|opaque| {
                                    let ptr = unsafe { opaque.get::<Self>().as_ptr() };
                                    OpaqueConst::new(ptr)
                                })
                                .new_into_fn(|this, ptr| {
                                    let ptr = unsafe { ptr.read::<*mut T>() };
                                    let non_null =
                                        unsafe { core::ptr::NonNull::new_unchecked(ptr) };
                                    unsafe { this.put(non_null) }
                                })
                                .build()
                        },
                    )
                    .build(),
            ))
            .vtable(value_vtable!(core::ptr::NonNull<T>, |f, _opts| write!(
                f,
                "NonNull"
            )))
            .build()
    };
}
