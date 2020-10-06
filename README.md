# Ekko
Ekko is a simple utility for sending echo requests, giving you (mostly) everything you need.

![Rust](https://github.com/dev-bio/Ekko/workflows/Rust/badge.svg)

## Installation
To use `ekko`, add this to your `Cargo.toml`:

```toml
[dependencies]
ekko = "0.1.0"
```

## Usage
The following code will trace the route to the specified destination.
```rust
use ekko::{Ekko, EkkoResponse};

fn main() -> Result<()> {
    let mut sender = Ekko::with_target("rustup.rs")?;

    for hops in 1..32 {
        let response = sender.send(hops)?;

        match response {
            EkkoResponse::DestinationResponse(_) => {
                println!("{:?}", response);
                break
            },
            _ => {
                println!("{:?}", response);
            },
        }
    }

    Ok(())
}
```

## Contributing
All contributions are welcome, don't hesitate to open an issue if something is missing!

## License
[MIT](https://choosealicense.com/licenses/mit/)
