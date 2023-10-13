use defmt::*;
use crate::event::Handler;
use embassy_stm32::gpio::{Level, Output, Pin};

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
        info!("Got event {}", on);
        self.output
            .set_level(if on { Level::Low } else { Level::High })
    }
}
