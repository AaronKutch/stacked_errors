use alloc::boxed::Box;
use core::{
    any::Any,
    fmt::Display,
    panic::Location,
    slice::{Iter, IterMut},
};

use thin_vec::{thin_vec, ThinVec};

use crate::{ProbablyNotRootCauseError, TimeoutError, UnitError};

/// Trait implemented for all `T: Display + Send + Sync + 'static`
///
/// clever workaround from
/// <https://users.rust-lang.org/t/impossible-to-use-any-combined-with-any-other-trait/85949/5>
/// needed to enable using a type in both `dyn Display` form for displaying and
/// in `dyn Any + Send + Sync` form for later downcasting
pub trait StackableErrorTrait: Display + Any + Send + Sync + 'static {
    // put as underscores since this this a hack implemented for all `T` that we
    // don't want in IDEs
    fn _as_any(&self) -> &(dyn Any + Send + Sync);
    fn _as_any_mut(&mut self) -> &mut (dyn Any + Send + Sync);
    fn _as_display(&self) -> &(dyn Display + Send + Sync);
}

impl<T: Display + Send + Sync + 'static> StackableErrorTrait for T {
    fn _as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }

    fn _as_any_mut(&mut self) -> &mut (dyn Any + Send + Sync) {
        self
    }

    fn _as_display(&self) -> &(dyn Display + Send + Sync) {
        self
    }
}

pub trait StackedErrorDowncast: Sized {
    fn get_err(&self) -> &(impl Display + Send + Sync + 'static);

    fn get_location(&self) -> Option<&'static Location<'static>>;

    // Attempts to downcast to a concrete type.
    //fn downcast<E: Display + Send + Sync + 'static>(self) -> Result<E, Self>;

    fn downcast_ref<E>(&self) -> Option<&E>
    where
        E: Display + Send + Sync + 'static;

    fn downcast_mut<E>(&mut self) -> Option<&mut E>
    where
        E: Display + Send + Sync + 'static;
}

/// NOTE: this type is only public because `impl Trait` in associated types is
/// unstable, only `StackedErrorDowncast` methods are intended to be used on
/// this.
// The specific type that `Error` uses in its stack. NOTE the `error_kind_size`
// should be updated whenever this is changed. pub type ErrorBox = Box<dyn
// Display + Send + Sync + 'static>;
pub struct ErrorItem {
    b: Box<dyn StackableErrorTrait>,
    l: Option<&'static Location<'static>>,
}
// FIXME
//SmallBox<dyn Display + Send + Sync + 'static, S4>;

impl ErrorItem {
    pub fn new<E: Display + Send + Sync + 'static>(
        e: E,
        l: Option<&'static Location<'static>>,
    ) -> Self {
        Self { b: Box::new(e), l }
    }

    // TODO when upcasting is implemented, this would enable `downcast` in
    // `StackedErrorDowncast`
    /*pub fn inner_downcast(self) -> Box<dyn Any + Send + Sync + 'static> {
        self.0 as Box<_>
    }*/
}

impl StackedErrorDowncast for ErrorItem {
    fn get_err(&self) -> &(impl Display + Send + Sync + 'static) {
        &self.b
    }

    fn get_location(&self) -> Option<&'static Location<'static>> {
        self.l
    }

    //fn downcast<E: Display + Send + Sync + 'static>(self) -> Result<E, Self> {
    //    self.0.as_any().
    //}

    fn downcast_ref<E>(&self) -> Option<&E>
    where
        E: Display + Send + Sync + 'static,
    {
        self.b._as_any().downcast_ref()
    }

    fn downcast_mut<E>(&mut self) -> Option<&mut E>
    where
        E: Display + Send + Sync + 'static,
    {
        self.b._as_any_mut().downcast_mut()
    }
}

/// An error struct intended for high level error propogation with programmable
/// backtraces
///
/// For lower level error propogation, you should still use ordinary [Option]
/// and [Result] with domain-specific enums, it is only when using OS-level
/// functions or when multiple domains converge that this is intended to be
/// used. This has an internal stack for different kinds of arbitrary lower
/// level errors and [Location](core::panic::Location)s. When used with the
/// [StackableErr](crate::StackableErr) trait, this enables easy conversion and
/// software defined backtraces for better `async` debugging. See the crate docs
/// for more.
///
/// Note that due to trait conflicts and not wanting users to accidentally
/// wastefully embed `stacked_errors::Error` in a `BoxedErr` of another
/// `stacked_errors::Error`, `stacked_errors::Error` itself does not actually
/// implement [core::error::Error]. This does not pose a problem in most cases
/// since it is intended to be the highest level of error that is directly
/// returned or panicked on. However, if a user needs the end result struct to
/// implement [core::error::Error], they can use the
/// [StackedError](crate::StackedError) wrapper.
pub struct Error {
    /// Using a ThinVec has advantages such as taking as little space as
    /// possible on the stack (since we are commiting to some indirection at
    /// this point), and having the niche optimizations applied to things like
    /// `Result<(), Error>`.
    stack: ThinVec<ErrorItem>,
}

