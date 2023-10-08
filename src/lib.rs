#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]

pub mod debounce;

pub use debounce::{ButtonState, WaitDebonce};
pub use debounce::exti::DebonceExtiInput;
