struct MessageBox{
    title: String,
    message: String
}

impl eframe::App for MessageBox {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(&self.title);
                ui.label(&self.message);    

                if ui.button("OK").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });
    }
}

pub fn error(title: String, message: String) {
    let options = eframe::NativeOptions{
        viewport: egui::ViewportBuilder::default()
        .with_inner_size([240.0, 80.0])
        .with_resizable(false)
        .with_always_on_top()
        .with_window_type(egui::X11WindowType::Notification)
        .with_taskbar(false),
        ..Default::default()
    };
    let _ = eframe::run_native(
        &title.clone(),
        options,
        Box::new(|_| Ok(Box::new(MessageBox {title, message}))),
    );
}