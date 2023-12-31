use crate::event::Event;
use crate::{ButtonState, DebonceExtiInput, WaitDebonce};
use embassy_stm32::gpio::Pin;
use embassy_time::{Duration, Timer};

pub struct ExtiInputSwitcherEvent<'d, T: Pin> {
    button: DebonceExtiInput<'d, T>,
}

impl<'d, T: Pin> ExtiInputSwitcherEvent<'d, T> {
    pub fn new(button: DebonceExtiInput<'d, T>) -> ExtiInputSwitcherEvent<'d, T> {
        ExtiInputSwitcherEvent::<'d, T> { button }
    }
}

impl<'d, T: Pin> Event for ExtiInputSwitcherEvent<'d, T> {
    type Data = ButtonState;
    async fn next(&mut self) -> ButtonState {
        self.button.wait_for_change().await
    }
    fn initial(&mut self) -> Option<Self::Data> {
        Some(self.button.get())
    }
}

#[derive(PartialEq, defmt::Format, Clone, Copy)]
pub enum PushState {
    Off,
    Pushed,
    On,
}

pub struct ExtiInputPushEvent<'d, T: Pin> {
    button: DebonceExtiInput<'d, T>,
    state: PushState,
    gap: Duration,
}

fn state_from_button(btn: ButtonState) -> PushState {
        match btn {
            ButtonState::High => PushState::Off,
            ButtonState::Low => PushState::On,
        }
}

impl<'d, T: Pin> ExtiInputPushEvent<'d, T> {
    pub fn new(button: DebonceExtiInput<'d, T>, gap: Duration) -> ExtiInputPushEvent<'d, T> {
        let state = state_from_button(button.get());
        ExtiInputPushEvent::<'d, T> { button, gap, state }
    }
}

impl<'d, T: Pin> Event for ExtiInputPushEvent<'d, T> {
    type Data = PushState;
    async fn next(&mut self) -> PushState {
            use defmt::*;
        let res = loop {
            use embassy_futures::select;
            use select::{select, Either};

            let btn = self.button.wait_for_change().await;
            let btn = state_from_button(btn);

            if self.state == btn {
                continue;
            }
            if btn == PushState::Off {
                self.state = PushState::Off;
                break self.state;
            }
            // Wait for gap or for ButtonState::Low next
            let to = Timer::after(self.gap);
            let ev = async {
                loop {
                    let btn = self.button.wait_for_change().await;
                    let btn = state_from_button(btn);
                    if btn == PushState::Off {
                        break;
                    }
                }
            };
            break match select(ev, to).await {
                Either::First(_) => {
                    self.state = PushState::Off;
                    PushState::Pushed
                }
                Either::Second(_) => {
                    self.state = PushState::On;
                    self.state
                }
            };
        };

        info!("Got event {:?}", res);
        res
    }
}
