use super::{ButtonState, WaitDebonce};

use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Pin, Pull};
use embassy_stm32::Peripheral;
use embassy_time::{Duration, Timer};

pub struct DebonceExtiInput<'d, T: Pin> {
    state: ButtonState,
    input: ExtiInput<'d, T>,
    debonce: Duration,
}

impl<'d, T: Pin> DebonceExtiInput<'d, T> {
    pub async fn new(
        pin: T,
        channel: impl Peripheral<P = T::ExtiChannel> + 'd,
        pull_mode: Pull,
        debonce: Duration,
    ) -> DebonceExtiInput<'d, T> {
        let btn = Input::new(pin, pull_mode);
        let btn = ExtiInput::new(btn, channel);
        Self::from(btn, debonce).await
    }
    pub async fn from(mut input: ExtiInput<'d, T>, debonce: Duration) -> DebonceExtiInput<'d, T> {
        Self {
            state: read_state_debonce(&mut input, Duration::from_millis(5)).await,
            input,
            debonce,
        }
    }
}

impl<T: Pin> WaitDebonce for DebonceExtiInput<'_, T> {
    async fn wait_for_change(&mut self) -> ButtonState {
        loop {
            let st1 = read_state_debonce(&mut self.input, self.debonce).await;
            if st1 != self.state {
                self.state = st1;
                break self.state.clone();
            }
            self.input.wait_for_any_edge().await;
            let st2 = read_state_debonce(&mut self.input, self.debonce).await;
            if st1 != st2 {
                self.state = st2;
                break self.state.clone();
            }
        }
    }
}

async fn read_state_debonce<T: Pin>(input: &mut ExtiInput<'_, T>, deb: Duration) -> ButtonState {
    let do_read = || {
        if input.is_low() {
            ButtonState::Low
        } else {
            ButtonState::High
        }
    };
    loop {
        let st1 = do_read();
        Timer::after(deb).await;
        let st2 = do_read();
        if st1 == st2 {
            break st1;
        }
    }
}
