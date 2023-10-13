#[derive(PartialEq, Clone, Copy)]
pub enum ButtonState {
    High,
    Low,
}

impl From<ButtonState> for bool {
    fn from(st: ButtonState) -> bool {
        st == ButtonState::Low
    }
}

pub trait WaitDebonce {
    async fn wait_for_change(&mut self) -> ButtonState;
}

pub mod exti;
pub mod exti_event;
