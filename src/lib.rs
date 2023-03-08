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
    /* Holds the overflow 32 bit count to extend the RTC to 64 bit */
    overflow: u32,
    /* This is not a T type like nRF because we only have one 
    RTC peripheral for this device */
    rtc: Rtc
}
/* RTC based timer */
impl MonoRtc {
    /* we are expecting the user to provide us a RTC instance */
    pub fn new(mut rtc: Rtc) -> Self {
        rtc.disable();
        rtc.set_scale(0);
        rtc.set_rtccmp(0);
        rtc.set_rtc(0);

        MonoRtc { overflow: 0, rtc: rtc }
    }
    /* Check the rtccmpip interrupt bit for overflows */
    pub fn is_overflow(&self) -> bool {
        self.rtc.rtc_hi() > 0
    }

    pub fn wait_for_overflow(&mut self) {
        self.rtc.set_scale(1);
        /* Wait until we get  */
        self.rtc.set_rtccmp(0x8000);
    }

    pub fn fix_overflow(&mut self) {
        if self.is_overflow() {
            self.overflow += self.rtc.rtc_hi();
            self.rtc.set_rtc_hi(0);
        }
    }
}

impl Monotonic for MonoRtc {
    /* Asocciated types and constants */
    /* base system clock is 32_768 */
    type Instant = fugit::TimerInstantU64<32_768>;
    type Duration = fugit::TimerDurationU64<32_768>;
    const DISABLE_INTERRUPT_ON_EMPTY_QUEUE: bool = false;
    /* End associated */

    fn now(&mut self) -> Self::Instant {
        self.fix_overflow();
        /* Only read lower part of the RTC */
        let count: u64 = self.rtc.rtc_lo().into();
        /* Add the current lower part of the RTC to the 
        accumulated overflow */
        let now: u64 = count + ((self.overflow as u64) << 32);
        Self::Instant::from_ticks(now)
    }

    fn clear_compare_flag(&mut self) {
        self.fix_overflow();
        /* Move the scale one bit to the left to account for the overflow */
        self.wait_for_overflow();
    }

    fn disable_timer(&mut self) {
        self.rtc.disable();
    }

    fn enable_timer(&mut self) {
        self.rtc.enable();
    }
    /* Perform housekeeping for overflow situations */
    fn on_interrupt(&mut self) {
        self.fix_overflow();
    }

    unsafe fn reset(&mut self) {
        self.rtc.set_rtc(0);
        self.clear_compare_flag();
        self.overflow = 0;
    }

    fn set_compare(&mut self, instant: Self::Instant) {
        // REVIEW: checking akin to nRF implementation (do we need it?)
        assert!(self.now() <= instant);
        if (instant.ticks() >> 32) as u32 > self.overflow {
            self.wait_for_overflow();
        } else {
            self.rtc.set_scale(0);
            /* Only take lower 32 bits */
            self.rtc.set_rtccmp(instant.ticks() as u32);
        }
    }

    fn zero() -> Self::Instant {
        Self::Instant::from_ticks(0)
    }
}