use std::error::Error;

use crate::{Client, Hook};

struct Smartty {
    name: String,
}

impl Hook<String, String> for Smartty {
    type Result = Result<String, Box<dyn Error>>;

    async fn call(&self, input: String) -> Self::Result {
        Ok(format!("{} {}", self.name, input))
    }
}

#[tokio::test]
async fn test_hook() -> Result<(), Box<dyn Error>> {
    let smart = Smartty {
        name: "Mark".to_string(),
    };

    let client = Client::<Smartty, String, String>::new(smart)
      .chain_fn(|req, next| async move {
          let output  = next.call(req).await?;
          Ok(format!("{} {}", output, "Holmes"))
      });

    let output = client.hook.call("Hello".to_string()).await?;

    assert_eq!(output, "Mark Hello Holmes");

    Ok(())
}
