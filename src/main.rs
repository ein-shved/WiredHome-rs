#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]

use embassy_executor::Spawner;
use embassy_stm32::exti::{AnyChannel, Channel};
use embassy_stm32::gpio::{AnyPin, Level, Output, Pin, Pull, Speed};
use embassy_time::Duration;
use wired_home::{Connection, DebonceExtiInput, ExtiInputSwitcherEvent, Led};

use {defmt_rtt as _, panic_probe as _};

async fn led_button_task(button: AnyPin, channel: AnyChannel, led: AnyPin) {
    let led = Output::new(led, Level::High, Speed::Low);
    let btn = DebonceExtiInput::new(button, channel, Pull::Up, Duration::from_millis(5)).await;
    let led = Led::from(led);
    let btn = ExtiInputSwitcherEvent::new(btn);
    Connection::new(btn, led).run().await;
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    led_button_task(p.PB11.degrade(), p.EXTI11.degrade(), p.PC13.degrade()).await;
}
