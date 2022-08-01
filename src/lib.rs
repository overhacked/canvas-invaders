use std::{rc::Rc, cell::RefCell};

use wasm_bindgen::{prelude::*, Clamped, JsCast};
use web_sys::{ImageData, CanvasRenderingContext2d};

#[wasm_bindgen(start)]
pub fn start() {
    let document = web_sys::window().unwrap().document().unwrap();
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

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let block_width = 32;
    let mut block = Block::new(block_width, block_width).unwrap();
    let max_offset = (width - block_width).min(height - block_width);
    *g.borrow_mut() = Some(Closure::new(move || {
        if block.position().x() > max_offset {
            let _ = f.borrow_mut().take();
            return;
        }
        block.animate(&context);

        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    let window = web_sys::window().expect("no global `window` exists");
    window
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[derive(Clone, Copy)]
struct Coordinates {
    x: u32,
    y: u32,
}

trait XY: Clone {
    fn get_coordinates(&self) -> Coordinates;
    fn get_coordinates_mut(&mut self) -> &mut Coordinates;
    fn x(&self) -> u32 {
        self.get_coordinates().x
    }

    fn y(&self) -> u32 {
        self.get_coordinates().y
    }

    fn set_x(&mut self, x: u32) {
        self.get_coordinates_mut().x = x;
    }

    fn set_y(&mut self, y: u32) {
        self.get_coordinates_mut().y = y;
    }

    fn set(&mut self, x: u32, y: u32) {
        self.set_x(x);
        self.set_y(y);
    }

    fn offset(&mut self, offset_x: i32, offset_y: i32) -> Self {
        let previous = self.clone();
        let new_x = saturating_offset(self.x(), offset_x);
        let new_y = saturating_offset(self.y(), offset_y);

        self.set_x(new_x);
        self.set_y(new_y);
        previous
    }
}

fn saturating_offset(n: u32, offset: i32) -> u32 {
    let abs_offset = offset.abs_diff(0);
    if offset.is_positive() {
        n + abs_offset
    } else {
        n.saturating_sub(abs_offset)
    }
}

#[derive(Clone, Copy)]
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
        Self(Coordinates { x: 0, y: 0 })
    }
}

#[derive(Clone, Copy)]
struct Size(Coordinates);

impl XY for Size {
    fn get_coordinates(&self) -> Coordinates {
        self.0
    }

    fn get_coordinates_mut(&mut self) -> &mut Coordinates {
        &mut self.0
    }
}

trait Animate {
    fn position(&self) -> Position;
    fn size(&self) -> Size;
    fn animate(&mut self, context: &CanvasRenderingContext2d);
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
                Coordinates {x: width, y: height}
            ),
            position: Default::default(),
            data,
        })
    }
}

impl Animate for Block {
    fn position(&self) -> Position {
        self.position
    }

    fn size(&self) -> Size {
        self.size
    }

    fn animate(&mut self, context: &CanvasRenderingContext2d) {
        let previous = self.position.offset(1, 1);

        let x = self.position.x().into();
        let y = self.position.y().into();
        let width = self.size.x();
        let height = self.size.y();

        context.clear_rect(previous.x().into(), previous.y().into(), width.into(), height.into());

        let image = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&self.data),
            width,
            height,
        ).expect("ImageData");

        context.put_image_data(&image, x, y)
            .expect("put_image_data");
    }
}
