# Statsig for Rust


> [!CAUTION]
> This SDK is deprecated an will no longer receive support. Please migrate to [statsig-rust](https://github.com/statsig-io/statsig-server-core/tree/main/statsig-rust) [ [Docs](https://docs.statsig.com/server-core/rust-core) ].



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
