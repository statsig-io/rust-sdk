# Statsig for Rust

[![Test](https://github.com/statsig-io/rust-sdk/actions/workflows/kong.yml/badge.svg)](https://github.com/statsig-io/rust-sdk/actions/workflows/kong.yml)

```rust
use statsig::{Statsig, StatsigUser};
use tokio;

#[tokio::main]
async fn main() {
    Statsig::initialize("secret-key").await;

    let user = StatsigUser::with_user_id("a-user".to_string());

    let passes_gate =  Statsig::check_gate(&user, "a_gate").ok().unwrap_or(false);
    if passes_gate {
        // Show cool new feature
    }
}
```
