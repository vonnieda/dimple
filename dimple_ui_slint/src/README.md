# Adding a UI callback

1. Add the callback to ui/common.slint#AppState. Use the existing ones as a template.
2. Add a matching function to the page controller, e.g. src/ui/settings.rs#settings_set_online.
3. Add a connecting closure to src/ui/app_window_controller.rs#AppWindowController.run.

