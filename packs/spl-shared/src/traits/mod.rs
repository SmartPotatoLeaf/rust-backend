/// A trait for converting a type `T` into `Self` using a context `C`.
/// This is similar to `std::convert::From`, but allows for an additional context argument
/// which can be used to pass dependencies or other necessary information for the conversion.
pub trait FromWithContext<T, C>: Sized {
    type Error;

    /// Performs the conversion.
    fn from_with_context(item: T, context: C) -> Result<Self, Self::Error>;
}

/// A trait for converting `Self` into `T` using a context `C`.
/// This is the reciprocal of `FromWithContext`.
/// It is automatically implemented for any type that implements `FromWithContext`.
pub trait IntoWithContext<T, C>: Sized {
    type Error;

    /// Performs the conversion.
    fn into_with_context(self, context: C) -> Result<T, Self::Error>;
}

// Blanket implementation of IntoWithContext for any type that implements FromWithContext
impl<T, U, C> IntoWithContext<U, C> for T
where
    U: FromWithContext<T, C>,
{
    type Error = U::Error;

    fn into_with_context(self, context: C) -> Result<U, Self::Error> {
        U::from_with_context(self, context)
    }
}


