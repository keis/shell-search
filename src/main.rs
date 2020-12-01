extern crate gio;
extern crate gtk;
extern crate glib;

use std::sync::Arc;

use gio::prelude::*;
use gtk::prelude::*;
use gio::{ListStore, DesktopAppInfo};
use gtk::{FlowBox, SearchEntry, Label, Window, WindowType};

struct LauncherWindow {
    window: Window,
    search: SearchEntry,
    flowbox: FlowBox,
    model: ListStore,
}

impl LauncherWindow {
    fn new() -> LauncherWindow {
        let window = Window::new(WindowType::Toplevel);

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
            flowbox: flowbox,
            model: model,
        }
    }
}


fn create_launcher_entry(info: &DesktopAppInfo) -> Result<gtk::Widget, Box<dyn std::error::Error>> {
    let name = info.get_display_name().ok_or("Missing display name")?;
    let label = Label::new(Some(name.as_str()));
    return Ok(label.upcast::<gtk::Widget>());
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
