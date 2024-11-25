//! Helper module to compute the set of types that a type links to and expand it
//! into the complete type graph.

#[doc(hidden)]
pub trait ComputeSet: sealed::ComputeSet {
    const LEN: usize;

    type Output<H: ComputeTypeSet>: ExpandTypeSet;

    type Head: ComputeTypeSet;
    type Tail: ExpandTypeSet;

    type TyHList: 'static + Copy + core::marker::Freeze;
    const TYS: &'static Self::TyHList;
}

pub mod foo {
    use core::{mem::MaybeUninit, ops::Deref};

    use crate::{Field, Variant, TypeLayoutInfo};

    use super::{ComputeSet, ComputeTypeSet, ExpandTypeSet, private::Empty};

    struct Node<
        'a,
        'b,
        F: Deref<Target = [Field<'a>]> = &'a [Field<'a>],
        V: Deref<Target = [Variant<'a, F>]> = &'a [Variant<'a, F>],
    > {
        ty: TypeLayoutInfo<'a, F, V>,
        next: Option<&'b Self>,
    }

    const fn str_eq(a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let (a, b) = (a.as_bytes(), b.as_bytes());

        let mut i = 0;

        while i < a.len() {
            if a[i] != b[i] {
                return false;
            }

            i += 1;
        }

        true
    }

    pub const fn typset_len<T: ComputeTypeSet>() -> usize {
        expand_len::<T, Empty>(None)
    }

    pub const fn typeset_tys<'a, 'b, T: ComputeTypeSet>(tys: &'a mut [MaybeUninit<TypeLayoutInfo<'b>>]) -> usize {
        expand_tys::<T, Empty>(tys, 0)
    }

    const fn expand_len<T: ComputeTypeSet, S: ExpandTypeSet>(tys: Option<&Node>) -> usize {
        const fn len(mut tys: Option<&Node>) -> usize {
            let mut len = 0;

            while let Some(t) = tys {
                len += 1;
                tys = t.next;
            }

            len
        }

        let info = T::TYPE_LAYOUT;
        let mut it = &tys;
        while let Some(i) = it {
            if str_eq(i.ty.name, info.name) {
                if S::IS_EMPTY {
                    return len(tys);
                }
                return expand_len::<S::Head, S::Tail>(tys);
            }

            it = &i.next;
        }
        let mut cons = Node {
            ty: info,
            next: tys,
        };
        if <T::Output<S> as ExpandTypeSet>::IS_EMPTY {
            return len(Some(&mut cons));
        }
        expand_len::<<T::Output<S> as ComputeSet>::Head, <T::Output<S> as ComputeSet>::Tail>(Some(&mut cons))
    }

    const fn expand_tys<'a, 'b, T: ComputeTypeSet, S: ExpandTypeSet>(tys: &'a mut [MaybeUninit<TypeLayoutInfo<'b>>], tys_len: usize) -> usize {
        let info = T::TYPE_LAYOUT;
        let mut i = 0;
        while i < tys_len {
            if str_eq(unsafe { tys[i].assume_init_ref() }.name, info.name) {
                if S::IS_EMPTY {
                    return tys_len;
                }
                return expand_tys::<S::Head, S::Tail>(tys, tys_len);
            }
            i += 1;
        }
        tys[tys_len] = MaybeUninit::new(info);
        if <T::Output<S> as ExpandTypeSet>::IS_EMPTY {
            return tys_len + 1;
        }
        expand_tys::<<T::Output<S> as ComputeSet>::Head, <T::Output<S> as ComputeSet>::Tail>(tys, tys_len + 1)
    }
}

mod sealed {
    pub trait ComputeSet {}

    impl ComputeSet for super::private::Empty {}
    impl<H2: super::ComputeTypeSet, T: ComputeSet> ComputeSet for super::private::Cons<H2, T> {}
}

type Set<H, T> = <T as ComputeSet>::Output<H>;

/// Computes the set of types that a type links to.
///
/// # Safety
///
/// It is only safe to implement this trait if it accurately includes
/// all inner component types that are referenced by this type's layout. Use
/// [`#[derive(TypeLayout)]`](const_type_layout_derive::TypeLayout) instead.
///
/// # Example
///
/// The struct `Foo` with `u8` and `u16` fields links to `u8` and `u16`:
///
/// ```rust
/// # #![feature(const_type_name)]
/// # #![feature(offset_of)]
/// # use const_type_layout::{
/// #    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure,
/// # };
/// # use const_type_layout::inhabited;
/// # use const_type_layout::typeset::{ComputeTypeSet, ExpandTypeSet, tset};
/// struct Foo {
///     a: u8,
///     b: u16,
/// }
///
/// # unsafe impl TypeLayout for Foo {
/// #     const INHABITED: MaybeUninhabited = inhabited::all![u8, u16];
/// #
/// #     const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
/// #         name: ::core::any::type_name::<Self>(),
/// #         size: ::core::mem::size_of::<Self>(),
/// #         alignment: ::core::mem::align_of::<Self>(),
/// #         structure: TypeStructure::Struct {
/// #             repr: "",
/// #             fields: &[
/// #                 Field {
/// #                     name: "a",
/// #                     offset: MaybeUninhabited::new::<u8>(::core::mem::offset_of!(Self, a)),
/// #                     ty: ::core::any::type_name::<u8>(),
/// #                 },
/// #                 Field {
/// #                     name: "b",
/// #                     offset: MaybeUninhabited::new::<u16>(::core::mem::offset_of!(Self, b)),
/// #                     ty: ::core::any::type_name::<u16>(),
/// #                 },
/// #             ],
/// #         },
/// #     };
/// # }
///
/// unsafe impl ComputeTypeSet for Foo {
///     type Output<T: ExpandTypeSet> = tset![u8, u16];
/// }
/// ```
///
/// Note that to you implement [`ComputeTypeSet`] you must also implement
/// [`crate::TypeLayout`] for it.
pub unsafe trait ComputeTypeSet: crate::TypeLayout {
    /// Extend the set `T` into a (larger) set containing also the types this
    /// type links to.
    ///
    /// Enums implementing [`crate::TypeLayout`] and [`ComputeTypeSet`]
    /// manually should include [`core::mem::Discriminant<Self>`] in
    /// their [`ComputeTypeSet::Output`] using the [`tset`] helper macro.
    type Output<T: ExpandTypeSet>: ExpandTypeSet;
}

/// Helper macro to expand a list of types, e.g. `H, R1, R2`, and an optional
/// tail, `.. @ T`, into a set of types.
///
/// This macro is used when implementing the [`ComputeTypeSet::Output`]
/// associated type to specify the list of types a type links to.
pub macro tset {
    () => { private::Empty },
    (.. @ $T:tt) => { $T },
    ($H:ty $(, $R:ty)*) => {
        Set<$H, tset![$($R),*]>
    },
    ($H:ty, $($R:ty,)* .. @ $T:ty ) => {
        Set<$H, tset![$($R,)* .. @ $T]>
    },
}

#[doc(hidden)]
pub trait ExpandTypeSet: ComputeSet {
    const IS_EMPTY: bool;
    type Output<T: ExpandTypeSet>: ExpandTypeSet;
}

impl ExpandTypeSet for private::Empty {
    const IS_EMPTY: bool = true;
    type Output<T: ExpandTypeSet> = T;
}

impl<H: ComputeTypeSet, T: ExpandTypeSet> ExpandTypeSet for private::Cons<H, T> {
    const IS_EMPTY: bool = false;
    type Output<R: ExpandTypeSet> =
        <T as ExpandTypeSet>::Output<Set<H, <H as ComputeTypeSet>::Output<R>>>;
}

// #[doc(hidden)]
// pub trait TypeSetFixedPoint: ExpandTypeSet {
//     type Output: ExpandTypeSet;
// }

// impl<T: ExpandTypeSet> TypeSetFixedPoint for T {
//     type Output = <T as private::ComputeTypeSetFixedPoint<
//         <T as ExpandTypeSet>::Output<private::Empty>,
//     >>::Output;
// }

mod private {
    use super::{sealed, ComputeSet, ComputeTypeSet, ExpandTypeSet, Set};

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Empty;

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Cons<H, T> {
        head: H,
        tail: T,
    }

    impl ComputeSet for Empty {
        type Output<H: ComputeTypeSet> = Cons<H, Self>;
        type TyHList = Self;

        type Head = !; // FIXME
        type Tail = Self;

        const LEN: usize = 0;
        const TYS: &'static Self::TyHList = &Self;
    }

    impl<H2: ComputeTypeSet, T: ExpandTypeSet> ComputeSet for Cons<H2, T> {
        type Output<H1: ComputeTypeSet> = Cons<H1, Self>;//<Self as ComputeCons<H1>>::Output;
        type TyHList = Cons<&'static crate::TypeLayoutInfo<'static>, T::TyHList>;

        type Head = H2;
        type Tail = T;

        const LEN: usize = T::LEN + 1;
        const TYS: &'static Self::TyHList = &Cons {
            head: &H2::TYPE_LAYOUT,
            tail: *T::TYS,
        };
    }

    // pub trait ComputeCons<H: ComputeTypeSet>: sealed::ComputeSet {
    //     type Output: ExpandTypeSet;
    // }

    // impl<H: ComputeTypeSet> ComputeCons<H> for Empty {
    //     type Output = Cons<H, Self>;
    // }

    // impl<H: ComputeTypeSet, T: ExpandTypeSet> ComputeCons<H> for Cons<H, T> {
    //     type Output = Self;
    // }

    // impl<H1: ComputeTypeSet, H2: ComputeTypeSet, T: ExpandTypeSet> ComputeCons<H1> for Cons<H2, T> {
    //     default type Output = Cons<H2, Set<H1, T>>;
    // }

    // pub trait ComputeTypeSetFixedPoint<E: ExpandTypeSet>: ExpandTypeSet {
    //     type Output: ExpandTypeSet;
    // }

    // impl<T: ExpandTypeSet, E: ExpandTypeSet> ComputeTypeSetFixedPoint<E> for T {
    //     default type Output = <E as ComputeTypeSetFixedPoint<<E as ExpandTypeSet>::Output<Empty>>>::Output;
    // }

    // trait True {}
    // struct Assert<const ASSERT: bool>;
    // impl True for Assert<true> {}

    // impl<T: ExpandTypeSet, E: ExpandTypeSet> ComputeTypeSetFixedPoint<E> for T
    // where
    //     Assert<{ T::LEN == E::LEN }>: True,
    // {
    //     type Output = T;
    // }
}

// pub(super) type TypeSet<T> =
//     <Set<T, <T as ComputeTypeSet>::Output<private::Empty>> as TypeSetFixedPoint>::Output;
