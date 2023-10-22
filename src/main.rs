#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]

use embassy_executor::Spawner;
use embassy_stm32::exti::{AnyChannel, Channel as _};
use embassy_stm32::gpio::{AnyPin, Level, Output, Pin, Pull, Speed};
use embassy_time::{Duration, Timer};
use wired_home::*;

use embassy_stm32::pwm::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::pwm::Channel;
use embassy_stm32::time::hz;
use wired_home::event::JoinEvent;

use {defmt_rtt as _, panic_probe as _};

async fn led_button_task(button: AnyPin, channel: AnyChannel, led: AnyPin) {}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let led = Output::new(p.PC13, Level::High, Speed::Low);
    let led = Led::from(led);

    let btn1 = DebonceExtiInput::new(p.PB11, p.EXTI11, Pull::Up, Duration::from_millis(5)).await;
    let btn1 = ExtiInputPushEvent::new(btn1, Duration::from_millis(300));
    let btn1 = SinglePwmLedInput::new(btn1, Channel::Ch2);

    let btn2 = DebonceExtiInput::new(p.PB10, p.EXTI10, Pull::Up, Duration::from_millis(5)).await;
    let btn2 = ExtiInputSwitcherEvent::new(btn2);
    let btn2 = SinglePwmLedInput::new(btn2, Channel::Ch2);

    let mut pwm = SimplePwm::new(
        p.TIM1,
        Some(PwmPin::new_ch1(p.PA8)),
        Some(PwmPin::new_ch2(p.PA9)),
        None,
        None,
        hz(2000),
    );
    pwm.set_freq(hz(2000));
    pwm.enable(Channel::Ch1);
    pwm.enable(Channel::Ch2);
    let pwm = PwmLed::new(pwm, Duration::from_secs(2));

    ConnectionInterrupting::new(JoinEvent::new(btn1, btn2), pwm)
        .run()
        .await;
}
