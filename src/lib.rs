//! Ekko aims to be a light utility for sending echo requests; currently in its early stages.
//!
//! ## Example
//! ```rust,no_run
//! use ekko::{ 
//! 
//!     EkkoResponse,
//!     EkkoError,
//!     Ekko,
//! };
//! 
//! fn main() -> Result<(), EkkoError> {
//!     let sender = Ekko::with_target([8, 8, 8, 8])?;
//! 
//!     for hops in 0..32 {
//!         let responses = sender.send_range(0..hops)?;
//!         for ekko in responses.iter() {
//!             match ekko {
//! 
//!                 EkkoResponse::Destination(_) => {
//!                     for ekko in responses.iter() {
//!                         println!("{ekko:?}")
//!                     }
//!     
//!                     return Ok(()) 
//!                 }
//! 
//!                 ekko => continue
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
mod error;

pub use error::{EkkoError};

pub use sender::{

    EkkoSettings,
    Ekko,
};

pub use responses::{

    UnreachableCodeV6,
    UnreachableCodeV4,
    Unreachable,
    Redirect,

    EkkoResponse,
    EkkoData,
};
