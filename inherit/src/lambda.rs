//! Type metaprogramming based on lambda calculus
//!
//! # Examples
//! ```
//! # use coplt_inherit::lambda::*;
//! let a: Count<Dec<Add<Two, Two>>> = Default::default();
//! assert_eq!(a.value(), 3);
//! ```
//!
//! ```
//! # use coplt_inherit::lambda::*;
//! let a: Count<Mul<Three, Three>> = Default::default();
//! assert_eq!(a.value(), 9);
//! ```
//!
//! ```
//! # use coplt_inherit::lambda::*;
//! let a: IsEq<Five, Five> = Default::default();
//! assert!(matches!(a, True));
//! ```
//!

/// A trait representing a type-level function in the lambda calculus.
pub trait Λ {
    type Type;
    /// The associated type representing the result of applying the function to an argument.
    type A<X: Λ>: Λ;
}

use core::marker::PhantomData;

use private::*;
mod private {
    use super::*;

    /// ```text
    /// λ x with c. c
    /// ```
    #[derive(Debug, Default, Clone, Copy)]
    pub struct Cap0<X>(X);
    impl<X: Λ> Λ for Cap0<X> {
        type Type = ();
        type A<_X: Λ> = X;
    }
}

/// Convert a type-level number to a value-level number.
pub type Count<X> = <count_private::CalcCount as Λ>::A<X>;
mod count_private {
    use super::*;
    #[repr(u8)]
    #[derive(Debug, Default, Clone, Copy)]
    pub enum 𝕀 {
        #[default]
        One = 0,
    }
    #[derive(Debug, Default, Clone, Copy)]
    pub struct ℤ;
    impl Λ for ℤ {
        type Type = ();
        type A<X: Λ> = ℤ;
    }
    impl ℤ {
        pub const fn value(&self) -> usize {
            0
        }
    }
    #[derive(Debug, Default, Clone, Copy)]
    pub struct ℂ<T = 𝕀>(T);
    impl<T> Λ for ℂ<T> {
        type Type = ();
        type A<X: Λ> = ℂ<(T, X)>;
    }
    impl<T> ℂ<T> {
        pub const fn value(&self) -> usize {
            core::mem::size_of::<T>()
        }
    }
    #[derive(Debug, Default, Clone, Copy)]
    pub struct CalcCount;
    impl Λ for CalcCount {
        type Type = ();
        type A<X: Λ> = <X::A<ℂ> as Λ>::A<ℤ>;
    }
}

/// Storage a Type
#[derive(Debug, Default, Clone, Copy)]
pub struct Type<T>(PhantomData<T>);
impl<T> Λ for Type<T> {
    type Type = T;
    type A<X: Λ> = Self;
}

/// `Identity`
/// ```text
/// λ x. x
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct Id;
impl Λ for Id {
    type Type = ();
    type A<X: Λ> = X;
}

/// `True` | `Constant`
/// ```text
/// λ a. λ b. a
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct True;
pub type False = Z;

/// `Zero` | `False`
/// ```text
/// λ a. λ b. b
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct Z;
mod bool_private {
    use super::*;
    #[derive(Debug, Default, Clone, Copy)]
    pub struct False;
    #[derive(Debug, Default, Clone, Copy)]
    pub struct True<A>(A);

    impl Λ for Z {
        type Type = ();
        type A<F: Λ> = False;
    }
    impl Λ for False {
        type Type = ();
        type A<X: Λ> = X;
    }
    impl<A: Λ> Λ for True<A> {
        type Type = ();
        type A<X: Λ> = A;
    }
    impl Λ for super::True {
        type Type = ();
        type A<A: Λ> = True<A>;
    }
}

/// ```text
/// λ c. λ a. λ b. c a b
/// ```
pub type If<Cond, Then, Else> = <<Cond as Λ>::A<Then> as Λ>::A<Else>;

/// ```text
/// λ c. c False True
/// ```
pub type Not<B> = If<B, False, True>;

/// ```text
/// λ a. λ b. a b False
/// ```
pub type And<A, B> = <<A as Λ>::A<B> as Λ>::A<False>;

/// ```text
/// λ a. λ b. a True b
/// ```
pub type Or<A, B> = <<A as Λ>::A<True> as Λ>::A<B>;

/// ```text
/// λ n. n (λ x. False) True
/// ```
pub type IsZero<N> = <is_zero_private::TIsZero as Λ>::A<N>;
mod is_zero_private {
    use super::*;

    #[derive(Debug, Default, Clone, Copy)]
    pub struct TIsZero;

