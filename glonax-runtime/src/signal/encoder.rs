pub trait Normalize<T> {
    fn normalize(&self, value: T) -> T;
}

impl<T: std::cmp::PartialOrd + std::ops::Sub<Output = T> + Copy> Normalize<T>
    for std::ops::Range<T>
{
    fn normalize(&self, value: T) -> T {
        let domain_value = if value < self.start {
            self.start
        } else if value > self.end {
            self.end
        } else {
            value
        };

        domain_value - self.start
    }
}

pub struct Encoder<T> {
    range_from: std::ops::Range<T>,
    range_to: std::ops::Range<T>,
}

impl<T> Encoder<T>
where
    T: Copy,
    T: std::ops::Div,
    T: std::ops::Sub + std::ops::Sub<Output = T>,
    T: std::ops::Div<<T as std::ops::Sub>::Output>,
    T: std::ops::Mul<<T as std::ops::Div<<T as std::ops::Sub>::Output>>::Output, Output = T>,
    std::ops::Range<T>: Normalize<T>,
{
    pub fn new(range_from: std::ops::Range<T>, range_to: std::ops::Range<T>) -> Self {
        Self {
            range_from,
            range_to,
        }
    }

    #[inline]
    pub fn scale_to(&self, domain: T, value: T) -> T {
        self.range_from.normalize(value) * (domain / (self.range_from.end - self.range_from.start))
    }

    pub fn scale(&self, value: T) -> T {
        self.scale_to(self.range_to.end - self.range_to.start, value)
    }
}
