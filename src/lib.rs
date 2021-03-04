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
//!     for response in ping.trace(0..64)? {
//!         match response {
//!             EkkoResponse::Destination(data) => {
//!                 println!("DestinationResponse: {:?}", data);
//!                 break
//!             }
//! 
//!             EkkoResponse::Unreachable((data, reason)) => {
//!                 println!("UnreachableResponse: {:?} | {:?}", data, reason);
//!                 continue
//!             }
//! 
//!             EkkoResponse::Unexpected((data, (t, c))) => {
//!                 println!("UnexpectedResponse: ({}, {}), {:?}", t, c, data);
//!                 continue
//!             }
//! 
//!             EkkoResponse::Exceeded(data) => {
//!                 println!("ExceededResponse: {:?}", data);
//!                 continue
//!             }
//! 
//!             EkkoResponse::Lacking(data) => {
//!                 println!("LackingResponse: {:?}", data);
//!                 continue
//!             }
//! 
//!             _ => continue
//!         }
//!     }
//! 
//!     Ok(())
//! }
//! ```

mod responses;
mod packets;
mod sender;

pub mod error;

pub use sender::{Ekko};
pub use responses::{
    EkkoResponse,
    EkkoData,
    Unreachable,
    UnreachableCodeV6,
    UnreachableCodeV4,
};
