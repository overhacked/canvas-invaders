mod icons;

use wasm_bindgen::{Clamped, JsValue};
use web_sys::{CanvasRenderingContext2d, ImageData};

use crate::geom::{Coordinates, Distance, OffsetStrategy, Position, Rect, Size, XY};
use crate::graphics::{Draw, TimeStamp};

pub(crate) struct Entity {
    pub(crate) size: Size,
    pub(crate) position: Position,
    pub(crate) data: Vec<u8>,
}

impl Entity {
    pub(crate) fn new(width: u32, height: u32, image: impl AsRef<[u8]>) -> Result<Self, JsValue> {
        let data = image.as_ref().to_vec();

        Ok(Self {
            size: Size::new(width.into(), height.into()),
            position: Default::default(),
            data,
        })
    }
}

impl Rect for Entity {
    fn position(&self) -> Position {
        self.position
    }

    fn position_mut(&mut self) -> &mut Position {
        &mut self.position
    }

    fn size(&self) -> Size {
        self.size
    }
}

impl Draw for Entity {
    fn draw(&mut self, context: &CanvasRenderingContext2d) {
        let x = self.position.x();
        let y = self.position.y();
        let width = self.size.x();
        let height = self.size.y();

        let image = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&self.data),
            width as u32,
            height as u32,
        )
        .expect("ImageData");

        context
            .put_image_data(&image, x, y)
            .expect("put_image_data");
    }
}

#[derive(Default)]
pub(crate) enum Direction {
    Left,
    #[default]
    Stopped,
    Right,
}

pub(crate) struct Ship {
    pub(crate) inner: Entity,
    pub(crate) direction: Direction,
    pub(crate) rate: f64,
    bullets: Vec<Bullet>,
}

impl Ship {
    pub(crate) fn new(
        rate: f64,
        y_position: Distance,
        left_bound: Distance,
        right_bound: Distance,
    ) -> Self {
        let mut inner = Entity::new(icons::SHIP_WIDTH, icons::SHIP_HEIGHT, icons::SHIP).unwrap();
        let position = inner.position_mut();
        position.set_offset_x(OffsetStrategy::limit(
            left_bound,
            right_bound - Distance::from(icons::SHIP_WIDTH),
        ));
        let center = left_bound
            + ((right_bound - left_bound) / 2.0)
            + (Distance::from(icons::SHIP_WIDTH) / 2.0);
        position.set_x(center);
        position.set_offset_y(OffsetStrategy::limit(y_position, y_position));
        position.set_y(y_position - Distance::from(icons::SHIP_HEIGHT));

        Self {
            inner,
            direction: Default::default(),
            rate,
            bullets: Vec::new(),
        }
    }

    pub(crate) fn animate(&mut self, context: &CanvasRenderingContext2d, offset_ts: TimeStamp) {
        let offset = offset_ts * self.rate;
        match self.direction {
            Direction::Left => self.inner.position_mut().offset(-offset, 0.0),
            Direction::Right => self.inner.position_mut().offset(offset, 0.0),
            Direction::Stopped => {}
        }
        self.inner.draw(context);
        // Way better to use nightly's drain_filter here. Alas.
        let mut i = 0;
        while i < self.bullets.len() {
            if self.bullets[i].inner.position().y() < -(f64::from(icons::BULLET_HEIGHT)) {
                // swap_remove more performant here, becuase
                // bullet iteration order doesn't matter
                self.bullets.swap_remove(i);
            } else {
                self.bullets[i].animate(context, offset_ts);
                i += 1;
            }
        }
    }

    pub(crate) fn shoot(&mut self) {
        let position = Position::new(
            self.inner.position().x() + 11.0,
            self.inner.position().y() + 10.0,
        );
        let bullet = Bullet::new(position);
        self.bullets.push(bullet);
    }
}

pub(crate) struct Fleet {
    pub(crate) size: Size,
    pub(crate) position: Position,
    pub(crate) rate: f64,
    pub(crate) spacing: Distance,
    pub(crate) members: Vec<Vec<Entity>>,
}

