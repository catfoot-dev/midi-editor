use std::sync::{LazyLock, Mutex};
use egui::{Id, Modal};

pub struct MessageBox {
    caption: &'static str,
    message: &'static str,
    is_open: bool,
}

impl Default for MessageBox {
    fn default() -> Self {
        MessageBox {
            caption: "Message",
            message: "",
            is_open: false,
        }
    }
}

impl MessageBox {
    const WIDTH: f32 = 200.0;

    pub fn show(&mut self, message: &'static str) {
        self.caption = "OK";
        self.message = message;
        self.is_open = true;
    }

    pub fn error(&mut self, message: &'static str) {
        self.caption = "Error";
        self.message = message;
        self.is_open = true;
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        if self.is_open == false { return; }

        let modal = Modal::new(Id::new("MessageBox"))
            .show(ui.ctx(), |ui| {
                ui.set_width(MessageBox::WIDTH);
                ui.heading(self.caption);
                ui.add_space(8.0);
                ui.label(self.message);
                ui.add_space(24.0);

                egui::Sides::new().show(ui, |_ui| {}, |ui| {
                    if ui.button("OK").clicked() { ui.close(); }
                });
            });

        if modal.should_close() { self.is_open = false; }
    }
}

static MESSAGE_BOX: LazyLock<Mutex<MessageBox>> = LazyLock::new(|| {
    Mutex::new(MessageBox::default())
});

pub fn get_message_box() -> &'static Mutex<MessageBox> {
    &MESSAGE_BOX
}
