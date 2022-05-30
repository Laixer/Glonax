pub struct SerialDeviceProfile {}

impl super::IoDeviceProfile for SerialDeviceProfile {
    const CLASS: super::Subsystem = super::Subsystem::TTY;

    fn properties() -> std::collections::HashMap<&'static str, &'static str> {
        let mut props = std::collections::HashMap::<&str, &str>::new();
        props.insert("ID_USB_DRIVER", "cp210x");
        props
    }
}

pub struct NullDeviceProfile {}

impl super::IoDeviceProfile for NullDeviceProfile {
    const CLASS: super::Subsystem = super::Subsystem::Memory;

    #[inline]
    fn filter(device: &udev::Device) -> bool {
        device.sysname().to_str().unwrap() == "null"
    }
}

pub struct CanDeviceRuleset {}

impl super::IoDeviceProfile for CanDeviceRuleset {
    const CLASS: super::Subsystem = super::Subsystem::Net;

    #[inline]
    fn filter(device: &udev::Device) -> bool {
        device.sysname().to_str().unwrap().starts_with("can")
    }
}
