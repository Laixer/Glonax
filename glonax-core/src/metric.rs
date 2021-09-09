use crate::position::Position;

#[derive(Debug, Clone, Copy)]
pub enum MetricValue {
    Temperature(i16),
    Position(Position),
}
