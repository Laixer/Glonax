#[derive(Debug, Clone, serde_derive::Serialize)]
pub struct Telemetry {
    pub location: Option<(f32, f32)>,
    pub altitude: Option<f32>,
    pub speed: Option<f32>,
    pub heading: Option<f32>,
    pub satellites: Option<u8>,
    pub memory: Option<u64>,
    pub swap: Option<u64>,
    pub cpu_load: Option<(f64, f64, f64)>,
    pub uptime: Option<u64>,
    pub rpm: Option<u16>,
    pub encoders: std::collections::HashMap<u8, f32>,
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
            cpu_load: None,
            uptime: None,
            rpm: None,
            encoders: std::collections::HashMap::new(),
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

        if let Some((cpu_1, cpu_5, cpu_15)) = self.cpu_load {
            s.push_str(&format!(" CPU 1: {}%", cpu_1));
            s.push_str(&format!(" CPU 5: {}%", cpu_5));
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
