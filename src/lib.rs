#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
use std::{
    error::Error,
    fmt::{Debug, Display},
    future::Future,
    marker::PhantomData,
    pin::Pin,
    rc::Rc,
    result::Result,
};

/// A future that resolves to a `Out` type.
pub type HookFuture<'a, R> = Pin<Box<dyn Future<Output = R> + 'a>>;

/// Type alias for a hook that can be chained
pub type NextHook<H> = Rc<H>;

#[derive(Debug)]
/// Error type for hooks
pub struct HookError {
    message: String,
}

impl HookError {
    /// Create a new hook error
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for HookError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

#[cfg(test)]
mod tests;

/// Trait for implementing hooks in a client lib.
///
/// This trait is used to implement hooks that can be chained together to form a pipeline.
///
/// # Example
///
/// ```
/// use tackle::Hook;
///
/// struct MyHook;
///
/// impl Hook<String, String> for MyHook {
///     type Result = Result<String, HookError>;
///     type Error = HookError;
///
///     async fn call(&self, input: String) -> Self::Result {
///         Ok(input)
///     }
/// }
/// ```
pub trait Hook<In, Out> {
    /// The expected result type
    type Result;

    /// The error type
    type Error;

    /// Call the hook with the given request.
    ///
    /// # Arguments
    ///
    /// * `input` - The request to process.
    ///
    /// # Returns
    ///
    /// * `impl Future<Output = Self::Result>` - The response to the request.
    fn call(&self, input: In) -> impl Future<Output = Self::Result>;
}

impl<H, In, Out, E> Hook<In, Out> for Rc<H>
where
    H: Hook<In, Out, Result = Result<Out, E>>,
    E: Error,
{
    type Result = H::Result;
    type Error = H::Error;

    async fn call(&self, input: In) -> Self::Result {
        self.as_ref().call(input).await
    }
}

/// Trait for chaining hooks
pub trait Chain<H, E, In, Out> {
    /// The type of the hook to pull
    type Hook: Hook<In, Out, Result = Result<Out, E>>;

    /// Pull a hook from the builder
    fn chain(&self, hook: H) -> Self::Hook;
}

/// Wrapper for hook function
///
/// # Type Parameters
///
/// * `H` - The type of the hook to pull
pub struct HookFn<F, H> {
    inner: Rc<H>,
    closure: F,
}

impl<F, H> HookFn<F, H> {
    /// Create a new wrapper from next hook and closure
    ///
    /// # Arguments
    ///
    /// * `inner` - The next hook to call
    /// * `closure` - The closure to call
    ///
    /// # Returns
    ///
    /// * `Self` - The new hook
    pub fn new(inner: H, closure: F) -> Self {
        Self {
            inner: inner.into(),
            closure,
        }
    }
}

impl<F, Fut, H, In, Out, E> Hook<In, Out> for HookFn<F, H>
where
    F: Fn(In, NextHook<H>) -> Fut,
    Fut: Future<Output = Result<Out, E>>,
    H: Hook<In, Out, Result = Result<Out, E>>,
    E: Error,
{
    type Result = Result<Out, E>;
    type Error = E;

    async fn call(&self, input: In) -> Self::Result {
        let inner = self.inner.clone();
        (self.closure)(input, inner).await
    }
}

/// Client struct
pub struct Client<H, In, Out, E = HookError> {
    hook: H,
    _marker: PhantomData<(In, Out, E)>,
}

impl<H, E, In, Out> Client<H, In, Out, E>
where
    H: Hook<In, Out, Result = Result<Out, E>> + 'static,
    E: Error,
{
    /// Initialize a client from hook
    pub fn new(inner: H) -> Self {
        Self {
            hook: inner,
            _marker: PhantomData,
        }
    }

    /// Add a new hook to the chain
    ///
    /// # Arguments
    ///
    /// * `chain` - The chain to add
    ///
    /// # Returns
    ///
    /// * `Client<Hout, In, Out>` - The new client
    ///
    /// # Example
    ///
    /// ```
    /// use tackle::Client;
    ///
    /// struct MyHook;
    ///
    /// impl tackle::Hook<String, String> for MyHook {
    ///     type Result = Result<String, HookError>;
    ///     type Error = HookError;
    ///
    ///     async fn call(&self, input: String) -> Self::Result {
    ///         Ok(input)
    ///     }
    /// }
    ///
    /// let client = Client::new(InnerClient).chain(MyHook);
    /// ```
    ///
    /// # Notes
    ///
    /// Keep in mind the last hook added to the chain will
    /// be the first to be called
    pub fn chain<C, Hout>(self, chain: C) -> Client<Hout, In, Out, E>
    where
        C: Chain<H, E, In, Out, Hook = Hout>,
        E: Error,
        Hout: Hook<In, Out, Result = Result<Out, E>> + 'static,
    {
        Client::new(chain.chain(self.hook))
    }

    /// Add a hook from a function
    ///
    /// # Arguments
    ///
    /// * `f` - The function to call
    ///
    /// # Returns
    ///
    /// * `Client<HookFn<F, H>, In, Out>` - The new client
    ///
    /// # Example
    ///
    /// ```
    /// use tackle::Client;
    ///
    /// // The target struct which will do some processing at the
    /// // end of hook chain
    /// struct InnerClient;
    ///
    /// let client = Client::new(InnerClient).chain_fn(|input, next| async move {
    ///     // Do something with input
    ///     let output = next.call(input).await
    ///     // Do something with output
    ///     Ok(output)
    /// });
    /// ```
    /// # Notes
    ///
    /// Keep in mind the last hook added to the chain will
    /// be the first to be called
    pub fn chain_fn<F, Fut>(self, f: F) -> Client<HookFn<F, H>, In, Out, E>
    where
        F: Fn(In, NextHook<H>) -> Fut + Send,
        E: Error,
        Fut: Future<Output = Result<Out, E>>,
    {
        Client::from_fn(HookFn::new(self.hook, f))
    }
}

impl<F, H, E, In, Out> Client<HookFn<F, H>, E, In, Out> {
    /// Initialize a client from hook
    ///
    /// # Arguments
    ///
    /// * `inner` - The hook to use
    ///
    /// # Returns
    ///
    /// * `Self` - The new client
    pub fn from_fn(inner: HookFn<F, H>) -> Self {
        Self {
            hook: inner,
            _marker: PhantomData,
        }
    }
}
