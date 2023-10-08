#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::{AnyChannel, Channel, ExtiInput};
use embassy_stm32::gpio;
use embassy_stm32::gpio::{AnyPin, Input, Level, Output, Pin, Speed};
use embassy_stm32::Peripheral;
use futures::join;

use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[derive(PartialEq, Clone)]
enum State {
    High,
    Low,
}

trait WaitDebonce {
    async fn wait_for_change(&mut self) -> State;
}

async fn read_state_debonce<T: Pin>(input: &mut ExtiInput<'_, T>, deb: Duration) -> State {
    let do_read = || {
        if input.is_low() {
            State::Low
        } else {
            State::High
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
struct DebonceExtiInput<'d, T: Pin> {
    state: State,
    input: ExtiInput<'d, T>,
}

impl<'d, T: Pin> DebonceExtiInput<'d, T> {
    pub async fn new(pin: T, channel: impl Peripheral<P = T::ExtiChannel> + 'd) -> DebonceExtiInput<'d, T> {
        let btn = Input::new(pin, gpio::Pull::Up);
        let btn = ExtiInput::new(btn, channel);
        Self::from(btn).await
    }
    pub async fn from(mut input: ExtiInput<'d, T>) -> DebonceExtiInput<'d, T> {
        Self {
            state: read_state_debonce(&mut input, Duration::from_millis(5)).await,
            input,
        }
    }
}

impl<T: Pin> WaitDebonce for DebonceExtiInput<'_, T> {
    async fn wait_for_change(&mut self) -> State {
        loop {
            let deb = Duration::from_millis(5);
            let st1 = read_state_debonce(&mut self.input, deb).await;
            if st1 != self.state {
                self.state = st1;
                break self.state.clone();
            }
            self.input.wait_for_any_edge().await;
            let st2 = read_state_debonce(&mut self.input, deb).await;
            if st1 != st2 {
                self.state = st2;
                break self.state.clone();
            }
        }
    }
}

async fn led_button_task(button: AnyPin, channel: AnyChannel, led: AnyPin) {
    let mut led = Output::new(led, Level::High, Speed::Low);
    let mut btn = DebonceExtiInput::new(button, channel).await;
    loop {
        match btn.wait_for_change().await {
            State::Low => {
                info!("Low!");
                led.set_low()
            }
            State::High => {
                info!("High!");
                led.set_high();
            }
        }
    }
}

async fn hello() {
    loop {
        async {
            Timer::after(Duration::from_secs(1)).await;
            info!("Hello WiredHome-rs");
        }
        .await
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let hello = hello();
    let but = led_button_task(p.PB11.degrade(), p.EXTI11.degrade(), p.PC13.degrade());
    join!(hello, but);
}
