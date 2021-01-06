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
//!     for hops in 1..32 {
//!         let response = ping.send(hops)?;
//! 
//!         match response {
//!             EkkoResponse::DestinationResponse(data) => {
//!                 println!("DestinationResponse: {:#?}", data);
//!                 break
//!             }
//! 
//!             EkkoResponse::UnreachableResponse((data, reason)) => {
//!                 println!("UnreachableResponse: {:#?} | {:#?}", data, reason);
//!                 continue
//!             }
//! 
//!             EkkoResponse::UnexpectedResponse((data, (t, c))) => {
//!                 println!("UnexpectedResponse: ({}, {}), {:#?}", t, c, data);
//!                 continue
//!             }
//! 
//!             EkkoResponse::ExceededResponse(data) => {
//!                 println!("ExceededResponse: {:#?}", data);
//!                 continue
//!             }
//! 
//!             EkkoResponse::LackingResponse(data) => {
//!                 println!("LackingResponse: {:#?}", data);
//!                 continue
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

pub mod error;

pub use sender::{Ekko};
pub use responses::{
    EkkoResponse,
    EkkoData,
    Unreachable,
    UnreachableCodeV6,
    UnreachableCodeV4,
};
