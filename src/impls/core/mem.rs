use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet, Set},
    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure,
};

unsafe impl<T: ~const TypeLayout> const TypeLayout for core::mem::ManuallyDrop<T> {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "transparent",
            fields: &[Field {
                name: "value",
                offset: match unsafe { <T as TypeLayout>::uninit() } {
                    MaybeUninhabited::Inhabited(_) => MaybeUninhabited::Inhabited(0),
                    MaybeUninhabited::Uninhabited => MaybeUninhabited::Uninhabited,
                },
                ty: ::core::any::type_name::<T>(),
            }],
        },
    };

    unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
        match <T as TypeLayout>::uninit() {
            MaybeUninhabited::Uninhabited => MaybeUninhabited::Uninhabited,
            MaybeUninhabited::Inhabited(uninit) => MaybeUninhabited::Inhabited(
                core::mem::MaybeUninit::new(Self::new(uninit.assume_init())),
            ),
        }
    }
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::mem::ManuallyDrop<T> {
    type Output<R: ExpandTypeSet> = Set<Self, tset![T, .. @ R]>;
}

unsafe impl<T: ~const TypeLayout> const TypeLayout for core::mem::MaybeUninit<T> {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Union {
            repr: "transparent",
            fields: &[
                Field {
                    name: "uninit",
                    offset: MaybeUninhabited::Inhabited(0),
                    ty: ::core::any::type_name::<()>(),
                },
                Field {
                    name: "value",
                    offset: match unsafe { <T as TypeLayout>::uninit() } {
                        MaybeUninhabited::Inhabited(_) => MaybeUninhabited::Inhabited(0),
                        MaybeUninhabited::Uninhabited => MaybeUninhabited::Uninhabited,
                    },
                    ty: ::core::any::type_name::<core::mem::ManuallyDrop<T>>(),
                },
            ],
        },
    };

    unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
        MaybeUninhabited::Inhabited(core::mem::MaybeUninit::new(core::mem::MaybeUninit::uninit()))
    }
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::mem::MaybeUninit<T> {
    type Output<R: ExpandTypeSet> = Set<Self, tset![(), core::mem::ManuallyDrop<T>, .. @ R]>;
}
