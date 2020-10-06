#![deny(unsafe_code)]

//! Ekko is a simple utility for sending echo requests, giving you (mostly) everything you need.
//!
//! ## Example
//! ```rust,no_run
//! use ekko::{Ekko, EkkoResponse};
//! use anyhow::{Result};
//! 
//! fn main() -> Result<()> {
//!     let mut ping = Ekko::with_target("rustup.rs")?;
//! 
//!     for hops in 1..32 {
//!         let response = ping.send(hops)?;
//! 
//!         match response {
//!             EkkoResponse::DestinationResponse(_) => {
//!                 println!("{:?}", response);
//!                 break
//!             },
//!             _ => {
//!                 println!("{:?}", response);
//!             },
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

pub use responses::{
    EkkoResponse,
    EkkoData,
    Unreachable,
    UnreachableCodeV6,
    UnreachableCodeV4,
};
