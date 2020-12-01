extern crate gio;
extern crate gtk;

use std::sync::Arc;

use gio::prelude::*;
use gtk::prelude::*;
use gio::{ListStore, DesktopAppInfo};
use gtk::{FlowBox, SearchEntry, Label, Image, Window, WindowType};

struct LauncherWindow {
    window: Window,
    search: SearchEntry,
    model: ListStore,
}

impl LauncherWindow {
    fn new() -> LauncherWindow {
        let window = Window::new(WindowType::Toplevel);
        window.set_default_size(600, 300);

        window.set_resizable(false);
        window.set_decorated(false);

        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        window.add(&container);

        let search = SearchEntry::new();
        container.add(&search);

        let flowbox = FlowBox::new();
        container.add(&flowbox);

        let model = ListStore::new(DesktopAppInfo::static_type());
        flowbox.bind_model(
            Some(&model),
            |item| {
                let info = item.downcast_ref::<DesktopAppInfo>().expect("Model data of wrong type");
                let widget = create_launcher_entry(info).expect("Could not create widget");
                return widget;
            }
        );

        LauncherWindow {
            window: window,
            search: search,
            model: model,
        }
    }
}

fn create_launcher_entry(info: &DesktopAppInfo) -> Result<gtk::Widget, Box<dyn std::error::Error>> {
    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let icon = Image::new();
    icon.set_pixel_size(128);
    match info.get_icon() {
        Some(gicon) => icon.set_from_gicon(&gicon, gtk::IconSize::unscaled()),
        None => icon.set_from_icon_name(Some("application-x-executable"), gtk::IconSize::unscaled()),
    }
    container.add(&icon);

    let name = info.get_display_name().ok_or("Missing display name")?;
    let label = Label::new(Some(name.as_str()));
    container.add(&label);

    container.show_all();
    return Ok(container.upcast::<gtk::Widget>());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if gtk::init().is_err() {
        return Err("Failed to initialize GTK.".into());
    }

    let launcher = Arc::new(LauncherWindow::new());
    {
        let model = launcher.model.clone();
        launcher.search.connect_search_changed(move |search| {
            let query = search.get_text();
            println!("Searching {}", query);
            let result = DesktopAppInfo::search(query.as_str());
            model.remove_all();
            for r in result {
                if let Some(info) = DesktopAppInfo::new(r[0].as_str()) {
                    model.append(&info);
                }
            }
        });
    }
    launcher.window.show_all();

    gtk::main();

    Ok(())
}
