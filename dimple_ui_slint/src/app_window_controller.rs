slint::include_modules!();

pub struct AppWindowController {
    ui: AppWindow,
}

impl Default for AppWindowController {
    fn default() -> Self {
        Self { ui: AppWindow::new().unwrap() }
    }
}

impl AppWindowController {
    pub fn run(&self) -> Result<(), slint::PlatformError> {
        let ui_handle = self.ui.as_weak();
        self.ui.global::<Navigator>().on_navigate(move |url| {
            let _ = ui_handle.unwrap();
            dbg!(url);
        });

        self.ui.run()
    }
}

