# Ekko
Ekko is a simple utility for sending echo requests, giving you (mostly) everything you need. The project is currently at a <u>very</u> early stage so things may be broken or behave unexpectedly!

![Rust](https://github.com/dev-bio/Ekko/workflows/Rust/badge.svg)

## Installation
To use `ekko`, add this to your `Cargo.toml`:

```toml
[dependencies]
ekko = "0.1.1"
```

## Usage
The following code will trace the route to the specified destination.
```rust
use ekko::{ error::{EkkoError},
    EkkoResponse,
    Ekko,
};

fn main() -> Result<(), EkkoError> {
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
