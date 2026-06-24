//！ Metaprogramming Utility

pub use crate::lambda::*;

/// Conditional selection type
pub type Select<C, T, F> = <If<C, Type<T>, Type<F>> as Λ>::Type;

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    #[test]
    fn foo() {
        let a: Select<IsEq<Ten, Mul<Two, Three>>, u8, u16> = Default::default();
        std::println!("{}", std::any::type_name_of_val(&a));

        let a: Select<IsEq<Six, Mul<Two, Three>>, u8, u16> = Default::default();
        std::println!("{}", std::any::type_name_of_val(&a));
    }
}
