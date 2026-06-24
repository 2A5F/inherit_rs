#![no_std]
#![allow(mixed_script_confusables)]
#![allow(confusable_idents)]
#![allow(uncommon_codepoints)]

use core::marker::PhantomData;

use crate::lambda::{IsEq, Type};

pub mod lambda;
pub mod meta;

pub use traits::*;
mod traits {
    use super::*;

    pub trait ChainId {
        type Id: lambda::Λ;
    }

    pub trait GetThis<This> {
        unsafe fn this(&self) -> *mut This;
    }

    pub trait Field<T>: Default + ChainId {
        type Type;

        unsafe fn write(&self, this: *mut T, val: Self::Type);
    }

    pub trait Fields<N: lambda::Λ>: ChainId {
        type Removed: lambda::Λ;
        type Current: lambda::Λ;
    }

    pub trait FieldsCond<N: lambda::Λ, Cond: lambda::Λ>: Sized + Fields<N> {
        fn remove_chain(
            self,
        ) -> (
            <<Self as Fields<N>>::Removed as lambda::Λ>::Type,
            <<Self as Fields<N>>::Current as lambda::Λ>::Type,
        );
    }
}

pub use ctor_this::CtorThis;
mod ctor_this {
    use super::*;

    #[derive(Debug)]
    pub struct CtorThis<This>(*mut This);

    impl<This> CtorThis<This> {
        pub fn new(this: *mut This) -> Self {
            CtorThis(this)
        }
    }

    impl<This> ChainId for CtorThis<This> {
        type Id = lambda::Zero;
    }
    impl<This> GetThis<This> for CtorThis<This> {
        unsafe fn this(&self) -> *mut This {
            self.0
        }
    }

    impl<N: lambda::Λ, This> Fields<N> for CtorThis<This> {
        type Removed = Type<Self>;
        type Current = Type<()>;
    }

    impl<N: lambda::Λ, Cond: lambda::Λ, This> FieldsCond<N, Cond> for CtorThis<This> {
        fn remove_chain(self) -> (Self, ()) {
            (self, ())
        }
    }
}

pub use field_chain::FieldChain;
mod field_chain {
    use super::*;
    use lambda::*;

    #[derive(Debug, Clone, Copy, Default)]
    pub struct FieldChain<Target, Parent, Current>(Parent, PhantomData<(Target, Parent, Current)>);

    impl<Target, Parent, Current> FieldChain<Target, Parent, Current> {
        pub fn new(parent: Parent) -> Self {
            FieldChain(parent, PhantomData)
        }
    }

    impl<Target, Parent, Current: ChainId> ChainId for FieldChain<Target, Parent, Current> {
        type Id = Current::Id;
    }

    impl<Target, Parent, Current> GetThis<Target> for FieldChain<Target, Parent, Current>
    where
        Parent: GetThis<Target>,
    {
        unsafe fn this(&self) -> *mut Target {
            unsafe { self.0.this() }
        }
    }

    impl<Target, Parent, Current, N: Λ> Fields<N> for FieldChain<Target, Parent, Current>
    where
        Current: Field<Target>,
        Parent: Fields<N>,
    {
        type Removed = If<
            IsEq<Self::Id, N>,
            Type<Parent>,
            Type<FieldChain<Target, <Parent::Removed as Λ>::Type, Current>>,
        >;
        type Current = If<IsEq<Self::Id, N>, Type<Current>, Parent::Current>;
    }

    impl<Target, Parent, Current, N: Λ> FieldsCond<N, True> for FieldChain<Target, Parent, Current>
    where
        Current: Field<Target>,
        Parent: Fields<N>,
        Self: Fields<N, Removed = Type<Parent>, Current = Type<Current>>,
    {
        fn remove_chain(self) -> (Parent, Current) {
            (self.0, Default::default())
        }
    }

    impl<Target, Parent, Current, N: Λ> FieldsCond<N, False> for FieldChain<Target, Parent, Current>
    where
        Current: Field<Target>,
        Parent: FieldsCond<N, IsEq<<Parent as ChainId>::Id, N>>,
        Self: Fields<
                N,
                Removed = Type<FieldChain<Target, <Parent::Removed as Λ>::Type, Current>>,
                Current = Parent::Current,
            >,
    {
        fn remove_chain(
            self,
        ) -> (
            <<Self as Fields<N>>::Removed as lambda::Λ>::Type,
            <<Self as Fields<N>>::Current as lambda::Λ>::Type,
        ) {
            let (parent, current) = self.0.remove_chain();
            (FieldChain(parent, PhantomData), current)
        }
    }
}

