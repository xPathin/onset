use std::path::PathBuf;

use gtk4::glib;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::desktop_entry::EffectiveState;
use crate::model::AutostartEntry;

pub fn create_autostart_row<E, D>(
    entry: &AutostartEntry,
    on_edit: E,
    on_delete: D,
) -> adw::ActionRow
where
    E: Fn(PathBuf, String) + 'static,
    D: Fn(PathBuf, String) + 'static,
{
    let row = adw::ActionRow::builder()
        .title(&entry.desktop_entry.name)
        .activatable(true)
        .build();

    if let Some(ref comment) = entry.desktop_entry.comment {
        row.set_subtitle(comment);
    } else {
        row.set_subtitle(&entry.base_exec());
    }

    let icon_name = entry
        .desktop_entry
        .icon
        .as_deref()
        .unwrap_or("application-x-executable-symbolic");

    let icon = gtk4::Image::builder()
        .icon_name(icon_name)
        .pixel_size(32)
        .build();
    row.add_prefix(&icon);

    let toggle = gtk4::Switch::builder()
        .valign(gtk4::Align::Center)
        .active(entry.effective_state == EffectiveState::Enabled)
        .build();

    let entry_path = entry.path.clone();
    let entry_id = entry.id.clone();

    toggle.connect_state_set(move |_, state| {
        match crate::operations::set_entry_enabled_by_path(&entry_path, state) {
            Ok(_) => {
                tracing::info!("Toggled {} to {}", entry_id, state);
            }
            Err(e) => {
                tracing::error!("Failed to toggle {}: {}", entry_id, e);
            }
        }

        glib::Propagation::Proceed
    });

    row.add_suffix(&toggle);

    let info_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .valign(gtk4::Align::Center)
        .build();

    if let Some(delay) = entry.delay_seconds() {
        let delay_label = gtk4::Label::builder()
            .label(format!("{}s", delay))
            .css_classes(vec!["dim-label", "caption"])
            .tooltip_text(format!("{} second delay", delay))
            .build();

        let delay_icon = gtk4::Image::builder()
            .icon_name("alarm-symbolic")
            .pixel_size(12)
            .css_classes(vec!["dim-label"])
            .build();

        let delay_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(2)
            .build();
        delay_box.append(&delay_icon);
        delay_box.append(&delay_label);

        info_box.append(&delay_box);
    }

    match entry.effective_state {
        EffectiveState::EnvironmentExcluded => {
            let env_icon = gtk4::Image::builder()
                .icon_name("computer-symbolic")
                .pixel_size(16)
                .tooltip_text("Excluded for current desktop environment")
                .css_classes(vec!["warning"])
                .build();
            info_box.append(&env_icon);
        }
        EffectiveState::TryExecFailed => {
            let warning_icon = gtk4::Image::builder()
                .icon_name("dialog-warning-symbolic")
                .pixel_size(16)
                .tooltip_text("Required binary not found")
                .css_classes(vec!["warning"])
                .build();
            info_box.append(&warning_icon);
        }
        _ => {}
    }

    row.add_suffix(&info_box);

    if entry.effective_state == EffectiveState::Disabled {
        row.add_css_class("dim-label");
    }

    // Add edit button
    let edit_button = gtk4::Button::builder()
        .icon_name("document-edit-symbolic")
        .valign(gtk4::Align::Center)
        .css_classes(vec!["flat", "circular"])
        .tooltip_text("Edit entry")
        .build();

    let edit_path = entry.path.clone();
    let edit_id = entry.id.clone();
    edit_button.connect_clicked(move |_| {
        on_edit(edit_path.clone(), edit_id.clone());
    });

    row.add_suffix(&edit_button);

    // Add delete button
    let delete_button = gtk4::Button::builder()
        .icon_name("user-trash-symbolic")
        .valign(gtk4::Align::Center)
        .css_classes(vec!["flat", "circular"])
        .tooltip_text("Delete entry")
        .build();

    let delete_path = entry.path.clone();
    let delete_id = entry.id.clone();
    delete_button.connect_clicked(move |_| {
        on_delete(delete_path.clone(), delete_id.clone());
    });

    row.add_suffix(&delete_button);

    row
}
