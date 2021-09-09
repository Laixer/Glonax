/// Input device scancode.
///
/// Scancodes are indirectly mapped to input pheripherials. Any
/// command device can emit these codes. There effect is left to
/// device implementations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Scancode {
    LeftStickX(f32),
    LeftStickY(f32),
    RightStickX(f32),
    RightStickY(f32),
    LeftTrigger(f32),
    RightTrigger(f32),
    Activate,
    Cancel,
}
