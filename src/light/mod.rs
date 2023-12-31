use crate::debounce::exti_event::PushState;
use crate::event::{Event, Handler};
use bitmask_enum::*;
use embassy_futures::join::join4;
use embassy_stm32::gpio::{Level, Output, Pin};
use embassy_stm32::pwm::simple_pwm::SimplePwm;
use embassy_stm32::pwm::{CaptureCompare16bitInstance, Channel as PwmChannel};
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

#[derive(Default, PartialEq, Clone, Copy)]
pub enum PwmLedEvent {
    OnOf(bool),
    #[default]
    Switched,
    Hold(bool),
}

#[bitmask]
pub enum Channel {
    Ch1,
    Ch2,
    Ch3,
    Ch4,
}

impl From<Channel> for PwmChannel {
    fn from(value: Channel) -> Self {
        let mut res: Option<PwmChannel> = None;

        value.for_each(|ch| {
            if res.is_none() {
                res = Some(ch)
            }
        });
        res.unwrap()
    }
}

impl Channel {
    pub fn for_each<F: FnMut(PwmChannel)>(&self, mut f: F) {
        if self.contains(Channel::Ch1) {
            f(PwmChannel::Ch1);
        }
        if self.contains(Channel::Ch2) {
            f(PwmChannel::Ch2);
        }
        if self.contains(Channel::Ch3) {
            f(PwmChannel::Ch3);
        }
        if self.contains(Channel::Ch4) {
            f(PwmChannel::Ch4);
        }
    }
}

impl<B> From<B> for PwmLedEvent
where
    B: Into<bool>,
{
    fn from(value: B) -> Self {
        PwmLedEvent::OnOf(value.into())
    }
}

impl From<PushState> for PwmLedEvent {
    fn from(value: PushState) -> Self {
        match value {
            PushState::On => PwmLedEvent::Hold(true),
            PushState::Off => PwmLedEvent::Hold(false),
            PushState::Pushed => PwmLedEvent::Switched,
        }
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
enum PwmLedDirection {
    Off,
    #[default]
    On,
}

impl From<bool> for PwmLedDirection {
    fn from(value: bool) -> Self {
        if value {
            PwmLedDirection::On
        } else {
            PwmLedDirection::Off
        }
    }
}

impl From<PwmLedDirection> for bool {
    fn from(value: PwmLedDirection) -> Self {
        value == PwmLedDirection::On
    }
}

#[derive(Default)]
struct PwmLedChannelState {
    duty: u16,
    last_direction: PwmLedDirection,
}

pub struct PwmLed<'d, T>
where
    T: CaptureCompare16bitInstance,
{
    pwm: SimplePwm<'d, T>,
    warming: Duration,
    ch1: PwmLedChannelState,
    ch2: PwmLedChannelState,
    ch3: PwmLedChannelState,
    ch4: PwmLedChannelState,
}

impl<'d, T> PwmLed<'d, T>
where
    T: CaptureCompare16bitInstance,
{
    pub fn new(pwm: SimplePwm<'d, T>, warming: Duration) -> Self {
        Self {
            pwm,
            warming,
            ch1: PwmLedChannelState::default(),
            ch2: PwmLedChannelState::default(),
            ch3: PwmLedChannelState::default(),
            ch4: PwmLedChannelState::default(),
        }
    }

    fn upd_duty(&mut self, n: i16, ch: PwmChannel) {
        let max_duty = self.pwm.get_max_duty();
        let sch = self.get_channel(ch);
        let mut duty = sch.duty;
        if n >= 0 {
            let mut n = n as u16;
            if n > max_duty {
                n = max_duty;
            }
            if duty >= max_duty - n {
                duty = max_duty - 1;
            } else {
                duty += n as u16;
            }
        } else {
            let n = -n as u16;
            if duty < n {
                duty = 0;
            } else {
                duty -= n;
            }
        }
        sch.duty = duty;
        self.pwm.set_duty(ch.into(), duty);
    }

    fn get_channel(&mut self, ch: PwmChannel) -> &mut PwmLedChannelState {
        match ch {
            PwmChannel::Ch1 => &mut self.ch1,
            PwmChannel::Ch2 => &mut self.ch2,
            PwmChannel::Ch3 => &mut self.ch3,
            PwmChannel::Ch4 => &mut self.ch4,
        }
    }

    async fn on_onof(&mut self, ch: Channel, on: bool) {
        if ch.is_none() {
            return;
        }
        let max_duty = self.pwm.get_max_duty();
        let limit = if on { max_duty - 1 } else { 0 };
        let dur = self.warming / max_duty.into();
        let pwmch: PwmChannel = ch.into();
        let duty = self.get_channel(pwmch).duty;
        ch.for_each(|ch| {
            let sch = self.get_channel(ch);
            sch.last_direction = on.into();
            sch.duty = duty;
            self.upd_duty(0, ch);
        });
        while self.get_channel(pwmch).duty != limit {
            Timer::after(dur).await;
            ch.for_each(|ch| {
                if on {
                    self.upd_duty(1, ch);
                } else {
                    self.upd_duty(-1, ch);
                }
            });
        }
    }

    async fn on_switched(&mut self, ch: Channel) {
        let pwmch: PwmChannel = ch.into();
        let on: bool = self.get_channel(pwmch).last_direction.into();
        self.on_onof(ch, !on).await
    }

    async fn on_hold(&mut self, ch: Channel, hold: bool) {
        if hold {
            self.on_switched(ch).await;
        } // else - executer should stop previously called hold event
    }
}

impl<'d, T> Handler for PwmLed<'d, T>
where
    T: CaptureCompare16bitInstance,
{
    type Data = (PwmLedEvent, Channel);
    async fn handle(&mut self, ev: Self::Data) {
        let ch = ev.1;
        let ev = ev.0;

        match ev {
            PwmLedEvent::OnOf(_) => self.on_switched(ch).await,
            PwmLedEvent::Switched => self.on_switched(ch).await,
            PwmLedEvent::Hold(hold) => self.on_hold(ch, hold).await,
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
    EvData: Into<PwmLedEvent>,
{
    type Data = (PwmLedEvent, Channel);
    fn initial(&mut self) -> Option<Self::Data> {
        let data = self.event.initial();
        if let Some(data) = data {
            Some((data.into(), self.ch))
        } else {
            None
        }
    }
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
