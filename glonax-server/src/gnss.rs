use glonax::{
    runtime::{Component, ComponentContext},
    RobotState,
};
use glonax_serial::Uart;
use tokio::io::{AsyncBufReadExt, BufReader, Lines};

pub struct GNSS {
    service: glonax::net::NMEAService,
    line_reader: Lines<BufReader<Uart>>,
}

impl GNSS {
    pub fn new() -> Self {
        log::debug!("Starting GNSS service");

        let serial = glonax_serial::Uart::open(
            std::path::Path::new("/dev/ttyACM0"),
            glonax_serial::BaudRate::from_speed(9_600),
        )
        .unwrap();

        let reader = BufReader::new(serial);
        let lines = reader.lines();

        // match glonax_serial::Uart::open(std::path::Path::new("/dev/ttyACM0"), glonax_serial::BaudRate::from_speed(9_600)) {
        //     Ok(serial) => {
        //         // let reader = BufReader::new(serial);
        //         // let mut lines = reader.lines();

        //         let service = glonax::net::NMEAService;

        //         // while let Ok(Some(line)) = lines.next_line().await {
        //         //     if let Some(message) = service.decode(line) {
        //         //         message.fill(runtime_state.clone()).await;
        //         //     }
        //         // }
        //     }
        //     Err(e) => {
        //         log::error!("Failed to open serial: {}", e);
        //         tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        //     }
        // }

        Self {
            service: glonax::net::NMEAService,
            line_reader: lines,
        }
    }
}

impl<R: RobotState> Component<R> for GNSS {
    fn tick(&mut self, _ctx: &mut ComponentContext, _state: &mut R) {
        // if let Some(message) = self.service.decode(line) {
        //     message.fill(state.clone()).await;
        // }
    }
}
