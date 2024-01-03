#![allow(dead_code)]
struct Pid {
    /// Proportional gain
    kp: f32,
    /// Integral gain
    ki: f32,
    /// Derivative gain
    kd: f32,
    /// Last error value
    last_error: f32,
    /// Integral of error
    integral: f32,
}

#[allow(dead_code)]
impl Pid {
    /// Constructor to create a new PID controller
    pub fn new(kp: f32, ki: f32, kd: f32) -> Pid {
        Pid {
            kp,
            ki,
            kd,
            last_error: 0.0,
            integral: 0.0,
        }
    }

    /// Method to update the PID controller based on the current error
    pub fn update(&mut self, error: f32, dt: f32) -> f32 {
        // Calculate integral of error
        self.integral += error * dt;

        // Calculate derivative of error
        let derivative = (error - self.last_error) / dt;

        // Remember this error for next time
        self.last_error = error;

        // Calculate control output
        self.kp * error + self.ki * self.integral + self.kd * derivative
    }
}
