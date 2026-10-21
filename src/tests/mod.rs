#[allow(unused)]
use std::error::Error as _;

use crate::{Chain, Client, Hook, HookError};

struct Smartty {
    name: String,
}

impl Hook<String, String> for Smartty {
    type Result = Result<String, HookError>;
    type Error = HookError;

    async fn call(&self, input: String) -> Self::Result {
        Ok(format!("{} {}", self.name, input))
    }
}

struct MyHook<H> {
    inner: H,
}

impl<H> MyHook<H> {
    fn new(inner: H) -> Self {
        Self { inner }
    }
}

impl<H> Hook<String, String> for MyHook<H>
where
    H: Hook<String, String, Result = Result<String, HookError>>,
{
    type Result = Result<String, HookError>;
    type Error = HookError;

    async fn call(&self, input: String) -> Self::Result {
        self.inner.call(input).await
    }
}

struct OhMy;

impl<H> Chain<H, HookError, String, String> for OhMy
where
    H: Hook<String, String, Result = Result<String, HookError>>,
{
    type Hook = MyHook<H>;

    fn chain(&self, hook: H) -> Self::Hook {
        MyHook::new(hook)
    }
}

#[tokio::test]
async fn test_inline_hook() -> Result<(), HookError> {
    let smart = Smartty {
        name: "Mark".to_string(),
    };

    let client = Client::<Smartty, String, String>::new(smart).chain_fn(|req, next| async move {
        let output = next.call(req).await?;
        Ok(format!("{} {}", output, "Holmes"))
    });

    let output = client.hook.call("Hello".to_string()).await?;

    assert_eq!(output, "Mark Hello Holmes");

    Ok(())
}

#[tokio::test]
async fn test_chained_hook() -> Result<(), HookError> {
    let smart = Smartty {
        name: "Mark".to_string(),
    };

    let client = Client::<Smartty, String, String>::new(smart)
        .chain_fn(|req, next| async move {
            let output = next.call(req).await?;
            Ok(format!("{} {}", output, "Holmes"))
        })
        .chain_fn(|req, next| async move {
            let output = next.call(req).await?;
            Ok(format!("{} {}", output, "Junior"))
        });

    let output = client.hook.call("Hello".to_string()).await?;

    assert_eq!(output, "Mark Hello Holmes Junior");

    Ok(())
}

#[tokio::test]
async fn test_hook_impl() -> Result<(), HookError> {
    let smart = Smartty {
        name: "Mark".to_string(),
    };

    let client = Client::<Smartty, String, String>::new(smart)
        .chain_fn(|req, next| async move {
            let output = next.call(req).await?;
            Ok(format!("{} {}", output, "Holmes"))
        })
        .chain(OhMy);

    let output = client.hook.call("Hello".to_string()).await?;

    assert_eq!(output, "Mark Hello Holmes");

    Ok(())
}

#[test]
fn test_error() {
    let error = HookError::new("test".to_string());
    assert_eq!(error.to_string(), "test");
}

#[test]
fn test_error_display() {
    let error = HookError::new("test".to_string());
    assert_eq!(format!("{}", error), "test");
}

#[test]
fn test_error_debug() {
    let error = HookError::new("test".to_string());
    assert_eq!(format!("{:?}", error), "HookError { message: \"test\" }");
}
