use std::{rc::Rc, cell::RefCell, sync::mpsc};

use geom::{Size, Coordinates, Position, XY, Rect, Distance};
use wasm_bindgen::{prelude::*, Clamped, JsCast};
use web_sys::{ImageData, CanvasRenderingContext2d, console};

mod geom;

type TimeStamp = f64;

const MARGIN_X: Distance = 30.0;
const MARGIN_Y: Distance = 30.0;

#[wasm_bindgen(start)]
pub fn start() {
    let window = web_sys::window().unwrap();

    let (key_sender, key_receiver) = mpsc::sync_channel(100);
    let key_event_closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
        // try_send so filling the buffer with backlogged keystrokes never blocks this closure
        let send_result = key_sender.try_send(event);

        // Log failures to console for troubleshooting, with cause of failure
        if let Err(ref err @ (mpsc::TrySendError::Full(ref evt)|mpsc::TrySendError::Disconnected(ref evt))) = send_result {
            console::log_1(&format!("Failed to send key event, {}: {}", err, evt.key()).into());
        }
    });
    window.add_event_listener_with_callback("keydown", key_event_closure.as_ref().unchecked_ref()).unwrap();
    window.add_event_listener_with_callback("keyup", key_event_closure.as_ref().unchecked_ref()).unwrap();
    // Must std::mem::forget() the closure so JavaScript holds onto the memory for the lifetime of
    // the program
    key_event_closure.forget();

    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("game").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    let width = Distance::from(canvas.width());
    let height = Distance::from(canvas.height());

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    // The closure will need to be held onto and re-submitted for `request_animation_frame`
    // callbacks from within the body of the closure, so we need a reference-counted pointer that
    // we can hold within the closure and also a handle to it from the outside to kick off the loop
    let animation_closure = Rc::new(RefCell::new(None));
    let animation_closure_initial = animation_closure.clone();

    // Initialze game "globals" that the closure will take ownership over
    // TODO: make these pixel values more dynamic
    let mut enemies = Fleet::new(4, 6, MARGIN_Y, MARGIN_X, width - MARGIN_X);
    let mut ship = Ship::new(0.5, height - MARGIN_Y, MARGIN_X, width - MARGIN_X);
    let mut last_ts = window.performance().unwrap().now();

    let closure_inner: Closure<dyn FnMut(TimeStamp)> = Closure::new(move |ts: TimeStamp| {
        match key_receiver.try_recv() {
            Ok(evt) => {
                let evt_type = evt.type_();
                console::log_1(&format!("Key event: {} {} ({})", evt_type, evt.key(), evt.key_code()).into());
                match evt.key().as_str() {
                    "a"|"ArrowLeft" => {
                        ship.direction = if evt_type == "keydown" { Direction::Left } else { Direction::Stopped };
                    },
                    "d"|"ArrowRight" => {
                        ship.direction = if evt_type == "keydown" { Direction::Right } else { Direction::Stopped };
                    },
                    _ => {}, // Ignore
                }
            },
            Err(mpsc::TryRecvError::Empty) => {}, // OK, no keys pressed
            Err(err) => {
                console::log_1(&format!("Failed to receive key event, {}", err).into());
            },
        }
        context.clear_rect(0.0, 0.0, width, height);

        let ts_offset = ts - last_ts;
        last_ts = ts;
        enemies.animate(&context, ts_offset);
        ship.animate(&context, ts_offset);

        request_animation_frame(animation_closure.borrow().as_ref().unwrap());
    });
    *animation_closure_initial.borrow_mut() = Some(closure_inner);

    request_animation_frame(animation_closure_initial.borrow().as_ref().unwrap());
}

