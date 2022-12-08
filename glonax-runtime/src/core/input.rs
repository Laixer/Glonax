/// Button state.
#[derive(PartialEq, Eq)]
pub enum ButtonState {
    /// Button pressed.
    Pressed,
    /// Button released.
    Released,
}

/// Input device scancode.
///
/// Scancodes are indirectly mapped to input pheripherials. Any
/// input device can emit these codes. Their effect is left to
/// device implementations.
pub enum Scancode {
    /// Left stick X axis.
    LeftStickX(i16),
    /// Left stick Y axis.
    LeftStickY(i16),
    /// Right stick X axis.
    RightStickX(i16),
    /// Right stick Y axis.
    RightStickY(i16),
    /// Left trigger axis.
    LeftTrigger(i16),
    /// Right trigger axis.
    RightTrigger(i16),
    /// Activate button.
    Activate(ButtonState),
    /// Cancel button.
    Cancel(ButtonState),
    /// Activate button.
    Restrict(ButtonState),
}
