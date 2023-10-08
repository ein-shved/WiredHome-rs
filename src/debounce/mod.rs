#[derive(PartialEq, Clone)]
pub enum ButtonState {
    High,
    Low,
}

pub trait WaitDebonce {
    async fn wait_for_change(&mut self) -> ButtonState;
}

pub mod exti;
