<div align="center">

<a href="https://crates.io/crates/ekko">
<img width="200" src="https://raw.githubusercontent.com/dev-bio/Ekko/master/media/ekko.svg" alt="Ekko - Echo Request Utility"/>
</a>

__Echo Request Utility__

[![dependency status](https://deps.rs/crate/ekko/0.7.3/status.svg)](https://deps.rs/crate/ekko/0.7.3)
[![Documentation](https://docs.rs/ekko/badge.svg)](https://docs.rs/ekko)
[![License](https://img.shields.io/crates/l/ekko.svg)](https://choosealicense.com/licenses/mit/)

</div>

---

Ekko aims to be a light utility for sending echo requests; currently in its early stages.

## Usage
To use `ekko`, add this to your `Cargo.toml`:

```toml
[dependencies]
ekko = "0.7.3"
```

## Example
The following example will trace the route to the specified destination.
```rust
use ekko::{ 

    EkkoResponse,
    EkkoError,
    Ekko,
};

fn main() -> Result<(), EkkoError> {
    let sender = Ekko::with_target([8, 8, 8, 8])?;

    for hops in 0..32 {
        let responses = sender.send_range(0..hops)?;
        for ekko in responses.iter() {
            match ekko {

                EkkoResponse::Destination(_) => {
                    for ekko in responses.iter() {
                        println!("{ekko:?}")
                    }
    
                    return Ok(()) 
                }

                _ => continue
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
