use web_sys::CanvasRenderingContext2d;

pub type TimeStamp = f64;

pub trait Draw {
    fn draw(&mut self, context: &CanvasRenderingContext2d);
}