// FIXME rename the above and have `Error` as an alias

/// Wraps around [stacked_errors::Error](crate::Error) to implement
/// [core::error::Error], since [stacked_errors::Error](crate::Error) itself
/// cannot implement the trait.
#[derive(Debug, thiserror::Error)]
pub struct StackedError(pub Error);

/// Note: in most cases you can use `Error::from` or a call from `StackableErr`
/// instead of these functions.
impl Error {
    /// Returns an empty error stack
    pub fn empty() -> Self {
        Self {
            stack: ThinVec::new(),
        }
    }

    /// Returns an error stack with just a `UnitError` and location information
    #[track_caller]
    pub fn new() -> Self {
        Self::from_err(UnitError {})
    }

    #[track_caller]
    pub fn from_err<E: Display + Send + Sync + 'static>(e: E) -> Self {
        Self {
            stack: thin_vec![ErrorItem::new(e, Some(Location::caller()))],
        }
    }

    pub fn from_err_locationless<E: Display + Send + Sync + 'static>(e: E) -> Self {
        Self {
            stack: thin_vec![ErrorItem::new(e, None)],
        }
    }

    /// Returns an error stack with just `kind`. The `impl From<_> for Error`
    /// implementations can usually be used in place of this.
    #[track_caller]
    pub fn stack_err<E: Display + Send + Sync + 'static>(mut self, e: E) -> Self {
        self.stack.push(ErrorItem::new(e, Some(Location::caller())));
        self
    }

    #[track_caller]
    pub(crate) fn stack_err_inner<E: Display + Send + Sync + 'static>(&mut self, e: E) {
        self.stack.push(ErrorItem::new(e, Some(Location::caller())));
    }

    pub(crate) fn stack_err_locationless_inner<E: Display + Send + Sync + 'static>(
        &mut self,
        e: E,
    ) {
        self.stack.push(ErrorItem::new(e, None));
    }

    /// Returns an error stack with just `kind`. The `impl From<_> for Error`
    /// implementations can usually be used in place of this.
    pub fn stack_err_locationless<E: Display + Send + Sync + 'static>(mut self, e: E) -> Self {
        self.stack.push(ErrorItem::new(e, None));
        self
    }

    /// Only pushes `track_caller` location to the stack
    #[track_caller]
    pub fn stack(self) -> Self {
        self.stack_err(UnitError {})
    }

    #[track_caller]
    pub(crate) fn stack_inner(&mut self) {
        self.stack_err_inner(UnitError {})
    }

    pub(crate) fn stack_locationless_inner(&mut self) {
        self.stack_err_locationless_inner(UnitError {})
    }

    /// Moves the stack of `other` onto `self`
    pub fn chain_errors(mut self, mut other: Self) -> Self {
        self.stack.append(&mut other.stack);
        self
    }

    /// Returns a base `TimeoutError` error
    #[track_caller]
    pub fn timeout() -> Self {
        Self::from_err(TimeoutError {})
    }

    /// Returns a base `ProbablyNotRootCauseError` error
    #[track_caller]
    pub fn probably_not_root_cause() -> Self {
        Self::from_err(ProbablyNotRootCauseError {})
    }

    /// Returns if a `TimeoutError` is in the error stack
    pub fn is_timeout(&self) -> bool {
        for e in &self.stack {
            if e.downcast_ref::<TimeoutError>().is_some() {
                return true
            }
        }
        false
    }

    /// Returns if a `ProbablyNotRootCauseError` is in the error stack
    pub fn is_probably_not_root_cause(&self) -> bool {
        for e in &self.stack {
            if e.downcast_ref::<ProbablyNotRootCauseError>().is_some() {
                return true
            }
        }
        false
    }

    /// Iteration over the [StackedErrorDowncast] items of `self`
    pub fn iter(&self) -> Iter<ErrorItem> {
        self.stack.iter()
    }

    /// Mutable iteration over the [StackedErrorDowncast] items of `self`
    pub fn iter_mut(&mut self) -> IterMut<ErrorItem> {
        self.stack.iter_mut()
    }
}

impl<'a> IntoIterator for &'a Error {
    type IntoIter = Iter<'a, ErrorItem>;
    type Item = &'a ErrorItem;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Error {
    type IntoIter = IterMut<'a, ErrorItem>;
    type Item = &'a mut ErrorItem;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl Default for Error {
    #[track_caller]
    fn default() -> Self {
        Error::new()
    }
}