#[cfg(test)]
#[allow(non_camel_case_types)]
pub mod tests {
    use core::mem::MaybeUninit;

    use super::*;

    extern crate std;

    #[derive(Debug)]
    pub struct Some {
        pub foo: i32,
        pub bar: f32,
    }

    #[derive(Debug, Clone, Copy, Default)]
    pub struct field_foo;
    #[derive(Debug, Clone, Copy, Default)]
    pub struct field_bar;

    impl ChainId for field_foo {
        type Id = lambda::One;
    }
    impl Field<Some> for field_foo {
        type Type = i32;

        unsafe fn write(&self, this: *mut Some, val: Self::Type) {
            unsafe { core::ptr::write(&mut (*this).foo, val) }
        }
    }

    impl ChainId for field_bar {
        type Id = lambda::Two;
    }
    impl Field<Some> for field_bar {
        type Type = f32;

        unsafe fn write(&self, this: *mut Some, val: Self::Type) {
            unsafe { core::ptr::write(&mut (*this).bar, val) }
        }
    }

    pub trait set_field_foo {
        type Output;
        fn foo(self, val: i32) -> Self::Output;
    }

    impl<T> set_field_foo for T
    where
        T: ChainId,
        T: GetThis<Some>,
        T: FieldsCond<
                <field_foo as ChainId>::Id,
                IsEq<<T as ChainId>::Id, <field_foo as ChainId>::Id>,
            >,
        T: Fields<<field_foo as ChainId>::Id, Current = Type<field_foo>>,
    {
        type Output = <T::Removed as lambda::Λ>::Type;
        fn foo(self, val: i32) -> Self::Output {
            let this = unsafe { self.this() };
            let (next, current) = self.remove_chain();
            unsafe { current.write(this, val) };
            next
        }
    }
    pub trait set_field_bar {
        type Output;
        fn bar(self, val: f32) -> Self::Output;
    }

    impl<T> set_field_bar for T
    where
        T: ChainId,
        T: GetThis<Some>,
        T: FieldsCond<
                <field_bar as ChainId>::Id,
                IsEq<<T as ChainId>::Id, <field_bar as ChainId>::Id>,
            >,
        T: Fields<<field_bar as ChainId>::Id, Current = Type<field_bar>>,
    {
        type Output = <T::Removed as lambda::Λ>::Type;
        fn bar(self, val: f32) -> Self::Output {
            let this = unsafe { self.this() };
            let (next, current) = self.remove_chain();
            unsafe { current.write(this, val) };
            next
        }
    }

    #[test]
    fn test0() {
        let a: FieldChain<Some, FieldChain<Some, CtorThis<Some>, field_foo>, field_bar> =
            FieldChain::new(FieldChain::new(CtorThis::new(core::ptr::null_mut())));
        let r = <FieldChain<Some, FieldChain<Some, CtorThis<Some>, field_foo>, field_bar> as FieldsCond<
            lambda::Two,
            lambda::True,
        >>::remove_chain(a);
        std::println!("{}", std::any::type_name_of_val(&r));

        let a: FieldChain<Some, FieldChain<Some, CtorThis<Some>, field_foo>, field_bar> =
            FieldChain::new(FieldChain::new(CtorThis::new(core::ptr::null_mut())));
        let r = <FieldChain<Some, FieldChain<Some, CtorThis<Some>, field_foo>, field_bar> as FieldsCond<
            lambda::One,
            lambda::False,
        >>::remove_chain(a);
        std::println!("{}", std::any::type_name_of_val(&r));
    }

    #[test]
    fn test1() {
        let mut some = MaybeUninit::<Some>::zeroed();
        let a: FieldChain<Some, FieldChain<Some, CtorThis<Some>, field_foo>, field_bar> =
            FieldChain::new(FieldChain::new(CtorThis::new(some.as_mut_ptr())));
        let _ = a.foo(456).bar(123.0);
        let some = unsafe { some.assume_init() };
        std::println!("{:?}", some);
        assert_eq!(some.foo, 456);
        assert_eq!(some.bar, 123.0);
    }
}
