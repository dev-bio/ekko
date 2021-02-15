<div align="center">

<img width="200" src="https://raw.githubusercontent.com/dev-bio/Ekko/master/media/ekko.svg" alt="Ekko - Echo Request Utility"/>

__Echo Request Utility__

[![dependency status](https://deps.rs/crate/ekko/0.3.0/status.svg)](https://deps.rs/crate/ekko/0.3.0)
[![Documentation](https://docs.rs/ekko/badge.svg)](https://docs.rs/ekko)
[![License](https://img.shields.io/crates/l/ekko.svg)](https://choosealicense.com/licenses/mit/)

</div>

---

Ekko is a simple and light utility for sending echo requests synchronously, built upon raw sockets; currently in its early stages with little to no coverage.

## Usage
To use `ekko`, add this to your `Cargo.toml`:

```toml
[dependencies]
ekko = "0.3.0"
```

## Example
The following example will trace the route to the specified destination.
```rust
use ekko::{ error::{EkkoError},
    EkkoResponse,
    Ekko,
};

fn main() -> Result<(), EkkoError> {
    let mut ping = Ekko::with_target("rustup.rs")?;

    for hops in 0..64 {
        let response = ping.send(hops)?;

        match response {
            EkkoResponse::Destination(data) => {
                println!("DestinationResponse: {:?}", data);
                break
            }

            EkkoResponse::Unreachable((data, reason)) => {
                println!("UnreachableResponse: {:?} | {:?}", data, reason);
                continue
            }

            EkkoResponse::Unexpected((data, (t, c))) => {
                println!("UnexpectedResponse: ({}, {}), {:?}", t, c, data);
                continue
            }

            EkkoResponse::Exceeded(data) => {
                println!("ExceededResponse: {:?}", data);
                continue
            }

            EkkoResponse::Lacking(data) => {
                println!("LackingResponse: {:?}", data);
                continue
            }

            _ => continue
        }
    }

    Ok(())
}
```

## Contributing
All contributions are welcome, don't hesitate to open an issue if something is missing!

## License
[MIT](https://choosealicense.com/licenses/mit/)
