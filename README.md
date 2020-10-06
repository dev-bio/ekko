<div align="center">

# Ekko
__Echo Request Utility__

<p>

[![Rust](https://github.com/dev-bio/Ekko/workflows/Rust/badge.svg)](https://crates.io/crates/ekko)
[![Documentation](https://docs.rs/ekko/badge.svg)](https://docs.rs/ekko)
[![License](https://img.shields.io/crates/l/ekko.svg)](https://choosealicense.com/licenses/mit/)

</p>
</div>

---

Ekko is a simple utility for sending echo requests, giving you (mostly) everything you need. The project is currently at a <u>very</u> early stage so things may be broken or behave unexpectedly!

## Usage
To use `ekko`, add this to your `Cargo.toml`:

```toml
[dependencies]
ekko = "0.1.1"
```

## Example
The following example will trace the route to the specified destination.
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
