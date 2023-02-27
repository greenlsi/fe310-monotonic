//! This is an implementation of Monotonic trait for fe310x 
//! family of microcontrollers.
//! Used in RTIC https://rtic.rs
#![no_std]

pub use fugit::{self, ExtU64};
use e310x_hal::{*, core::clint::Clint};
use rtic_monotonic::Monotonic;
/*
* Monotonic clock implementation using CLINT peripheral to
* control the mtime register 
*/
pub struct MonoClint;

impl MonoClint {
    pub fn new(freq: Hertz, clint: Clint) -> Self {
        
    }
}
