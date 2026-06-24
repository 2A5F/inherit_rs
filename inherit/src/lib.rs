#![no_std]
#![allow(mixed_script_confusables)]
#![allow(confusable_idents)]
#![allow(uncommon_codepoints)]

use core::{marker::PhantomData, ptr::NonNull};

pub mod lambda;
pub mod meta;

pub trait Field {
    type Id: lambda::Λ;
    type Type;
}

pub trait CtorTarget {
    type Id: lambda::Λ;
    type Target;
    unsafe fn this(&self) -> NonNull<Self::Target>;
}

pub trait CtorChain<N, C>: CtorTarget {
    type Removed;
    type Current;

    unsafe fn done(self) -> Self::Removed;
}

pub trait TakeChain {
    type Chain;
    fn take_chain(self) -> Self::Chain;
}

pub use ctor_this::CtorThis;
mod ctor_this {
    use super::*;
    #[derive(Debug)]
    pub struct CtorThis<T>(NonNull<T>);
    impl<T> CtorThis<T> {
        pub fn new(this: NonNull<T>) -> Self {
            Self(this)
        }
    }
    impl<T> CtorTarget for CtorThis<T> {
        type Id = lambda::Zero;
        type Target = T;

        unsafe fn this(&self) -> NonNull<Self::Target> {
            self.0
        }
    }
    impl<T, N, C> CtorChain<N, C> for CtorThis<T> {
        type Removed = Self;
        type Current = ();

        unsafe fn done(self) -> Self::Removed {
            self
        }
    }
}

pub use field_chain::FieldChain;

mod field_chain {
    use super::*;
    use crate::lambda::{False, IsEq, True};

    #[derive(Debug)]
    pub struct FieldChain<P, F>(P, PhantomData<F>);
    impl<P, F: Field> FieldChain<P, F> {
        pub fn new(parent: P) -> Self {
            Self(parent, PhantomData)
        }
    }
    impl<P, F: Field> CtorTarget for FieldChain<P, F>
    where
        P: CtorTarget,
    {
        type Id = F::Id;
        type Target = P::Target;

        unsafe fn this(&self) -> NonNull<Self::Target> {
            unsafe { self.0.this() }
        }
    }
    impl<P, F: Field, N: lambda::Λ> CtorChain<N, True> for FieldChain<P, F>
    where
        P: CtorTarget,
        P: CtorChain<N, IsEq<N, F::Id>>,
    {
        type Removed = P;
        type Current = F;

        unsafe fn done(self) -> Self::Removed {
            self.0
        }
    }
    impl<P, F: Field, N: lambda::Λ> CtorChain<N, False> for FieldChain<P, F>
    where
        P: CtorTarget,
        P: CtorChain<N, IsEq<N, <P as CtorTarget>::Id>>,
    {
        type Removed = FieldChain<P::Removed, F>;
        type Current = P::Current;

        unsafe fn done(self) -> Self::Removed {
            unsafe { FieldChain::new(self.0.done()) }
        }
    }
}

pub use partial_this::PartialThis;
mod partial_this {
    use super::*;
    #[derive(Debug)]
    pub struct PartialThis<C>(C);

    impl<C> PartialThis<C> {
        pub fn new(chain: C) -> Self {
            Self(chain)
        }
    }
    impl<C> TakeChain for PartialThis<C> {
        type Chain = C;
        fn take_chain(self) -> Self::Chain {
            self.0
        }
    }
}

#[cfg(test)]
#[allow(non_camel_case_types)]
mod tests {
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

    impl Field for field_foo {
        type Id = lambda::One;
        type Type = i32;
    }

    impl Field for field_bar {
        type Id = lambda::Two;
        type Type = f32;
    }

    pub trait set_field_foo {
        type Output;
        fn foo(self, val: i32) -> Self::Output;
    }

    pub trait set_field_bar {
        type Output;
        fn bar(self, val: f32) -> Self::Output;
    }

    impl<C> set_field_foo for PartialThis<C>
    where
        C: CtorTarget<Target = Some>,
        C: CtorChain<
                <field_foo as Field>::Id,
                lambda::IsEq<<C as CtorTarget>::Id, <field_foo as Field>::Id>,
                Current = field_foo,
            >,
    {
        type Output = PartialThis<C::Removed>;

        fn foo(self, val: i32) -> Self::Output {
            unsafe {
                let c = self.take_chain();
                core::ptr::write(&mut (*c.this().as_ptr()).foo, val);
                PartialThis::new(c.done())
            }
        }
    }

    impl<C> set_field_bar for PartialThis<C>
    where
        C: CtorTarget<Target = Some>,
        C: CtorChain<
                <field_bar as Field>::Id,
                lambda::IsEq<<C as CtorTarget>::Id, <field_bar as Field>::Id>,
                Current = field_bar,
            >,
    {
        type Output = PartialThis<C::Removed>;

        fn bar(self, val: f32) -> Self::Output {
            unsafe {
                let c = self.take_chain();
                core::ptr::write(&mut (*c.this().as_ptr()).bar, val);
                PartialThis::new(c.done())
            }
        }
    }

    #[test]
    fn test1() {
        let mut some = Some { foo: 0, bar: 0.0 };
        let c: PartialThis<FieldChain<FieldChain<CtorThis<Some>, field_foo>, field_bar>> =
            PartialThis::new(FieldChain::new(FieldChain::new(CtorThis::new(
                NonNull::from_mut(&mut some),
            ))));
        let a = c.foo(1);
        let b = a.bar(1.0);
        std::println!("{:?}", b);
        std::println!("{:?}", some);
        assert_eq!(some.foo, 1);
        assert_eq!(some.bar, 1.0);

        let mut some = Some { foo: 0, bar: 0.0 };
        let c: PartialThis<FieldChain<FieldChain<CtorThis<Some>, field_foo>, field_bar>> =
            PartialThis::new(FieldChain::new(FieldChain::new(CtorThis::new(
                NonNull::from_mut(&mut some),
            ))));
        let a = c.bar(1.0);
        let b = a.foo(1);
        std::println!("{:?}", b);
        std::println!("{:?}", some);
        assert_eq!(some.foo, 1);
        assert_eq!(some.bar, 1.0);
    }
}
