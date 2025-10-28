use inkview::bindings::APPLICATION_ATTRIBUTE_APPLICATION_READER;
use inkview::{bindings, Event};
use std::ffi::{c_int, CString};

fn main() {
    let iv = Box::leak(Box::new(inkview::load())) as &_;
    const FONT_SIZE: c_int = 42;

    inkview::iv_main(&iv, move |event| {
        match event {
            Event::Init => unsafe {
                iv.SetCurrentApplicationAttribute(APPLICATION_ATTRIBUTE_APPLICATION_READER, 1);

                let font_name = CString::new("LiberationSans").unwrap();
                let text = CString::new("Hello world!").unwrap();

                let font = iv.OpenFont(font_name.as_ptr(), FONT_SIZE, 0);
                iv.ClearScreen();

                iv.SetFont(font, bindings::BLACK as c_int);
                iv.DrawLine(
                    25,
                    iv.ScreenHeight() - 25,
                    iv.ScreenWidth() - 25,
                    iv.ScreenHeight() - 25,
                    0x00666666,
                );
                iv.FillArea(
                    50,
                    250,
                    iv.ScreenWidth() - 50 * 2,
                    iv.ScreenHeight() - 250 * 2,
                    0x00E0E0E0,
                );
                iv.FillArea(
                    100,
                    300,
                    iv.ScreenWidth() - 100 * 2,
                    iv.ScreenHeight() - 300 * 2,
                    0x00A0A0A0,
                );
                iv.DrawTextRect(
                    0,
                    iv.ScreenHeight() / 2 - FONT_SIZE / 2,
                    iv.ScreenWidth(),
                    FONT_SIZE,
                    text.as_ptr(),
                    bindings::ALIGN_CENTER as c_int,
                );

                // Copies the buffer to the real screen
                iv.FullUpdate();

                iv.CloseFont(font);
            },
            Event::KeyDown { .. } => unsafe {
                iv.CloseApp();
            },
            _ => {}
        }
        Some(())
    });
}