fn request_animation_frame(f: &Closure<dyn FnMut(TimeStamp)>) {
    let window = web_sys::window().expect("no global `window` exists");
    window
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

trait Draw {
    fn draw(&mut self, context: &CanvasRenderingContext2d);
}

struct Entity {
    size: Size,
    position: Position,
    data: Vec<u8>,
}

impl Entity {
    fn new(width: u32, height: u32) -> Result<Self, JsValue> {
        let mut data = Vec::new();
        for _x in 0..width {
            for _y in 0..height {
                data.push(255u8); // R
                data.push(0u8); // G
                data.push(0u8); // B
                data.push(255u8); // a
            }
        }

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
        ).expect("ImageData");

        context.put_image_data(&image, x, y)
            .expect("put_image_data");
    }
}

#[derive(Default)]
enum Direction {
    Left,
    #[default]
    Stopped,
    Right,
}

struct Ship {
    inner: Entity,
    direction: Direction,
    rate: f64,
}

impl Ship {
    const SHIP_WIDTH: u32 = 16;
    const SHIP_HEIGHT: u32 = 16;

    fn new(rate: f64, y_position: Distance, left_bound: Distance, right_bound: Distance) -> Self {
        let mut inner = Entity::new(Self::SHIP_WIDTH, Self::SHIP_HEIGHT).unwrap();
        let position = inner.position_mut();
        position.set_offset_x(geom::OffsetStrategy::limit(left_bound, right_bound - Distance::from(Self::SHIP_WIDTH)));
        let center = left_bound
            + ((right_bound - left_bound) / 2.0)
            + (Distance::from(Self::SHIP_WIDTH) / 2.0);
        position.set_x(center);
        position.set_offset_y(geom::OffsetStrategy::limit(y_position, y_position));
        position.set_y(y_position);

        Self {
            inner,
            direction: Default::default(),
            rate,
        }
    }

    fn animate(&mut self, context: &CanvasRenderingContext2d, offset_ts: TimeStamp) {
        let offset = offset_ts * self.rate;
        match self.direction {
            Direction::Left => self.inner.position_mut().offset(-offset, 0.0),
            Direction::Right => self.inner.position_mut().offset(offset, 0.0),
            Direction::Stopped => {},
        }
        self.inner.draw(context);
    }
}

struct Fleet {
    size: Size,
    position: Position,
    rate: f64,
    spacing: Distance,
    members: Vec<Vec<Entity>>,
}

impl Fleet {
    const MEMBER_WIDTH: u32 = 32;
    const MEMBER_HEIGHT: u32 = 32;

    fn new(rows: u32, columns: u32, spacing: Distance, left_bound: Distance, right_bound: Distance) -> Self {
        let mut members = Vec::new();
        for row_idx in 0..rows {
            let mut row = Vec::new();
            for col_idx in 0..columns {
                let mut member = Entity::new(Self::MEMBER_WIDTH, Self::MEMBER_HEIGHT).expect("Block"); // TODO: dynamic size
                member.position.set_x(Distance::from(col_idx) * (member.size().x() + spacing));
                member.position.set_y(Distance::from(row_idx) * (member.size().y() + spacing));
                row.push(member); 
            }
            members.push(row);
        }

        let size = Size::new(
            (Distance::from(columns) * (Distance::from(Self::MEMBER_WIDTH) + spacing)) - spacing,
            (Distance::from(rows) * (Distance::from(Self::MEMBER_HEIGHT) + spacing)) - spacing,
        );
        let mut position = Position::new(left_bound, 60.0); // TODO: 60.0 to variable
        position.set_offset_x(geom::OffsetStrategy::cycle(left_bound, right_bound - size.x()));
        Self {
            size,
            position,
            rate: 0.03, // TODO
            spacing,
            members,
        }
    }

    fn animate(&mut self, context: &CanvasRenderingContext2d, offset_ts: TimeStamp) {
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
                (member as &mut dyn Rect).set_x(((col_idx as Distance) * (member_width + self.spacing)) + x);
            }
        }
        self.position.set_x(x);
    }

    fn set_y(&mut self, y: Distance) {
        for (row_idx, row) in self.members.iter_mut().enumerate() {
            for member in row.iter_mut() {
                let member_height = member.size().y();
                (member as &mut dyn Rect).set_y(((row_idx as Distance) * (member_height + self.spacing)) + y);
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
