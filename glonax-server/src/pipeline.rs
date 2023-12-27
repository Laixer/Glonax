use glonax::runtime::Service;

use crate::state::{Component, ComponentContext, Excavator};

#[derive(Default)]
pub struct PipelineComponent {
    services: Vec<Box<dyn Component<Excavator>>>,
}

impl PipelineComponent {
    pub fn new() -> Self {
        let services: Vec<Box<dyn Component<Excavator>>> = vec![
            // Box::new(glonax::net::HostService::default()),
            Box::new(crate::kinematic::KinematicComponent::default()),
        ];

        Self { services }
    }
}

impl Component<Excavator> for PipelineComponent {
    fn tick(&mut self, _ctx: &mut ComponentContext, runtime_state: &mut Excavator) {
        for service in self.services.iter_mut() {
            service.tick(_ctx, runtime_state);
        }
    }
}
