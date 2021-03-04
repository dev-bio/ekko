#![deny(unsafe_code)]

//! Ekko is a simple utility for sending echo requests, giving you (mostly) everything you need.
//!
//! ## Example
//! ```rust,no_run
//! use ekko::{ error::{EkkoError},
//!     EkkoResponse,
//!     Ekko,
//! };
//! 
//! fn main() -> Result<(), EkkoError> {
//!     let mut ping = Ekko::with_target("rustup.rs")?;
//! 
//!     // Send single ..
//!     for hop in 0..64 {
//!         match ping.send(hop)? {
//! 
//!             EkkoResponse::Destination(data) => {
//!                 println!("{:?}", EkkoResponse::Destination(data));
//!                 break
//!             }
//! 
//!             x => println!("{:?}", x)
//!         }
//!     }
//! 
//!     // Send batch ..
//!     for response in ping.trace(0..64)? {
//!         match response {
//! 
//!             EkkoResponse::Destination(data) => {
//!                 println!("{:?}", EkkoResponse::Destination(data));
//!                 break
//!             }
//! 
//!             x => println!("{:?}", x)
//!         }
//!     }
//! 
//!     Ok(())
//! }
//! ```

mod responses;
mod packets;
mod sender;

pub use sender::{Ekko};
pub mod error;

pub use responses::{

    UnreachableCodeV6,
    UnreachableCodeV4,
    Unreachable,

    ParameterProblemV6,
    ParameterProblemV4,
    ParameterProblem,

    Redirect,

    EkkoResponse,
    EkkoData,
};