    #[derive(Debug, Default, Clone, Copy)]
    pub struct IZ {}
    impl Λ for IZ {
        type Type = ();
        type A<X: Λ> = False;
    }
    impl Λ for TIsZero {
        type Type = ();
        type A<N: Λ> = <N::A<IZ> as Λ>::A<True>;
    }
}

/// ```text
/// λ m. λ n. And (IsZero (Sub m n)) (IsZero (Sub n m))
/// ```
pub type IsEq<M, N> = And<IsZero<Sub<M, N>>, IsZero<Sub<N, M>>>;

/// ```text
/// λ n. λ f. λ x. f (n f x)
/// ```
pub type Inc<N> = <inc_private::TInc as Λ>::A<N>;
mod inc_private {
    /// ```text
    /// λ n. λ f. λ x. f (n f x)
    /// ```
    #[derive(Debug, Default, Clone, Copy)]
    pub struct TInc;
    use super::*;

    #[derive(Debug, Default, Clone, Copy)]
    pub struct I<N>(N);

    #[derive(Debug, Default, Clone, Copy)]
    pub struct II<N, F>(N, F);
    impl Λ for TInc {
        type Type = ();
        type A<N: Λ> = I<N>;
    }
    impl<N: Λ> Λ for I<N> {
        type Type = ();
        type A<F: Λ> = II<N, F>;
    }
    impl<N: Λ, F: Λ> Λ for II<N, F>
    where
        <N as Λ>::A<F>: Λ,
    {
        type Type = ();
        type A<X: Λ> = F::A<<N::A<F> as Λ>::A<X>>;
    }
}

/// ```text
/// λ m. λ n. λ f. λ x. m f (n f x)
/// ```
pub type Add<A, B> = <<add_private::TAdd as Λ>::A<A> as Λ>::A<B>;
mod add_private {
    use super::*;

    /// ```text
    /// λ m. λ n. λ f. λ x. m f (n f x)
    /// ```
    #[derive(Debug, Default, Clone, Copy)]
    pub struct TAdd;

    #[derive(Debug, Default, Clone, Copy)]
    pub struct A<M>(M);

    #[derive(Debug, Default, Clone, Copy)]
    pub struct AA<M, N>(M, N);

    #[derive(Debug, Default, Clone, Copy)]
    pub struct AAA<M, N, F>(M, N, F);
    impl Λ for TAdd {
        type Type = ();
        type A<M: Λ> = A<M>;
    }
    impl<M: Λ> Λ for A<M> {
        type Type = ();
        type A<N: Λ> = AA<M, N>;
    }
    impl<M: Λ, N: Λ> Λ for AA<M, N> {
        type Type = ();
        type A<F: Λ> = AAA<M, N, F>;
    }
    impl<M: Λ, N: Λ, F: Λ> Λ for AAA<M, N, F>
    where
        <M as Λ>::A<F>: Λ,
        <N as Λ>::A<F>: Λ,
    {
        type Type = ();
        type A<X: Λ> = <<M as Λ>::A<F> as Λ>::A<<<N as Λ>::A<F> as Λ>::A<X>>;
    }
}

/// ```text
/// λ n. λ f. λ x. n (λ g. λ h. h (g f)) (λ u. x) (λ u. u)
/// ```
pub type Dec<N> = <pre_desc_private::TPreDec as Λ>::A<N>;
mod pre_desc_private {
    use super::*;
    /// ```text
    /// λ n. λ f. λ x. n (λ g. λ h. h (g f)) (λ u. x) (λ u. u)
    /// ```
    #[derive(Debug, Default, Clone, Copy)]
    pub struct TPreDec;

    #[derive(Debug, Default, Clone, Copy)]
    pub struct PD0<N>(N);

    #[derive(Debug, Default, Clone, Copy)]
    pub struct PD1<N, F>(N, F);

    #[derive(Debug, Default, Clone, Copy)]
    pub struct PD2<F>(F);

    #[derive(Debug, Default, Clone, Copy)]
    pub struct PD3<F, G>(F, G);
    impl Λ for TPreDec {
        type Type = ();
        type A<N: Λ> = PD0<N>;
    }
    impl<N: Λ> Λ for PD0<N> {
        type Type = ();
        type A<F: Λ> = PD1<N, F>;
    }
    impl<F: Λ> Λ for PD2<F> {
        type Type = ();
        type A<G: Λ> = PD3<F, G>;
    }
    impl<F: Λ, G: Λ> Λ for PD3<F, G> {
        type Type = ();
        type A<H: Λ> = H::A<G::A<F>>;
    }
    impl<N: Λ, F: Λ> Λ for PD1<N, F> {
        type Type = ();
        type A<X: Λ> = <<N::A<PD2<F>> as Λ>::A<Cap0<X>> as Λ>::A<Id>;
    }
}