impl Fleet {
    pub(crate) fn new(
        rows: u32,
        columns: u32,
        spacing: Distance,
        left_bound: Distance,
        right_bound: Distance,
    ) -> Self {
        let mut images = icons::ENEMIES.into_iter().cycle();
        let mut members = Vec::new();
        for row_idx in 0..rows {
            let mut row = Vec::new();
            for col_idx in 0..columns {
                let mut member = Entity::new(
                    icons::ENEMY_WIDTH,
                    icons::ENEMY_HEIGHT,
                    images.next().unwrap(),
                )
                .expect("Block"); // TODO: dynamic size
                member
                    .position
                    .set_x(Distance::from(col_idx) * (member.size().x() + spacing));
                member
                    .position
                    .set_y(Distance::from(row_idx) * (member.size().y() + spacing));
                row.push(member);
            }
            members.push(row);
        }

        let size = Size::new(
            (Distance::from(columns) * (Distance::from(icons::ENEMY_WIDTH) + spacing)) - spacing,
            (Distance::from(rows) * (Distance::from(icons::ENEMY_HEIGHT) + spacing)) - spacing,
        );
        let mut position = Position::new(left_bound, 60.0); // TODO: 60.0 to variable
        position.set_offset_x(OffsetStrategy::cycle(left_bound, right_bound - size.x()));
        Self {
            size,
            position,
            rate: 0.03, // TODO
            spacing,
            members,
        }
    }

    pub(crate) fn animate(&mut self, context: &CanvasRenderingContext2d, offset_ts: TimeStamp) {
        let raw_offset = offset_ts * self.rate;
        self.offset(raw_offset, 0.0);
        self.draw(context);
    }
}

impl XY for Fleet {
    fn get_coordinates(&self) -> Coordinates {
        self.position.get_coordinates()
    }

    fn get_coordinates_mut(&mut self) -> &mut Coordinates {
        self.position.get_coordinates_mut()
    }

    fn set_x(&mut self, x: Distance) {
        for row in self.members.iter_mut() {
            for (col_idx, member) in row.iter_mut().enumerate() {
                let member_width = member.size().x();
                (member as &mut dyn Rect)
                    .set_x(((col_idx as Distance) * (member_width + self.spacing)) + x);
            }
        }
        self.position.set_x(x);
    }

    fn set_y(&mut self, y: Distance) {
        for (row_idx, row) in self.members.iter_mut().enumerate() {
            for member in row.iter_mut() {
                let member_height = member.size().y();
                (member as &mut dyn Rect)
                    .set_y(((row_idx as Distance) * (member_height + self.spacing)) + y);
            }
        }
        self.position.set_y(y);
    }
}

impl Rect for Fleet {
    fn position(&self) -> Position {
        self.position
    }

    fn position_mut(&mut self) -> &mut Position {
        &mut self.position
    }

    fn size(&self) -> Size {
        self.size
    }
}

impl Draw for Fleet {
    fn draw(&mut self, context: &CanvasRenderingContext2d) {
        for row in self.members.iter_mut() {
            for member in row.iter_mut() {
                member.draw(context);
            }
        }
    }
}

pub(crate) struct Bullet {
    pub(crate) inner: Entity,
}

impl Bullet {
    const RATE: f64 = 0.5;

    pub(crate) fn new(position: Position) -> Self {
        let mut inner = Entity::new(icons::BULLET_WIDTH, icons::BULLET_HEIGHT, icons::BULLET).unwrap();
        *inner.position_mut() = position;

        Self {
            inner,
        }
    }

    pub(crate) fn animate(&mut self, context: &CanvasRenderingContext2d, offset_ts: TimeStamp) {
        let pos = self.inner.position_mut();
        let y = pos.y();
        pos.set_y(y - (Self::RATE * offset_ts));
        self.inner.draw(context);
    }
}
