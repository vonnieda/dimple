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
            dbg!(&url);
            let ui = ui_handle.unwrap();
            if url.starts_with("dimple://home") {
                ui.set_page(0);
            }
            else if url.starts_with("dimple://artists") {
                ui.set_page(1);
            }
            else if url.starts_with("dimple://releases") {
                ui.set_page(2);
            }
        });

        self.ui.run()
    }
}

