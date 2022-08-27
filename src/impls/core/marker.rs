use crate::{TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure};

unsafe impl<T: TypeLayout> TypeLayout for core::marker::PhantomData<T> {
    type Static = core::marker::PhantomData<T::Static>;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "",
            fields: &[],
        },
    };
    const UNINIT: core::mem::ManuallyDrop<Self> =
        core::mem::ManuallyDrop::new(core::marker::PhantomData);
}

unsafe impl<T: ~const TypeGraph> const TypeGraph for core::marker::PhantomData<T> {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        graph.insert(&Self::TYPE_LAYOUT);
    }
}

unsafe impl TypeLayout for core::marker::PhantomPinned {
    type Static = Self;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "",
            fields: &[],
        },
    };
    const UNINIT: core::mem::ManuallyDrop<Self> =
        core::mem::ManuallyDrop::new(core::marker::PhantomPinned);
}

unsafe impl const TypeGraph for core::marker::PhantomPinned {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        graph.insert(&Self::TYPE_LAYOUT);
    }
}
