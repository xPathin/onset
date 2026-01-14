use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::model::Application;

type AppCallback = Rc<RefCell<Option<Box<dyn Fn(Option<Application>)>>>>;
type CustomCallback = Rc<RefCell<Option<Box<dyn Fn()>>>>;

pub struct AppChooserDialog {
    window: adw::Window,
    callback: AppCallback,
    custom_callback: CustomCallback,
}

impl AppChooserDialog {
    pub fn new(parent: &adw::ApplicationWindow, applications: &[Application]) -> Self {
        let callback: AppCallback = Rc::new(RefCell::new(None));
        let custom_callback: CustomCallback = Rc::new(RefCell::new(None));

        let window = adw::Window::builder()
            .title("Add Autostart Entry")
            .default_width(400)
            .default_height(500)
            .modal(true)
            .transient_for(parent)
            .build();

        let header_bar = adw::HeaderBar::new();

        let search_entry = gtk4::SearchEntry::builder()
            .placeholder_text("Search applications...")
            .hexpand(true)
            .build();

        let search_bar = gtk4::SearchBar::builder()
            .child(&search_entry)
            .search_mode_enabled(true)
            .build();

        let list_box = gtk4::ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .css_classes(vec!["boxed-list"])
            .build();

        let apps_clone = applications.to_vec();
        for app in &apps_clone {
            let row = Self::create_app_row(app);
            list_box.append(&row);
        }

        let scrolled_window = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vexpand(true)
            .child(&list_box)
            .build();

        let custom_button = gtk4::Button::builder()
            .label("Create Custom Entry")
            .css_classes(vec!["flat"])
            .build();

        let content_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .margin_start(12)
            .margin_end(12)
            .margin_bottom(12)
            .spacing(12)
            .build();

        content_box.append(&search_bar);
        content_box.append(&scrolled_window);
        content_box.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));
        content_box.append(&custom_button);

        let toolbar_view = adw::ToolbarView::new();
        toolbar_view.add_top_bar(&header_bar);
        toolbar_view.set_content(Some(&content_box));

        window.set_content(Some(&toolbar_view));

        let list_box_for_search = list_box.clone();
        {
            let apps = apps_clone.clone();
            search_entry.connect_search_changed(move |entry| {
                let query = entry.text().to_string().to_lowercase();
                let mut index = 0;
                let mut child = list_box_for_search.first_child();

                while let Some(widget) = child {
                    if let Some(app) = apps.get(index) {
                        let matches = query.is_empty() || app.matches_search(&query);
                        widget.set_visible(matches);
                    }
                    child = widget.next_sibling();
                    index += 1;
                }
            });
        }

        // Wire up custom button
        {
            let custom_callback_clone = custom_callback.clone();
            let window_clone = window.clone();
            custom_button.connect_clicked(move |_| {
                if let Some(ref cb) = *custom_callback_clone.borrow() {
                    cb();
                }
                window_clone.close();
            });
        }

        let chooser = AppChooserDialog {
            window: window.clone(),
            callback: callback.clone(),
            custom_callback: custom_callback.clone(),
        };

        {
            let callback_clone = callback.clone();
            let window_clone = window.clone();
            let apps = apps_clone.clone();

            list_box.connect_row_activated(move |_, row| {
                let index = row.index() as usize;
                if let Some(app) = apps.get(index) {
                    if let Some(ref cb) = *callback_clone.borrow() {
                        cb(Some(app.clone()));
                    }
                    window_clone.close();
                }
            });
        }

        chooser
    }

    fn create_app_row(app: &Application) -> adw::ActionRow {
        let row = adw::ActionRow::builder()
            .title(&app.name)
            .activatable(true)
            .build();

        if let Some(ref comment) = app.comment {
            row.set_subtitle(comment);
        }

        let icon_name = app
            .icon
            .as_deref()
            .unwrap_or("application-x-executable-symbolic");

        let icon = gtk4::Image::builder()
            .icon_name(icon_name)
            .pixel_size(32)
            .build();

        row.add_prefix(&icon);

        row
    }

    pub fn connect_response<F: Fn(Option<Application>) + 'static>(&self, callback: F) {
        *self.callback.borrow_mut() = Some(Box::new(callback));
    }

    pub fn connect_custom<F: Fn() + 'static>(&self, callback: F) {
        *self.custom_callback.borrow_mut() = Some(Box::new(callback));
    }

    pub fn present(&self) {
        self.window.present();
    }
}