/// ```text
/// λ m. λ n. n Dec m
/// ```
pub type Sub<A, B> = <<sub_private::TSub as Λ>::A<A> as Λ>::A<B>;
mod sub_private {
    use super::*;
    #[derive(Debug, Default, Clone, Copy)]
    /// ```text
    /// λ m. λ n. n Dec m
    /// ```
    pub struct TSub;
    #[derive(Debug, Default, Clone, Copy)]
    pub struct S<M>(M);
    impl Λ for TSub {
        type Type = ();
        type A<M: Λ> = S<M>;
    }
    impl<M: Λ> Λ for S<M> {
        type Type = ();
        type A<N: Λ> = <N::A<pre_desc_private::TPreDec> as Λ>::A<M>;
    }
}

/// ```text
/// λ m. λ n. λ f. m (n f)
/// ```
pub type Mul<A, B> = <<mul_private::TMul as Λ>::A<A> as Λ>::A<B>;
mod mul_private {
    use super::*;

    /// ```text
    /// λ m. λ n. λ f. m (n f)
    /// ```
    #[derive(Debug, Default, Clone, Copy)]
    pub struct TMul;
    #[derive(Debug, Default, Clone, Copy)]
    pub struct X<M>(M);
    #[derive(Debug, Default, Clone, Copy)]
    pub struct XX<M, N>(M, N);
    impl Λ for TMul {
        type Type = ();
        type A<M: Λ> = X<M>;
    }
    impl<M: Λ> Λ for X<M> {
        type Type = ();
        type A<N: Λ> = XX<M, N>;
    }
    impl<M: Λ, N: Λ> Λ for XX<M, N> {
        type Type = ();
        type A<F: Λ> = M::A<N::A<F>>;
    }
}

pub type Zero = Z;
pub type One = Inc<Zero>;
pub type Two = Inc<One>;
pub type Three = Inc<Two>;
pub type Four = Inc<Three>;
pub type Five = Inc<Four>;
pub type Six = Inc<Five>;
pub type Seven = Inc<Six>;
pub type Eight = Inc<Seven>;
pub type Nine = Inc<Eight>;
pub type Ten = Inc<Nine>;

/// Substitution
/// ```text
/// λ x. λ y. λ z. x z (y z)
/// ```
pub type Ss<X, Y, Z> = <<<S as Λ>::A<X> as Λ>::A<Y> as Λ>::A<Z>;
pub use s_private::S;
mod s_private {
    use super::*;
    /// Substitution
    /// ```text
    /// λ x. λ y. λ z. x z (y z)
    /// ```
    #[derive(Debug, Default, Clone, Copy)]
    pub struct S;
    #[derive(Debug, Default, Clone, Copy)]
    pub struct SS<X>(X);
    #[derive(Debug, Default, Clone, Copy)]
    pub struct SSS<X, Y>(X, Y);
    impl Λ for S {
        type Type = ();
        type A<X: Λ> = SS<X>;
    }
    impl<X: Λ> Λ for SS<X> {
        type Type = ();
        type A<Y: Λ> = SSS<X, Y>;
    }
    impl<X: Λ, Y: Λ> Λ for SSS<X, Y> {
        type Type = ();
        type A<Z: Λ> = <X::A<Z> as Λ>::A<Y::A<Z>>;
    }
}
pub use Id as I;
pub use True as K;

/// Fixed point | `Z` combinator
/// ```text
/// λ f. (λ x. f (λ v. x x v)) (λ x. f (λ v. x x v))
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct Fix;
impl Λ for Fix {
    type Type = ();
    type A<F: Λ> = <fix_private::ZF<F> as Λ>::A<fix_private::ZF<F>>;
}
mod fix_private {
    use super::*;
    #[derive(Debug, Default, Clone, Copy)]
    pub struct ZF<F>(F);
    impl<F: Λ> Λ for ZF<F> {
        type Type = ();
        type A<X: Λ> = F::A<ZX<X>>;
    }
    #[derive(Debug, Default, Clone, Copy)]
    pub struct ZX<X>(X);
    impl<X: Λ> Λ for ZX<X> {
        type Type = ();
        type A<V: Λ> = <X::A<X> as Λ>::A<V>;
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    #[test]
    fn foo() {
        let a: Count<Sub<Five, Two>> = Default::default();
        std::println!("{}", a.value());
        assert_eq!(a.value(), 3);

        let b: IsEq<Five, Five> = Default::default();
        assert!(matches!(b, True));

        let c: Count<Mul<Three, Three>> = Default::default();
        std::println!("{}", c.value());
    }
}
