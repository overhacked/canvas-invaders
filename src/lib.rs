use std::{rc::Rc, cell::RefCell};

use wasm_bindgen::{prelude::*, Clamped, JsCast};
use web_sys::{ImageData, CanvasRenderingContext2d, console};

#[wasm_bindgen(start)]
pub fn start() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("game").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    let width = canvas.width();
    let height = canvas.height();
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();
    context.set_font("bold 48px serif");

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut fleet = Fleet::new(4, 6, 15.0);
    let max_offset = (width - fleet.size().x() as u32).min(height - fleet.size().y() as u32); // TODO: don't cast
    let mut last_ts = window.performance().unwrap().now();

    let closure: Closure<dyn FnMut(f64)> = Closure::new(move |ts: f64| {
        if fleet.position().x() > max_offset.into() {
            let _ = f.borrow_mut().take();
            return;
        }
        // Paint fresh each time
        context.clear_rect(0.0, 0.0, width.into(), height.into());

        let ts_offset = ts - last_ts;
        last_ts = ts;
        fleet.animate(&context, ts_offset);

        request_animation_frame(f.borrow().as_ref().unwrap());
    });
    *g.borrow_mut() = Some(closure);

    request_animation_frame(g.borrow().as_ref().unwrap());
}

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    let window = web_sys::window().expect("no global `window` exists");
    window
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[derive(Clone, Copy, Debug)]
struct Coordinates {
    x: f64,
    y: f64,
}

trait XY {
    fn get_coordinates(&self) -> Coordinates;
    fn get_coordinates_mut(&mut self) -> &mut Coordinates;
    fn x(&self) -> f64 {
        self.get_coordinates().x
    }

    fn y(&self) -> f64 {
        self.get_coordinates().y
    }

    fn set_x(&mut self, x: f64) {
        self.get_coordinates_mut().x = x;
    }

    fn set_y(&mut self, y: f64) {
        self.get_coordinates_mut().y = y;
    }

    fn set(&mut self, x: f64, y: f64) {
        self.set_x(x);
        self.set_y(y);
    }

    fn offset(&mut self, offset_x: f64, offset_y: f64) {
        self.set_x(self.x() + offset_x);
        self.set_y(self.y() + offset_y);
    }
}

#[derive(Clone, Copy, Debug)]
struct Position(Coordinates);

impl XY for Position {
    fn get_coordinates(&self) -> Coordinates {
        self.0
    }

    fn get_coordinates_mut(&mut self) -> &mut Coordinates {
        &mut self.0
    }
}

impl Default for Position {
    fn default() -> Self {
        Self(Coordinates { x: 0.0, y: 0.0 })
    }
}

#[derive(Clone, Copy, Debug)]
struct Size(Coordinates);

impl XY for Size {
    fn get_coordinates(&self) -> Coordinates {
        self.0
    }

    fn get_coordinates_mut(&mut self) -> &mut Coordinates {
        &mut self.0
    }
}

trait Draw {
    fn position(&self) -> Position;
    fn size(&self) -> Size;
    fn draw(&mut self, context: &CanvasRenderingContext2d);
}

struct Block {
    size: Size,
    position: Position,
    data: Vec<u8>,
}

impl Block {
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
            size: Size(
                Coordinates {x: width.into(), y: height.into()}
            ),
            position: Default::default(),
            data,
        })
    }
}

impl Draw for Block {
    fn position(&self) -> Position {
        self.position
    }

    fn size(&self) -> Size {
        self.size
    }

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

struct Fleet {
    size: Size,
    position: Position,
    rate: f64,
    spacing: f64,
    members: Vec<Vec<Block>>,
}

impl Fleet {
    const MEMBER_WIDTH: u32 = 32;
    const MEMBER_HEIGHT: u32 = 32;

    fn new(rows: u32, columns: u32, spacing: f64) -> Self {
        let mut members = Vec::new();
        for row_idx in 0..rows {
            let mut row = Vec::new();
            for col_idx in 0..columns {
                let mut member = Block::new(Self::MEMBER_WIDTH, Self::MEMBER_HEIGHT).expect("Block"); // TODO: dynamic size
                member.position.set_x(f64::from(col_idx) * (member.size().x() + spacing));
                member.position.set_y(f64::from(row_idx) * (member.size().y() + spacing));
                row.push(member); 
            }
            members.push(row);
        }

        Self {
            size: Size(Coordinates{
                x: (f64::from(columns) * (f64::from(Self::MEMBER_WIDTH) + spacing)) - spacing,
                y: (f64::from(rows) * (f64::from(Self::MEMBER_HEIGHT) + spacing)) - spacing,
            }),
            position: Default::default(),
            rate: 0.041, // TODO
            spacing,
            members,
        }
    }

    fn animate(&mut self, context: &CanvasRenderingContext2d, offset_ts: f64) {
        let offset_x = offset_ts * self.rate;
        let offset_y = offset_ts * self.rate;
        self.offset(offset_x, offset_y);
        self.draw(context);
    }
}

impl XY for Fleet {
    fn get_coordinates(&self) -> Coordinates {
        self.position.0
    }

    fn get_coordinates_mut(&mut self) -> &mut Coordinates {
        &mut self.position.0
    }

    fn set_x(&mut self, x: f64) {
        for row in self.members.iter_mut() {
            for (col_idx, member) in row.iter_mut().enumerate() {
                member.position.set_x(((col_idx as f64) * (member.size().x() + self.spacing)) + x);
            }
        }
        self.position.set_x(x);
    }

    fn set_y(&mut self, y: f64) {
        for (row_idx, row) in self.members.iter_mut().enumerate() {
            for member in row.iter_mut() {
                member.position.set_y(((row_idx as f64) * (member.size().y() + self.spacing)) + y);
            }
        }
        self.position.set_y(y);
    }
}

impl Draw for Fleet {
    fn position(&self) -> Position {
        self.position
    }

    fn size(&self) -> Size {
        self.size
    }

    fn draw(&mut self, context: &CanvasRenderingContext2d) {
        // TODO: make this less dumb
        for row in self.members.iter_mut() {
            for member in row.iter_mut() {
                member.draw(context);
            }
        }
    }
}
