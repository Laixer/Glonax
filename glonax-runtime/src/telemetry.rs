#[derive(Debug, Clone, serde_derive::Serialize)]
pub struct Telemetry {
    pub location: Option<(f32, f32)>,
    pub altitude: Option<f32>,
    pub speed: Option<f32>,
    pub heading: Option<f32>,
    pub satellites: Option<u8>,
    pub memory: Option<u64>,
    pub swap: Option<u64>,
    pub cpu_1: Option<f64>,
    pub cpu_5: Option<f64>,
    pub cpu_15: Option<f64>,
    pub uptime: Option<u64>,
    pub rpm: Option<u16>,
    pub encoders: [f32; 4],
}

impl Telemetry {
    pub fn new() -> Self {
        Self {
            location: None,
            altitude: None,
            speed: None,
            heading: None,
            satellites: None,
            memory: None,
            swap: None,
            cpu_1: None,
            cpu_5: None,
            cpu_15: None,
            uptime: None,
            rpm: None,
            encoders: [0.0; 4],
        }
    }
}

impl Default for Telemetry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Telemetry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        if let Some(uptime) = self.uptime {
            s.push_str(&format!(" Uptime: {}s", uptime));
        }

        if let Some(memory) = self.memory {
            s.push_str(&format!(" Memory: {}%", memory));
        }

        if let Some(swap) = self.swap {
            s.push_str(&format!(" Swap: {}%", swap));
        }

        if let Some(cpu_1) = self.cpu_1 {
            s.push_str(&format!(" CPU 1: {}%", cpu_1));
        }

        if let Some(cpu_5) = self.cpu_5 {
            s.push_str(&format!(" CPU 5: {}%", cpu_5));
        }

        if let Some(cpu_15) = self.cpu_15 {
            s.push_str(&format!(" CPU 15: {}%", cpu_15));
        }

        if let Some((value_lat, value_long)) = self.location {
            s.push_str(&format!(" Location: ({:.5}, {:.5})", value_lat, value_long));
        }

        if let Some(altitude) = self.altitude {
            s.push_str(&format!(" Altitude: {:.1}m", altitude));
        }

        if let Some(speed) = self.speed {
            s.push_str(&format!(" Speed: {:.1}m/s", speed));
        }

        if let Some(heading) = self.heading {
            s.push_str(&format!(" Heading: {:.1}°", heading));
        }

        if let Some(satellites) = self.satellites {
            s.push_str(&format!(" Satellites: {}", satellites));
        }

        if let Some(rpm) = self.rpm {
            s.push_str(&format!(" RPM: {}", rpm));
        }

        write!(f, "{}", s)
    }
}
