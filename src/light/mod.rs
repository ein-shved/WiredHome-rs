use crate::event::{Event, Handler};
use embassy_stm32::gpio::{Level, Output, Pin};
use embassy_stm32::pwm::simple_pwm::SimplePwm;
use embassy_stm32::pwm::{CaptureCompare16bitInstance, Channel};
use embassy_time::{Duration, Timer};

pub struct Led<'d, T: Pin> {
    output: Output<'d, T>,
}

impl<'d, T: Pin> From<Output<'d, T>> for Led<'d, T> {
    fn from(output: Output<'d, T>) -> Self {
        Self { output }
    }
}

impl<'d, T: Pin> Handler for Led<'d, T> {
    type Data = bool;
    async fn handle(&mut self, on: Self::Data) {
        self.output
            .set_level(if on { Level::Low } else { Level::High })
    }
}

pub struct PwmLed<'d, T>
where
    T: CaptureCompare16bitInstance,
{
    pwm: SimplePwm<'d, T>,
    warming: Duration,
    duty: u16,
}

impl<'d, T> PwmLed<'d, T>
where
    T: CaptureCompare16bitInstance,
{
    pub fn new(pwm: SimplePwm<'d, T>, warming: Duration) -> Self {
        Self {
            pwm,
            warming,
            duty: 0,
        }
    }
}

impl<'d, T> Handler for PwmLed<'d, T>
where
    T: CaptureCompare16bitInstance,
{
    type Data = (bool, Channel);
    async fn handle(&mut self, on: Self::Data) {
        let max_duty = self.pwm.get_max_duty();
        let limit = if on.0 { max_duty } else { 0 };
        let dur = self.warming / max_duty.into();
        while self.duty != limit {
            Timer::after(dur).await;
            if on.0 {
                self.duty += 1;
            } else {
                self.duty -= 1;
            }
            self.pwm.set_duty(on.1, self.duty);
        }
    }
}

pub struct SinglePwmLedInput<Ev: Event> {
    event: Ev,
    ch: Channel,
}

impl<Ev, EvData> Event for SinglePwmLedInput<Ev>
where
    Ev: Event<Data = EvData>,
    EvData: Into<bool>,
{
    type Data = (bool, Channel);
    async fn next(&mut self) -> Self::Data {
        (self.event.next().await.into(), self.ch)
    }
}

impl<Ev> SinglePwmLedInput<Ev>
where
    Ev: Event,
{
    pub fn new(event: Ev, ch: Channel) -> Self {
        Self { event, ch }
    }
}
