use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::desktop_entry::{CreateOptions, EntryChanges};
use crate::model::AutostartEntry;
use crate::operations::delay::unwrap_delay;

pub struct EntryDialog {
    window: adw::Window,
}

impl EntryDialog {
    pub fn new_for_create<F>(parent: &adw::ApplicationWindow, on_save: F) -> Self
    where
        F: Fn(String, String, String, CreateOptions) + 'static,
    {
        let window = adw::Window::builder()
            .title("Create Autostart Entry")
            .default_width(400)
            .modal(true)
            .transient_for(parent)
            .build();

        let header_bar = adw::HeaderBar::new();

        let cancel_button = gtk4::Button::builder().label("Cancel").build();

        let save_button = gtk4::Button::builder()
            .label("Add")
            .css_classes(vec!["suggested-action"])
            .build();

        header_bar.pack_start(&cancel_button);
        header_bar.pack_end(&save_button);

        let name_row = adw::EntryRow::builder().title("Name").build();

        let command_row = adw::EntryRow::builder().title("Command").build();

        let comment_row = adw::EntryRow::builder().title("Comment").build();

        let delay_row = adw::SpinRow::builder()
            .title("Startup Delay")
            .subtitle("Seconds to wait before starting")
            .adjustment(&gtk4::Adjustment::new(0.0, 0.0, 300.0, 1.0, 10.0, 0.0))
            .build();

        let terminal_row = adw::SwitchRow::builder().title("Run in Terminal").build();

        let preferences_group = adw::PreferencesGroup::builder()
            .title("Entry Settings")
            .build();

        preferences_group.add(&name_row);
        preferences_group.add(&command_row);
        preferences_group.add(&comment_row);
        preferences_group.add(&delay_row);
        preferences_group.add(&terminal_row);

        let content_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .spacing(12)
            .build();

        content_box.append(&preferences_group);

        let toolbar_view = adw::ToolbarView::new();
        toolbar_view.add_top_bar(&header_bar);
        toolbar_view.set_content(Some(&content_box));

        window.set_content(Some(&toolbar_view));

        {
            let window_clone = window.clone();
            cancel_button.connect_clicked(move |_| {
                window_clone.close();
            });
        }

        {
            let window_clone = window.clone();
            let name_row_clone = name_row.clone();
            let command_row_clone = command_row.clone();
            let comment_row_clone = comment_row.clone();
            let delay_row_clone = delay_row.clone();
            let terminal_row_clone = terminal_row.clone();

            save_button.connect_clicked(move |_| {
                let name = name_row_clone.text().to_string();
                let command = command_row_clone.text().to_string();
                let comment = comment_row_clone.text().to_string();
                let delay = delay_row_clone.value() as u32;
                let terminal = terminal_row_clone.is_active();

                if name.is_empty() || command.is_empty() {
                    return;
                }

                let id = name
                    .to_lowercase()
                    .chars()
                    .map(|c| if c.is_alphanumeric() { c } else { '-' })
                    .collect::<String>();

                let options = CreateOptions {
                    comment: if comment.is_empty() {
                        None
                    } else {
                        Some(comment)
                    },
                    delay_seconds: delay,
                    terminal,
                    ..Default::default()
                };

                on_save(id, name, command, options);
                window_clone.close();
            });
        }

        EntryDialog { window }
    }

    pub fn new_for_edit<F>(
        parent: &adw::ApplicationWindow,
        entry: &AutostartEntry,
        on_save: F,
    ) -> Self
    where
        F: Fn(EntryChanges) + 'static,
    {
        let window = adw::Window::builder()
            .title("Edit Autostart Entry")
            .default_width(400)
            .modal(true)
            .transient_for(parent)
            .build();

        let header_bar = adw::HeaderBar::new();

        let cancel_button = gtk4::Button::builder().label("Cancel").build();

        let save_button = gtk4::Button::builder()
            .label("Save")
            .css_classes(vec!["suggested-action"])
            .build();

        header_bar.pack_start(&cancel_button);
        header_bar.pack_end(&save_button);

        let (base_exec, current_delay) = unwrap_delay(&entry.desktop_entry.exec);

        let name_row = adw::EntryRow::builder()
            .title("Name")
            .text(&entry.desktop_entry.name)
            .build();

        let command_row = adw::EntryRow::builder()
            .title("Command")
            .text(&base_exec)
            .build();

        let comment_row = adw::EntryRow::builder()
            .title("Comment")
            .text(entry.desktop_entry.comment.as_deref().unwrap_or(""))
            .build();

        let delay_row = adw::SpinRow::builder()
            .title("Startup Delay")
            .subtitle("Seconds to wait before starting")
            .adjustment(&gtk4::Adjustment::new(
                current_delay.unwrap_or(0) as f64,
                0.0,
                300.0,
                1.0,
                10.0,
                0.0,
            ))
            .build();

        let terminal_row = adw::SwitchRow::builder()
            .title("Run in Terminal")
            .active(entry.desktop_entry.terminal)
            .build();

        let preferences_group = adw::PreferencesGroup::builder()
            .title("Entry Settings")
            .build();

        preferences_group.add(&name_row);
        preferences_group.add(&command_row);
        preferences_group.add(&comment_row);
        preferences_group.add(&delay_row);
        preferences_group.add(&terminal_row);

        let content_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .spacing(12)
            .build();

        content_box.append(&preferences_group);

        let toolbar_view = adw::ToolbarView::new();
        toolbar_view.add_top_bar(&header_bar);
        toolbar_view.set_content(Some(&content_box));

        window.set_content(Some(&toolbar_view));

        {
            let window_clone = window.clone();
            cancel_button.connect_clicked(move |_| {
                window_clone.close();
            });
        }

        let original_name = entry.desktop_entry.name.clone();
        let original_exec = base_exec.clone();
        let original_comment = entry.desktop_entry.comment.clone();
        let original_delay = current_delay;
        let original_terminal = entry.desktop_entry.terminal;

        {
            let window_clone = window.clone();

            save_button.connect_clicked(move |_| {
                let new_name = name_row.text().to_string();
                let new_exec = command_row.text().to_string();
                let new_comment = comment_row.text().to_string();
                let new_delay = delay_row.value() as u32;
                let new_terminal = terminal_row.is_active();

                let mut changes = EntryChanges::default();

                if new_name != original_name {
                    changes.name = Some(new_name);
                }
                if new_exec != original_exec {
                    changes.exec = Some(new_exec);
                }
                if Some(&new_comment) != original_comment.as_ref() {
                    changes.comment = Some(new_comment);
                }
                if Some(new_delay) != original_delay {
                    changes.delay_seconds = Some(new_delay);
                }
                if new_terminal != original_terminal {
                    changes.terminal = Some(new_terminal);
                }

                on_save(changes);
                window_clone.close();
            });
        }

        EntryDialog { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
