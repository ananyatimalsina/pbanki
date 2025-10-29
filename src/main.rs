use std::{rc::Rc};

use inkview::Event;
use slint::{ComponentHandle};

mod ui {
    slint::include_modules!();
}

fn main() {
    let iv = Box::leak(Box::new(inkview::load())) as &_;

    let (evt_tx, evt_rx) = std::sync::mpsc::channel();

    std::thread::spawn({
        move || {
            if evt_rx.recv().unwrap() != Event::Init {
                panic!("expected init event first");
            }

            // I hope this thing lives as long as the process
            let screen = inkview::screen::Screen::new(iv);

            let display = inkview_slint::Backend::new(screen, evt_rx);

            slint::platform::set_platform(Box::new(display)).unwrap();

            let window = Rc::new(ui::MainWindow::new().unwrap());

            window.run().unwrap();
        }
    });

    inkview::iv_main(iv, {
        move |evt| {
            // println!("got evt: {:?}", evt);

            if evt_tx.send(evt).is_err() {
                unsafe {
                    iv.CloseApp();
                }
            }

            Some(())
        }
    })
}
