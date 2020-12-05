extern crate gio;
extern crate gtk;

use std::rc::Rc;
use std::convert::TryFrom;

use gio::prelude::*;
use gtk::prelude::*;
use gio::{ListStore, AppLaunchContext, DesktopAppInfo};
use gdk::{Display};
use gtk::{FlowBox, SearchEntry, Label, Image, Window, WindowType};

struct LauncherWindow {
    window: Window,
    search: SearchEntry,
    flowbox: FlowBox,
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
        flowbox.set_activate_on_single_click(false);
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

fn get_launch_context() -> Result<AppLaunchContext, Box<dyn std::error::Error>> {
    let display = Display::get_default().ok_or("No default display")?;
    let launchctx = display.get_app_launch_context().ok_or("No launch context")?;
    return Ok(launchctx.upcast::<AppLaunchContext>());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if gtk::init().is_err() {
        return Err("Failed to initialize GTK.".into());
    }

    let launcher = Rc::new(LauncherWindow::new());
    let launchctx = get_launch_context().expect("Launch context is available");

    {
        let _launcher = launcher.clone();
        launcher.search.connect_search_changed(move |search| {
            let query = search.get_text();
            println!("Searching {}", query);
            let result = DesktopAppInfo::search(query.as_str());
            _launcher.model.remove_all();
            for r in result {
                if let Some(info) = DesktopAppInfo::new(r[0].as_str()) {
                    _launcher.model.append(&info);
                }
            }
            if let Some(first) = _launcher.flowbox.get_child_at_index(0) {
                _launcher.flowbox.select_child(&first);
            }
        });
    }
    {
        let _launcher = launcher.clone();
        launcher.window.connect_key_press_event(move |_window, event| {
            if let Some(keyval) = event.get_keyval().name() {
                match keyval.as_str() {
                    "Escape" => {
                        gtk::main_quit();
                        return Inhibit(true);
                    },
                    "Left" => {
                        _launcher.flowbox.child_focus(gtk::DirectionType::Left);
                        return Inhibit(true);
                    },
                    "Right" => {
                        _launcher.flowbox.child_focus(gtk::DirectionType::Right);
                        return Inhibit(true);
                    }
                    _ => Inhibit(false),
                }
            } else {
                return Inhibit(false);
            }
        });
    }
    {
        let _launcher = launcher.clone();
        launcher.search.connect_activate(move |_entry| {
            let selected = _launcher.flowbox.get_selected_children();
            for child in selected {
                child.activate();
                return;
            }
        });
    }
    {
        let _launcher = launcher.clone();
        launcher.flowbox.connect_child_activated(move |_flowbox, child| {
            if let Ok(idx) = u32::try_from(child.get_index()) {
                if let Some(obj) = _launcher.model.get_object(idx) {
                    let info = obj.downcast::<DesktopAppInfo>().expect("Model only contains DesktopAppInfo");
                    if let Err(e) = info.launch_uris(&[], Some(&launchctx)) {
                        println!("Failed to launch: {}", e);
                    }
                    gtk::main_quit();
                }
            }
        });
    }
    launcher.window.connect_destroy(|_window| {
        gtk::main_quit();
    });
    launcher.window.show_all();

    gtk::main();

    Ok(())
}
