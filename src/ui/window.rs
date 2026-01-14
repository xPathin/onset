use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::desktop_entry::CreateOptions;
use crate::discovery::{discover_applications, discover_autostart_entries};
use crate::model::{Application, AutostartEntry};
use crate::operations::create_autostart_entry;

use super::app_chooser::AppChooserDialog;
use super::autostart_row::create_autostart_row;
use super::entry_dialog::EntryDialog;

pub struct MainWindow {
    pub window: adw::ApplicationWindow,
    entries: Rc<RefCell<Vec<AutostartEntry>>>,
    applications: Rc<RefCell<Vec<Application>>>,
    list_box: gtk4::ListBox,
    toast_overlay: adw::ToastOverlay,
}

impl MainWindow {
    pub fn build(app: &adw::Application) -> adw::ApplicationWindow {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Autostart Manager")
            .default_width(700)
            .default_height(500)
            .build();

        let entries: Rc<RefCell<Vec<AutostartEntry>>> = Rc::new(RefCell::new(Vec::new()));
        let applications: Rc<RefCell<Vec<Application>>> = Rc::new(RefCell::new(Vec::new()));

        let header_bar = adw::HeaderBar::new();

        let add_button = gtk4::Button::builder()
            .icon_name("list-add-symbolic")
            .tooltip_text("Add autostart entry")
            .build();

        let refresh_button = gtk4::Button::builder()
            .icon_name("view-refresh-symbolic")
            .tooltip_text("Refresh")
            .build();

        header_bar.pack_start(&refresh_button);
        header_bar.pack_end(&add_button);

        let search_entry = gtk4::SearchEntry::builder()
            .placeholder_text("Search entries...")
            .hexpand(true)
            .build();

        let search_bar = gtk4::SearchBar::builder()
            .child(&search_entry)
            .search_mode_enabled(true)
            .build();

        let list_box = gtk4::ListBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .css_classes(vec!["boxed-list"])
            .build();

        let scrolled_window = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vexpand(true)
            .child(&list_box)
            .build();

        let empty_status = adw::StatusPage::builder()
            .icon_name("application-x-executable-symbolic")
            .title("No Autostart Entries")
            .description("Click the + button to add an application to autostart")
            .build();

        let stack = gtk4::Stack::new();
        stack.add_named(&scrolled_window, Some("list"));
        stack.add_named(&empty_status, Some("empty"));

        let content_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .spacing(12)
            .build();

        content_box.append(&search_bar);
        content_box.append(&stack);

        let toast_overlay = adw::ToastOverlay::new();
        toast_overlay.set_child(Some(&content_box));

        let toolbar_view = adw::ToolbarView::new();
        toolbar_view.add_top_bar(&header_bar);
        toolbar_view.set_content(Some(&toast_overlay));

        window.set_content(Some(&toolbar_view));

        let main_window = MainWindow {
            window: window.clone(),
            entries: entries.clone(),
            applications: applications.clone(),
            list_box: list_box.clone(),
            toast_overlay: toast_overlay.clone(),
        };

        main_window.load_entries(&stack);
        main_window.load_applications();

        {
            let window_clone = window.clone();
            let entries_clone = entries.clone();
            let list_box_clone = list_box.clone();
            let stack_clone = stack.clone();
            let toast_overlay_clone = toast_overlay.clone();

            refresh_button.connect_clicked(move |_| {
                Self::refresh_entries(
                    &window_clone,
                    &entries_clone,
                    &list_box_clone,
                    &stack_clone,
                    &toast_overlay_clone,
                );
            });
        }

        {
            let window_clone = window.clone();
            let entries_clone = entries.clone();
            let applications_clone = applications.clone();
            let list_box_clone = list_box.clone();
            let stack_clone = stack.clone();
            let toast_overlay_clone = toast_overlay.clone();

            add_button.connect_clicked(move |_| {
                Self::show_add_dialog(
                    &window_clone,
                    &entries_clone,
                    &applications_clone,
                    &list_box_clone,
                    &stack_clone,
                    &toast_overlay_clone,
                );
            });
        }

        {
            let entries_clone = entries.clone();
            let list_box_clone = list_box.clone();

            search_entry.connect_search_changed(move |entry| {
                let query = entry.text().to_string().to_lowercase();
                Self::filter_list(&entries_clone, &list_box_clone, &query);
            });
        }

        window
    }

