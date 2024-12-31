/// Supports conversion to `Err` in situations where no `Ok` value is needed.  This is useful mainly
/// for early return via `?`.
///
/// # Examples
///
/// ```
/// use matching_engine_v3::common::OrErr as _;
///
/// struct DomainError;
///
/// fn sqrt(n: f64) -> Result<f64, DomainError> {
///     (n >= 0.0).or_err(DomainError)?;
///     Ok(n.sqrt())
/// }
/// ```
pub trait OrErr {
    /// # Errors
    ///
    /// May return `Err(err)`.
    fn or_err<E>(self, err: E) -> Result<(), E>;

    /// # Errors
    ///
    /// May return `Err(err())`.
    fn or_else_err<E>(self, err: impl FnOnce() -> E) -> Result<(), E>;
}

impl OrErr for bool {
    /// # Errors
    ///
    /// Will return `Err(err)` if `self` is false.
    fn or_err<E>(self, err: E) -> Result<(), E> {
        self.then_some(()).ok_or(err)
    }

    /// # Errors
    ///
    /// Will return `Err(err())` if `self` is false.
    fn or_else_err<E>(self, err: impl FnOnce() -> E) -> Result<(), E> {
        self.then_some(()).ok_or_else(err)
    }
}
