use std::{cell::RefCell, rc::Rc};

use inkview::Event;
use slint::{ComponentHandle, Model};

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

            let model = Rc::new(slint::VecModel::default());
            window.set_model(model.clone().into());

            let undo_stack;
            {
                let model = model.clone();
                undo_stack = Rc::new(RefCell::new(UndoStack::new(move |change| match change {
                    Change::CircleAdded { row } => {
                        let circle = model.row_data(row).unwrap();
                        model.remove(row);
                        Change::CircleRemoved { row, circle }
                    }
                    Change::CircleRemoved { row, circle } => {
                        model.insert(row, circle);
                        Change::CircleAdded { row }
                    }
                    Change::CircleResized { row, old_d } => {
                        let mut circle = model.row_data(row).unwrap();
                        let d = circle.d;
                        circle.d = old_d;
                        model.set_row_data(row, circle);
                        Change::CircleResized { row, old_d: d }
                    }
                })));
            }

            {
                let model = model.clone();
                let undo_stack = undo_stack.clone();
                let window_weak = window.as_weak();
                window.on_background_clicked(move |x, y| {
                    println!("clicked at {x}, {y}");
                    let mut undo_stack = undo_stack.borrow_mut();
                    let window = window_weak.unwrap();

                    model.push(ui::Circle { x, y, d: 30.0 });
                    undo_stack.push(Change::CircleAdded {
                        row: model.row_count() - 1,
                    });

                    window.set_undoable(undo_stack.undoable());
                    window.set_redoable(undo_stack.redoable());
                });
            }

            {
                let undo_stack = undo_stack.clone();
                let window_weak = window.as_weak();
                window.on_undo_clicked(move || {
                    let mut undo_stack = undo_stack.borrow_mut();
                    let window = window_weak.unwrap();
                    undo_stack.undo();
                    window.set_undoable(undo_stack.undoable());
                    window.set_redoable(undo_stack.redoable());
                });
            }

            {
                let undo_stack = undo_stack.clone();
                let window_weak = window.as_weak();
                window.on_redo_clicked(move || {
                    let mut undo_stack = undo_stack.borrow_mut();
                    let window = window_weak.unwrap();
                    undo_stack.redo();
                    window.set_undoable(undo_stack.undoable());
                    window.set_redoable(undo_stack.redoable());
                });
            }

            {
                let model = model.clone();
                let undo_stack = undo_stack.clone();
                let window_weak = window.as_weak();
                window.on_circle_resized(move |row, diameter| {
                    let row = row as usize;
                    let mut undo_stack = undo_stack.borrow_mut();
                    let window = window_weak.unwrap();

                    let mut circle = model.row_data(row).unwrap();
                    let old_d = circle.d;
                    circle.d = diameter;
                    model.set_row_data(row, circle);
                    undo_stack.push(Change::CircleResized { row, old_d });

                    window.set_undoable(undo_stack.undoable());
                    window.set_redoable(undo_stack.redoable());
                });
            }

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

#[allow(clippy::enum_variant_names)]
enum Change {
    CircleAdded { row: usize },
    CircleRemoved { row: usize, circle: ui::Circle },
    CircleResized { row: usize, old_d: f32 },
}

struct UndoStack<F> {
    stack: Vec<Option<Change>>,
    // Everything at and after this is a redo action
    redo_offset: usize,
    undo2redo: F,
}

impl<F> UndoStack<F>
where
    F: Fn(Change) -> Change,
{
    fn new(undo2redo: F) -> Self {
        Self {
            stack: Vec::new(),
            redo_offset: 0,
            undo2redo,
        }
    }

    fn push(&mut self, change: Change) {
        self.stack.truncate(self.redo_offset);
        self.stack.push(Some(change));
        self.redo_offset += 1;
    }

    fn undoable(&self) -> bool {
        self.redo_offset > 0
    }

    fn redoable(&self) -> bool {
        self.redo_offset < self.stack.len()
    }

    fn undo(&mut self) {
        self.redo_offset -= 1;

        let undo = self
            .stack
            .get_mut(self.redo_offset)
            .unwrap()
            .take()
            .unwrap();
        let redo = (self.undo2redo)(undo);
        self.stack[self.redo_offset] = Some(redo);
    }

    fn redo(&mut self) {
        let redo = self
            .stack
            .get_mut(self.redo_offset)
            .unwrap()
            .take()
            .unwrap();
        let undo = (self.undo2redo)(redo);
        self.stack[self.redo_offset] = Some(undo);

        self.redo_offset += 1;
    }
}
