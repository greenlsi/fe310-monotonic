//! Implementations of the RTIC Monotonic trait for the RTC peripheral.

use e310x_hal::rtc::Rtc;
use fugit;
use rtic_monotonic::Monotonic;

/// 64-bit monotonic clock implementation for the RTC peripheral
pub struct MonoRtc64 {
    /// Holds the overflow 32 bit count to extend the RTC to 64 bit
    overflow: u32,
    /// RTC peripheral
    rtc: Rtc,
}

impl MonoRtc64 {
    /// we are expecting the user to provide us a RTC instance
    pub fn new(mut rtc: Rtc) -> Self {
        rtc.disable();
        rtc.set_scale(0);
        rtc.set_rtccmp(0);
        rtc.set_rtc(0);

        Self { overflow: 0, rtc }
    }

    /// We use RTCHI as an overflow indicator
    pub fn is_overflow(&self) -> bool {
        self.rtc.rtc_hi() > 0
    }

    /// Sets RTCMP for triggering an interrupt when the RTC overflows
    pub fn wait_for_overflow(&mut self) {
        self.rtc.set_scale(1);
        self.rtc.set_rtccmp(0x8000);
    }

    /// Clears RTCHI and add its previous values to overflow
    pub fn fix_overflow(&mut self) {
        if self.is_overflow() {
            self.overflow += self.rtc.rtc_hi();
            self.rtc.set_rtc_hi(0);
        }
    }
}

impl Monotonic for MonoRtc64 {
    /* base system clock is 32_768 */
    type Instant = fugit::TimerInstantU64<32_768>;
    type Duration = fugit::TimerDurationU64<32_768>;
    const DISABLE_INTERRUPT_ON_EMPTY_QUEUE: bool = false;

    fn now(&mut self) -> Self::Instant {
        self.fix_overflow();
        /* Only read lower part of the RTC */
        let count: u64 = self.rtc.rtc_lo().into();
        /* Add the current lower part of the RTC to the accumulated overflow */
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

    fn on_interrupt(&mut self) {
        self.fix_overflow();
    }

    unsafe fn reset(&mut self) {
        self.rtc.set_rtc(0);
        self.clear_compare_flag();
        self.overflow = 0;
    }

    fn set_compare(&mut self, instant: Self::Instant) {
        assert!(instant >= self.now()); // overflow disalowed in 64-bit timer
        if (instant.ticks() >> 32) as u32 > self.overflow {
            self.wait_for_overflow(); // we still need to wait for one or more overflows
        } else {
            self.rtc.set_scale(0);
            self.rtc.set_rtccmp(instant.ticks() as u32); /* Only take lower 32 bits */
        }
    }

    fn zero() -> Self::Instant {
        Self::Instant::from_ticks(0)
    }
}

/// Alternative implementation for 32-bit timer.
pub struct MonoRtc32(MonoRtc64);

impl MonoRtc32 {
    /// we are expecting the user to provide us a RTC instance
    pub fn new(rtc: Rtc) -> Self {
        Self(MonoRtc64::new(rtc))
    }
}

impl Monotonic for MonoRtc32 {
    /* base system clock is 32_768 */
    type Instant = fugit::TimerInstantU32<32_768>;
    type Duration = fugit::TimerDurationU32<32_768>;
    const DISABLE_INTERRUPT_ON_EMPTY_QUEUE: bool = true;

    fn now(&mut self) -> Self::Instant {
        self.0.fix_overflow();
        Self::Instant::from_ticks(self.0.rtc.rtc_lo())
    }

    fn clear_compare_flag(&mut self) {
        self.0.clear_compare_flag();
    }

    fn disable_timer(&mut self) {
        self.0.disable_timer();
    }

    fn enable_timer(&mut self) {
        self.0.enable_timer();
    }

    fn on_interrupt(&mut self) {
        self.0.on_interrupt();
    }

    unsafe fn reset(&mut self) {
        self.0.reset();
    }

    fn set_compare(&mut self, instant: Self::Instant) {
        if instant >= self.now() {
            self.0.rtc.set_rtc_lo(instant.ticks());
        } else {
            self.0.wait_for_overflow(); // past or overflow
        }
    }

    fn zero() -> Self::Instant {
        Self::Instant::from_ticks(0)
    }
}
