<div align="center">

<img width="200" src="https://raw.githubusercontent.com/dev-bio/Ekko/master/media/ekko.png" alt="Ekko - Echo Request Utility"/>

__Echo Request Utility__

[![dependency status](https://deps.rs/crate/ekko/0.3.0/status.svg)](https://deps.rs/crate/ekko/0.3.0)
[![Documentation](https://docs.rs/ekko/badge.svg)](https://docs.rs/ekko)
[![License](https://img.shields.io/crates/l/ekko.svg)](https://choosealicense.com/licenses/mit/)

</div>

---

Ekko is a simple and light utility for sending echo requests, giving you (mostly) everything you need.

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
    
    for hops in 1..32 {
        let response = ping.send(hops)?;
 
        match response {
            EkkoResponse::DestinationResponse(data) => {
                println!("DestinationResponse: {:#?}", data);
                break
            }
            
            EkkoResponse::UnreachableResponse((data, reason)) => {
                println!("UnreachableResponse: {:#?} | {:#?}", data, reason);
                continue
            }
            
            EkkoResponse::UnexpectedResponse((data, (t, c))) => {
                println!("UnexpectedResponse: ({}, {}), {:#?}", t, c, data);
                continue
            }
            
            EkkoResponse::ExceededResponse(data) => {
                println!("ExceededResponse: {:#?}", data);
                continue
            }
            
            EkkoResponse::LackingResponse(data) => {
                println!("LackingResponse: {:#?}", data);
                continue
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
