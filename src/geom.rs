pub type Distance = f64;

#[derive(Clone, Copy, Debug, Default)]
pub struct Coordinates {
    x: Distance,
    y: Distance,
    x_strategy: OffsetStrategy,
    y_strategy: OffsetStrategy,
}

impl XY for Coordinates {
    fn get_coordinates(&self) -> Coordinates {
        *self
    }

    fn get_coordinates_mut(&mut self) -> &mut Coordinates {
        self
    }
}

pub trait XY {
    fn get_coordinates(&self) -> Coordinates;
    fn get_coordinates_mut(&mut self) -> &mut Coordinates;
    fn x(&self) -> Distance {
        self.get_coordinates().x
    }

    fn y(&self) -> Distance {
        self.get_coordinates().y
    }

    fn set_x(&mut self, x: Distance) {
        self.get_coordinates_mut().x = x;
    }

    fn set_y(&mut self, y: Distance) {
        self.get_coordinates_mut().y = y;
    }

    fn set(&mut self, x: Distance, y: Distance) {
        self.set_x(x);
        self.set_y(y);
    }

    fn set_offset_x(&mut self, strategy: OffsetStrategy) {
        self.get_coordinates_mut().x_strategy = strategy;
    }

    fn set_offset_y(&mut self, strategy: OffsetStrategy) {
        self.get_coordinates_mut().y_strategy = strategy;
    }

    fn set_offset_xy(&mut self, strategy: OffsetStrategy) {
        self.set_offset_x(strategy);
        self.set_offset_y(strategy);
    }

    fn offset(&mut self, offset_x: Distance, offset_y: Distance) {
        let x = self.x();
        let y = self.y();
        let this = self.get_coordinates_mut();
        let new_x = this.x_strategy.offset(x, offset_x);
        let new_y = this.y_strategy.offset(y, offset_y);

        self.set_x(new_x);
        self.set_y(new_y);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Position(Coordinates);

impl Position {
    pub fn new(x: Distance, y: Distance) -> Self {
        Self(Coordinates {
            x,
            y,
            ..Default::default()
        })
    }
}

impl XY for Position {
    fn get_coordinates(&self) -> Coordinates {
        self.0
    }

    fn get_coordinates_mut(&mut self) -> &mut Coordinates {
        &mut self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Size(Coordinates);

impl Size {
    pub fn new(x: Distance, y: Distance) -> Self {
        Self(Coordinates {
            x,
            y,
            ..Default::default()
        })
    }
}

impl XY for Size {
    fn get_coordinates(&self) -> Coordinates {
        self.0
    }

    fn get_coordinates_mut(&mut self) -> &mut Coordinates {
        &mut self.0
    }
}

pub trait Rect {
    fn position(&self) -> Position;
    fn position_mut(&mut self) -> &mut Position;
    fn size(&self) -> Size;
    fn extent(&self) -> Position {
        let origin = self.position();
        let size = self.size();
        Position::new(origin.x() + size.x(), origin.y() + size.y())
    }
}

impl XY for dyn Rect {
    fn get_coordinates(&self) -> Coordinates {
        self.position().get_coordinates()
    }

    fn get_coordinates_mut(&mut self) -> &mut Coordinates {
        self.position_mut().get_coordinates_mut()
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub enum OffsetStrategy {
    Linear,
    Limit {
        min: Distance,
        max: Distance,
    },
    Cycle {
        min: Distance,
        max: Distance,
        direction: Distance,
    },
}

impl Default for OffsetStrategy {
    fn default() -> Self {
        Self::linear()
    }
}

impl OffsetStrategy {
    pub fn linear() -> Self {
        Self::Linear
    }

    pub fn limit(min: Distance, max: Distance) -> Self {
        Self::Limit { min, max }
    }

    pub fn cycle(min: Distance, max: Distance) -> Self {
        Self::Cycle {
            min,
            max,
            direction: 1.0,
        }
    }

    pub fn offset(&mut self, current: Distance, offset: Distance) -> Distance {
        match self {
            OffsetStrategy::Linear => (current + offset),
            OffsetStrategy::Limit { min, max } => (current + offset).min(*max).max(*min),
            OffsetStrategy::Cycle {
                min,
                max,
                direction,
            } => {
                let mut result = current + offset.copysign(*direction);
                loop {
                    if result > *max {
                        let offset = result - *max;
                        result -= offset;
                        *direction = -1.0;
                    } else if result < *min {
                        let offset = *min - result;
                        result = *min + offset;
                        *direction = 1.0;
                    } else {
                        break result;
                    }
                }
            }
        }
    }
}
