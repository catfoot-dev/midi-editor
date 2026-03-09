use egui::{Id, Modal};
use std::sync::{LazyLock, Mutex};

pub struct MessageBox {
    caption: String,
    message: String,
    is_open: bool,
}

impl Default for MessageBox {
    fn default() -> Self {
        MessageBox {
            caption: "Message".to_string(),
            message: String::new(),
            is_open: false,
        }
    }
}

impl MessageBox {
    const WIDTH: f32 = 200.0;

    /// 일반 정보 메시지를 띄운다.
    pub fn show(&mut self, message: impl Into<String>) {
        self.caption = "OK".to_string();
        self.message = message.into();
        self.is_open = true;
    }

    /// 오류 메시지를 띄운다.
    pub fn error(&mut self, message: impl Into<String>) {
        self.caption = "Error".to_string();
        self.message = message.into();
        self.is_open = true;
    }

    /// 중앙 모달을 그리고 닫힘 상태를 동기화한다.
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        if !self.is_open {
            return;
        }

        let modal = Modal::new(Id::new("MessageBox")).show(ui.ctx(), |ui| {
            ui.set_width(MessageBox::WIDTH);
            ui.heading(&self.caption);
            ui.add_space(8.0);
            ui.label(&self.message);
            ui.add_space(24.0);

            egui::Sides::new().show(
                ui,
                |_ui| {},
                |ui| {
                    if ui.button("OK").clicked() {
                        ui.close();
                    }
                },
            );
        });

        if modal.should_close() {
            self.is_open = false;
        }
    }
}

static MESSAGE_BOX: LazyLock<Mutex<MessageBox>> =
    LazyLock::new(|| Mutex::new(MessageBox::default()));

pub fn get_message_box() -> &'static Mutex<MessageBox> {
    // 앱 전역에서 하나의 메시지 박스를 공유한다.
    &MESSAGE_BOX
}