    fn load_entries(&self, stack: &gtk4::Stack) {
        match discover_autostart_entries() {
            Ok(discovered) => {
                *self.entries.borrow_mut() = discovered;
                self.populate_list();

                if self.entries.borrow().is_empty() {
                    stack.set_visible_child_name("empty");
                } else {
                    stack.set_visible_child_name("list");
                }
            }
            Err(e) => {
                tracing::error!("Failed to discover autostart entries: {}", e);
                self.show_toast(&format!("Failed to load entries: {}", e));
            }
        }
    }

    fn load_applications(&self) {
        match discover_applications() {
            Ok(apps) => {
                *self.applications.borrow_mut() = apps;
            }
            Err(e) => {
                tracing::error!("Failed to discover applications: {}", e);
            }
        }
    }

    fn populate_list(&self) {
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        let entries = self.entries.borrow();
        let entries_clone = self.entries.clone();
        let list_box_clone = self.list_box.clone();
        let toast_overlay_clone = self.toast_overlay.clone();
        let window_clone = self.window.clone();

        for entry in entries.iter() {
            let entries_for_edit = entries_clone.clone();
            let list_box_for_edit = list_box_clone.clone();
            let toast_overlay_for_edit = toast_overlay_clone.clone();
            let window_for_edit = window_clone.clone();
            let stack_for_edit = None::<gtk4::Stack>; // Not needed for edit

            let entries_for_delete = entries_clone.clone();
            let list_box_for_delete = list_box_clone.clone();
            let toast_overlay_for_delete = toast_overlay_clone.clone();

            let row = create_autostart_row(
                entry,
                move |path, _id| {
                    Self::handle_edit(
                        path,
                        &window_for_edit,
                        &entries_for_edit,
                        &list_box_for_edit,
                        stack_for_edit.as_ref(),
                        &toast_overlay_for_edit,
                    );
                },
                move |path, id| {
                    Self::handle_delete(
                        path,
                        &id,
                        &entries_for_delete,
                        &list_box_for_delete,
                        &toast_overlay_for_delete,
                    );
                },
            );
            self.list_box.append(&row);
        }
    }

    fn refresh_entries(
        window: &adw::ApplicationWindow,
        entries: &Rc<RefCell<Vec<AutostartEntry>>>,
        list_box: &gtk4::ListBox,
        stack: &gtk4::Stack,
        toast_overlay: &adw::ToastOverlay,
    ) {
        match discover_autostart_entries() {
            Ok(discovered) => {
                *entries.borrow_mut() = discovered;

                while let Some(child) = list_box.first_child() {
                    list_box.remove(&child);
                }

                let entries_ref = entries.borrow();
                let entries_clone = entries.clone();
                let list_box_clone = list_box.clone();
                let toast_overlay_clone = toast_overlay.clone();
                let stack_clone = stack.clone();
                let window_clone = window.clone();

                for entry in entries_ref.iter() {
                    let entries_for_edit = entries_clone.clone();
                    let list_box_for_edit = list_box_clone.clone();
                    let toast_overlay_for_edit = toast_overlay_clone.clone();
                    let window_for_edit = window_clone.clone();
                    let stack_for_edit = stack_clone.clone();

                    let entries_for_delete = entries_clone.clone();
                    let list_box_for_delete = list_box_clone.clone();
                    let toast_overlay_for_delete = toast_overlay_clone.clone();

                    let row = create_autostart_row(
                        entry,
                        move |path, _id| {
                            MainWindow::handle_edit(
                                path,
                                &window_for_edit,
                                &entries_for_edit,
                                &list_box_for_edit,
                                Some(&stack_for_edit),
                                &toast_overlay_for_edit,
                            );
                        },
                        move |path, id| {
                            MainWindow::handle_delete(
                                path,
                                &id,
                                &entries_for_delete,
                                &list_box_for_delete,
                                &toast_overlay_for_delete,
                            );
                        },
                    );
                    list_box.append(&row);
                }

                if entries_ref.is_empty() {
                    stack.set_visible_child_name("empty");
                } else {
                    stack.set_visible_child_name("list");
                }

                let toast = adw::Toast::new("Entries refreshed");
                toast_overlay.add_toast(toast);
            }
            Err(e) => {
                tracing::error!("Failed to refresh entries: {}", e);
                let toast = adw::Toast::new(&format!("Failed to refresh: {}", e));
                toast_overlay.add_toast(toast);
            }
        }
    }

