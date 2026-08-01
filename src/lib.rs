#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
use std::{error::Error, future::Future, marker::PhantomData, pin::Pin, rc::Rc, result::Result};

/// A future that resolves to a `Out` type.
pub type HookFuture<'a, R> = Pin<Box<dyn Future<Output = R> + 'a>>;

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
///     type Result = String;
///     
///     async fn call(&self, input: String) -> Self::Result {
///         input
///     }
/// }
/// ```
pub trait Hook<In, Out> {
    /// The expected result type
    type Result;
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

impl<H, In, Out> Hook<In, Out> for Rc<H>
where
    H: Hook<In, Out>,
{
    type Result = H::Result;

    async fn call(&self, input: In) -> Self::Result {
        self.as_ref().call(input).await
    }
}

/// Trait for chaining hooks
pub trait Chain<H, In, Out> {
    /// The type of the hook to pull
    type Hook: Hook<In, Out>;

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
            inner: Rc::new(inner),
            closure,
        }
    }
}

impl<F, Fut, H, In, Out> Hook<In, Out> for HookFn<F, H>
where
    F: Fn(In, Rc<H>) -> Fut,
    Fut: Future<Output = Result<Out, Box<dyn Error>>>,
    H: Hook<In, Out>,
{
    type Result = Result<Out, Box<dyn Error>>;

    async fn call(&self, input: In) -> Self::Result {
        let inner = Rc::clone(&self.inner);
        (self.closure)(input, inner).await
    }
}

/// Client struct
pub struct Client<H, In, Out> {
    hook: H,
    _marker: PhantomData<(In, Out)>,
}

impl<H, In, Out> Client<H, In, Out>
where
    H: Hook<In, Out> + 'static,
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
    ///     type Result = String;
    ///     
    ///     async fn call(&self, input: String) -> Self::Result {
    ///         input
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
    pub fn chain<C, Hout>(self, chain: C) -> Client<Hout, In, Out>
    where
        C: Chain<H, In, Out, Hook = Hout>,
        Hout: Hook<In, Out> + 'static,
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
    ///     output
    /// });
    /// ```
    /// # Notes
    /// 
    /// Keep in mind the last hook added to the chain will
    /// be the first to be called
    pub fn chain_fn<F, Fut>(self, f: F) -> Client<HookFn<F, H>, In, Out>
    where
        F: Fn(In, std::rc::Rc<H>) -> Fut + Send,
        Fut: Future<Output = Result<Out, Box<dyn Error>>>,
    {
        Client::from_fn(HookFn::new(self.hook, f))
    }
}

impl<F, H, In, Out> Client<HookFn<F, H>, In, Out> {
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
