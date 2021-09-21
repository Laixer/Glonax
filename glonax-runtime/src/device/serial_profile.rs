pub struct SerialDeviceProfile {}

impl super::IoDeviceProfile for SerialDeviceProfile {
    const CLASS: super::Subsystem = super::Subsystem::TTY;

    fn properties() -> std::collections::HashMap<&'static str, &'static str> {
        let mut props = std::collections::HashMap::<&str, &str>::new();
        props.insert("ID_USB_DRIVER", "cp210x");
        props
    }
}