    fn show_add_dialog(
        window: &adw::ApplicationWindow,
        entries: &Rc<RefCell<Vec<AutostartEntry>>>,
        applications: &Rc<RefCell<Vec<Application>>>,
        list_box: &gtk4::ListBox,
        stack: &gtk4::Stack,
        toast_overlay: &adw::ToastOverlay,
    ) {
        let apps = applications.borrow().clone();
        let dialog = AppChooserDialog::new(window, &apps);

        let window_clone = window.clone();
        let entries_clone = entries.clone();
        let list_box_clone = list_box.clone();
        let stack_clone = stack.clone();
        let toast_overlay_clone = toast_overlay.clone();

        dialog.connect_response(move |selected_app| {
            if let Some(app) = selected_app {
                let options = CreateOptions {
                    icon: app.icon.clone(),
                    comment: app.comment.clone(),
                    ..Default::default()
                };

                match create_autostart_entry(&app.id, &app.name, &app.exec, options) {
                    Ok(_) => {
                        Self::refresh_entries(
                            &window_clone,
                            &entries_clone,
                            &list_box_clone,
                            &stack_clone,
                            &toast_overlay_clone,
                        );
                        let toast = adw::Toast::new(&format!("Added {}", app.name));
                        toast_overlay_clone.add_toast(toast);
                    }
                    Err(e) => {
                        tracing::error!("Failed to create entry: {}", e);
                        let toast = adw::Toast::new(&format!("Failed to add: {}", e));
                        toast_overlay_clone.add_toast(toast);
                    }
                }
            }
        });

        // Wire up custom entry creation
        let window_clone = window.clone();
        let entries_clone2 = entries.clone();
        let list_box_clone2 = list_box.clone();
        let stack_clone2 = stack.clone();
        let toast_overlay_clone2 = toast_overlay.clone();

        dialog.connect_custom(move || {
            Self::show_custom_entry_dialog(
                &window_clone,
                &entries_clone2,
                &list_box_clone2,
                &stack_clone2,
                &toast_overlay_clone2,
            );
        });

        dialog.present();
    }

    fn show_custom_entry_dialog(
        window: &adw::ApplicationWindow,
        entries: &Rc<RefCell<Vec<AutostartEntry>>>,
        list_box: &gtk4::ListBox,
        stack: &gtk4::Stack,
        toast_overlay: &adw::ToastOverlay,
    ) {
        let window_clone = window.clone();
        let entries_clone = entries.clone();
        let list_box_clone = list_box.clone();
        let stack_clone = stack.clone();
        let toast_overlay_clone = toast_overlay.clone();

        let dialog = EntryDialog::new_for_create(window, move |id, name, exec, options| {
            match create_autostart_entry(&id, &name, &exec, options) {
                Ok(_) => {
                    MainWindow::refresh_entries(
                        &window_clone,
                        &entries_clone,
                        &list_box_clone,
                        &stack_clone,
                        &toast_overlay_clone,
                    );
                    let toast = adw::Toast::new(&format!("Created {}", name));
                    toast_overlay_clone.add_toast(toast);
                }
                Err(e) => {
                    tracing::error!("Failed to create entry: {}", e);
                    let toast = adw::Toast::new(&format!("Failed to create: {}", e));
                    toast_overlay_clone.add_toast(toast);
                }
            }
        });

        dialog.present();
    }

