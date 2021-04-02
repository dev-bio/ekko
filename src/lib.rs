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
//!     if let Some(destination) = "8.8.8.8:0".to_socket_addrs()?.last() {
//!         let mut sender = Ekko::with_target(destination)?;
//! 
//!         for hops in 0..64 {
//!             let responses = sender.send_range(0..(hops))?;
//!             for ekko in responses.iter() {
//!                 match ekko {
//! 
//!                     EkkoResponse::Destination(_) => {
//! 
//!                         for ekko in responses.iter() {
//!                             println!("{:?}", ekko)
//!                         }
//!         
//!                         return Ok(()) 
//!                     }
//! 
//!                     _ => continue
//!                 }
//!             }
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
