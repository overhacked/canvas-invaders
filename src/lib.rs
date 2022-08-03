mod graphics;
mod entities;
mod geom;

use std::{rc::Rc, cell::RefCell, sync::mpsc};

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::console;

use crate::graphics::TimeStamp;
use crate::geom::Distance;

const MARGIN_X: Distance = 30.0;
const MARGIN_Y: Distance = 48.0;

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
    let canvas_width = Distance::from(canvas.width());
    let canvas_height = Distance::from(canvas.height());

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
    let mut enemies = entities::Fleet::new(4, 6, MARGIN_Y, MARGIN_X, canvas_width - MARGIN_X);
    let mut ship = entities::Ship::new(0.5, canvas_height - MARGIN_Y, MARGIN_X, canvas_width - MARGIN_X);
    let mut last_ts = window.performance().unwrap().now();

    let closure_inner: Closure<dyn FnMut(TimeStamp)> = Closure::new(move |ts: TimeStamp| {
        match key_receiver.try_recv() {
            Ok(evt) => {
                let evt_type = evt.type_();
                console::log_1(&format!("Key event: {} {} ({})", evt_type, evt.key(), evt.key_code()).into());
                match evt.key().as_str() {
                    "a"|"ArrowLeft" => {
                        ship.direction = if evt_type == "keydown" { entities::Direction::Left } else { entities::Direction::Stopped };
                    },
                    "d"|"ArrowRight" => {
                        ship.direction = if evt_type == "keydown" { entities::Direction::Right } else { entities::Direction::Stopped };
                    },
                    _ => {}, // Ignore
                }
            },
            Err(mpsc::TryRecvError::Empty) => {}, // OK, no keys pressed
            Err(err) => {
                console::log_1(&format!("Failed to receive key event, {}", err).into());
            },
        }
        context.clear_rect(0.0, 0.0, canvas_width, canvas_height);

        let ts_offset = ts - last_ts;
        last_ts = ts;
        // TODO: consolidate game entities into one top-level struct that can have a single
        // `.animate()` called
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
