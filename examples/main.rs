use eframe::{Frame, NativeOptions};
use egui::Context;
use egui_svg::SVG;

fn main() -> eframe::Result {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native("SVG Example", options, Box::new(|_cc| Ok(Box::new(MyApp))))
}

struct MyApp;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        ctx.set_debug_on_hover(true);

        let test = include_bytes!("./test.svg");
        egui::CentralPanel::default().show(ctx, |ui| {
            let svg = SVG::new(test).expect("failed to load test SVG");
            ui.add(svg.with_size(ui.available_size()));
        });
    }
}
