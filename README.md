<div align="center">

<a href="https://crates.io/crates/ekko">
<img width="200" src="https://raw.githubusercontent.com/dev-bio/Ekko/master/media/ekko.svg" alt="Ekko - Echo Request Utility"/>
</a>

__Echo Request Utility__

[![dependency status](https://deps.rs/crate/ekko/0.5.0/status.svg)](https://deps.rs/crate/ekko/0.5.0)
[![Documentation](https://docs.rs/ekko/badge.svg)](https://docs.rs/ekko)
[![License](https://img.shields.io/crates/l/ekko.svg)](https://choosealicense.com/licenses/mit/)

</div>

---

Ekko is a simple and light utility for sending echo requests synchronously, built upon raw sockets; currently in its early stages with little to no test coverage.

## Usage
To use `ekko`, add this to your `Cargo.toml`:

```toml
[dependencies]
ekko = "0.5.0"
```

## Example
The following example will trace the route to the specified destination.
```rust
use ekko::{ error::{EkkoError},
    EkkoResponse,
    Ekko,
};

fn main() -> Result<(), EkkoError> {
    if let Some(destination) = "8.8.8.8:0".to_socket_addrs()?.last() {
        let mut sender = Ekko::with_target(destination)?;

        for hops in 0..64 {
            let responses = sender.send_range(0..(hops))?;
            for ekko in responses.iter() {
                match ekko {

                    EkkoResponse::Destination(_) => {

                        for ekko in responses.iter() {
                            println!("{:?}", ekko)
                        }
        
                        return Ok(()) 
                    }

                    _ => continue
                }
            }
        }
    }

    Ok(())
}
```

## Contributing
All contributions are welcome, don't hesitate to open an issue if something is missing!

## License
[MIT](https://choosealicense.com/licenses/mit/)