    fn filter_list(
        entries: &Rc<RefCell<Vec<AutostartEntry>>>,
        list_box: &gtk4::ListBox,
        query: &str,
    ) {
        let entries_ref = entries.borrow();
        let mut index = 0;

        let mut child = list_box.first_child();
        while let Some(widget) = child {
            if let Some(entry) = entries_ref.get(index) {
                let matches = query.is_empty()
                    || entry.desktop_entry.name.to_lowercase().contains(query)
                    || entry
                        .desktop_entry
                        .comment
                        .as_ref()
                        .map(|c| c.to_lowercase().contains(query))
                        .unwrap_or(false)
                    || entry.desktop_entry.exec.to_lowercase().contains(query);

                widget.set_visible(matches);
            }

            child = widget.next_sibling();
            index += 1;
        }
    }

    fn show_toast(&self, message: &str) {
        let toast = adw::Toast::new(message);
        self.toast_overlay.add_toast(toast);
    }

    fn handle_edit(
        path: PathBuf,
        window: &adw::ApplicationWindow,
        entries: &Rc<RefCell<Vec<AutostartEntry>>>,
        list_box: &gtk4::ListBox,
        stack: Option<&gtk4::Stack>,
        toast_overlay: &adw::ToastOverlay,
    ) {
        // Find the entry by path
        let entry_opt = entries.borrow().iter().find(|e| e.path == path).cloned();

        if let Some(entry) = entry_opt {
            let window_clone = window.clone();
            let entries_clone = entries.clone();
            let list_box_clone = list_box.clone();
            let stack_clone = stack.cloned();
            let toast_overlay_clone = toast_overlay.clone();

            let dialog = EntryDialog::new_for_edit(window, &entry, move |changes| {
                use crate::operations::edit_autostart_entry;

                // Load the current entry again - drop borrow before refresh
                let current_entry = {
                    entries_clone
                        .borrow()
                        .iter()
                        .find(|e| e.path == path)
                        .cloned()
                };

                if let Some(current_entry) = current_entry {
                    let entry_name = current_entry.desktop_entry.name.clone();
                    match edit_autostart_entry(&current_entry, changes) {
                        Ok(_) => {
                            // Refresh the list
                            if let Some(ref stack) = stack_clone {
                                MainWindow::refresh_entries(
                                    &window_clone,
                                    &entries_clone,
                                    &list_box_clone,
                                    stack,
                                    &toast_overlay_clone,
                                );
                            }
                            let toast = adw::Toast::new(&format!("Updated {}", entry_name));
                            toast_overlay_clone.add_toast(toast);
                        }
                        Err(e) => {
                            tracing::error!("Failed to edit entry: {}", e);
                            let toast = adw::Toast::new(&format!("Failed to update: {}", e));
                            toast_overlay_clone.add_toast(toast);
                        }
                    }
                }
            });

            dialog.present();
        }
    }

    fn handle_delete(
        path: PathBuf,
        id: &str,
        entries: &Rc<RefCell<Vec<AutostartEntry>>>,
        list_box: &gtk4::ListBox,
        toast_overlay: &adw::ToastOverlay,
    ) {
        match std::fs::remove_file(&path) {
            Ok(_) => {
                tracing::info!("Deleted autostart entry: {}", path.display());

                // Find the index of the entry to delete BEFORE removing
                let delete_index = { entries.borrow().iter().position(|e| e.path == path) };

                // Remove from entries list
                entries.borrow_mut().retain(|e| e.path != path);

                // Remove the row at the found index
                if let Some(index) = delete_index {
                    if let Some(row) = list_box.row_at_index(index as i32) {
                        list_box.remove(&row);
                    }
                }

                let toast = adw::Toast::new(&format!("Deleted {}", id));
                toast_overlay.add_toast(toast);
            }
            Err(e) => {
                tracing::error!("Failed to delete {}: {}", path.display(), e);
                let toast = adw::Toast::new(&format!("Failed to delete: {}", e));
                toast_overlay.add_toast(toast);
            }
        }
    }
}
