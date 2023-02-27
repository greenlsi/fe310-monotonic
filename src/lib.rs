//! This is an implementation of Monotonic trait for fe310x 
//! family of microcontrollers.
//! Used in RTIC https://rtic.rs
#![no_std]

pub use fugit::{self, ExtU64};
use e310x_hal::{rtc::Rtc};
use rtic_monotonic::Monotonic;
/*
* Monotonic clock implementation using RTC peripheral
* Adapted from the nRF implementation
*/
pub struct MonoRtc {
    /* Holds the overflow interrupt bit from rtccmpip*/
    pending: u8, // FIXME: is this the correct bit?
    /* This is not a T type like nRF because we only have one 
    RTC peripheral for this device */
    rtc: Rtc
}
/* RTC based timer */
impl MonoRtc {
    /* we are expecting the user to provide us a RTC instance */
    pub fn new(mut rtc: Rtc) -> Self {
        rtc.enable();
        rtc.set_scale(0);
        rtc.set_rtc(0);
        rtc.enable();

        MonoRtc { pending: 0, rtc: rtc }
    }
    /* Check the rtccmpip interrupt bit for overflows */
    pub fn is_pending(&self) -> bool {
        self.rtc.is_pending()
    }
}

impl Monotonic for MonoRtc {
    /* Asocciated types and constants */
    /* base system clock is 32_768 */
    type Instant = fugit::TimerInstantU32<32_768>;
    type Duration = fugit::TimerDurationU32<32_768>;
    const DISABLE_INTERRUPT_ON_EMPTY_QUEUE: bool = false;
    /* End associated */

    fn now(&mut self) -> Self::Instant {
        /* Only read lower part of the RTC */
        let rtc_lo = self.rtc.rtc_lo();
        Self::Instant::from_ticks(rtc_lo)
    }

    fn clear_compare_flag(&mut self) {

    }
    fn disable_timer(&mut self) {
        
    }
    fn enable_timer(&mut self) {
        
    }
    fn on_interrupt(&mut self) {
        
    }
    unsafe fn reset(&mut self) {
        
    }
    fn set_compare(&mut self, instant: Self::Instant) {
        
    }
    fn zero() -> Self::Instant {
        // TEMPORARY
        Self::Instant::from_ticks(0)
    }
}