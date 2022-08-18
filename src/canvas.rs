use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub struct Canvas {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
}

impl Canvas {
    fn context(&self) -> &CanvasRenderingContext2d {
        &self.context
    }

    fn x(&self, x: f64) -> f64 {
        self.canvas.width() * x.clamp(0.0, 1.0)
    }

    fn y(&self, y: f64) -> f64 {
        self.canvas.height() * y.clamp(0.0, 1.0)
    }
}

impl From<HtmlCanvasElement> for Canvas {
    fn from(canvas: HtmlCanvasElement) -> Self {
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();
        Self {
            canvas,
            context,
        }
    }
}
