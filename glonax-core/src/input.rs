/// Input device scancode.
///
/// Scancodes are indirectly mapped to input pheripherials. Any
/// input device can emit these codes. There effect is left to
/// device implementations.
pub enum Scancode {
    /// Left stick X axis.
    LeftStickX(f32),
    /// Left stick Y axis.
    LeftStickY(f32),
    /// Right stick X axis.
    RightStickX(f32),
    /// Right stick Y axis.
    RightStickY(f32),
    /// Left trigger axis.
    LeftTrigger(f32),
    /// Right trigger axis.
    RightTrigger(f32),
    /// Activate button.
    Activate,
    /// Cancel button.
    Cancel,
}
