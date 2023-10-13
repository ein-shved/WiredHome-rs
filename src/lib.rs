#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]

pub mod event;
pub mod debounce;
pub mod light;

pub use event::Connection;

pub use debounce::{ButtonState, WaitDebonce};
pub use debounce::exti::DebonceExtiInput;
pub use debounce::exti_event::ExtiInputSwitcherEvent;

pub use light::Led;

