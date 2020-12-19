extern crate gio;
extern crate gtk;

use std::rc::Rc;
use std::cell::RefCell;
use std::convert::TryFrom;

use gio::prelude::*;
use gtk::prelude::*;
use gio::{ListStore, AppLaunchContext, DesktopAppInfo};
use gdk::{Display};
use gtk::{FlowBox, ScrolledWindow, SearchEntry, Label, Image, Window, WindowType};

struct LauncherWindow {
    window: Window,
    search: SearchEntry,
    scroll: ScrolledWindow,
    flowbox: FlowBox,
    model: ListStore,
    details: ApplicationDetails,
}

impl LauncherWindow {
    fn new() -> LauncherWindow {
        let window = Window::new(WindowType::Toplevel);
        window.set_default_size(600, 500);

        window.set_resizable(false);
        window.set_decorated(false);

        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        window.add(&container);

        let search = SearchEntry::new();
        container.add(&search);

        let scroll = ScrolledWindow::new::<gtk::Adjustment, gtk::Adjustment>(None, None);
        scroll.set_min_content_width(600);
        scroll.set_min_content_height(500);
        container.add(&scroll);

        let flowbox = FlowBox::new();
        flowbox.set_activate_on_single_click(false);
        flowbox.set_valign(gtk::Align::Start);
        scroll.add(&flowbox);

        let model = ListStore::new(DesktopAppInfo::static_type());
        flowbox.bind_model(
            Some(&model),
            |item| {
                let info = item.downcast_ref::<DesktopAppInfo>().expect("Model data of wrong type");
                let widget = create_launcher_entry(info).expect("Could not create widget");
                return widget;
            }
        );

        let details = ApplicationDetails::new();
        container.add(&details.container);

        LauncherWindow {
            window,
            search,
            scroll,
            flowbox,
            model,
            details,
        }
    }

    fn get_selected_desktop_app_info(&self) -> Option<DesktopAppInfo> {
        for child in self.flowbox.get_selected_children() {
            if let Ok(idx) = u32::try_from(child.get_index()) {
                return self.model.get_object(idx).map(|obj| {
                    return obj.downcast::<DesktopAppInfo>()
                        .expect("Model only contains DesktopAppInfo");
                });
            }
        }
        return None;
    }

    fn show_details(&self) {
        self.scroll.hide();
        self.details.container.show_all();
    }

    fn get_flowbox(&self) -> &FlowBox {
        if self.scroll.get_visible() {
            return &self.flowbox;
        }
        return & self.details.actioncontainer;
    }

    fn focus_selected(&self) {
        let flowbox = self.get_flowbox();
        for child in flowbox.get_selected_children() {
            child.grab_focus();
            return;
        }
    }

    fn navigate(&self, dir: gtk::DirectionType) {
        if self.search.has_focus() {
            self.focus_selected();
        }
        self.get_flowbox().child_focus(dir);
    }
}

struct ApplicationDetails {
    appinfo: Option<DesktopAppInfo>,
    container: gtk::Box,
    icon: Image,
    label: Label,
    actioncontainer: FlowBox,
}

impl ApplicationDetails {
    fn new() -> ApplicationDetails {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        let infocontainer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let icon = Image::new();
        icon.set_pixel_size(128);
        infocontainer.add(&icon);
        let label = Label::new(Some("HELLO"));
        infocontainer.add(&label);
        container.add(&infocontainer);

        let actioncontainer = FlowBox::new();
        container.add(&actioncontainer);

        return ApplicationDetails {
            appinfo: None,
            container,
            icon,
            label,
            actioncontainer,
        };
    }

    fn set_desktop_app_info(&mut self, info: DesktopAppInfo) -> Result<(), Box<dyn std::error::Error>> {
        self.appinfo = Some(info.clone());
        let name = info.get_display_name().ok_or("Missing display name")?;
        self.label.set_text(name.as_str());
        match info.get_icon() {
            Some(gicon) => self.icon.set_from_gicon(&gicon, gtk::IconSize::unscaled()),
            None => self.icon.set_from_icon_name(Some("application-x-executable"), gtk::IconSize::unscaled()),
        }
        self.actioncontainer.foreach(|child| {
            self.actioncontainer.remove(child);
        });
        for action in info.list_actions() {
            let name = info.get_action_name(action.as_str()).unwrap_or(action);
            let label = Label::new(Some(name.as_str()));
            self.actioncontainer.add(&label);
        }
        return Ok(());
    }
}

fn create_launcher_entry(info: &DesktopAppInfo) -> Result<gtk::Widget, Box<dyn std::error::Error>> {
    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    container.set_size_request(128 + 64, 128 + 64);

    let icon = Image::new();
    icon.set_pixel_size(128);
    match info.get_icon() {
        Some(gicon) => icon.set_from_gicon(&gicon, gtk::IconSize::unscaled()),
        None => icon.set_from_icon_name(Some("application-x-executable"), gtk::IconSize::unscaled()),
    }
    container.add(&icon);

    let name = info.get_display_name().ok_or("Missing display name")?;
    let label = Label::new(Some(name.as_str()));
    label.set_max_width_chars(8);
    label.set_ellipsize(pango::EllipsizeMode::End);
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

    let launcher = Rc::new(RefCell::new(LauncherWindow::new()));

    {
        let _launcher = launcher.clone();
        launcher.borrow().search.connect_search_changed(move |search| {
            let __launcher = _launcher.borrow();
            let query = search.get_text();
            println!("Searching {}", query);
            let result = DesktopAppInfo::search(query.as_str());
            __launcher.model.remove_all();
            for r in result {
                if let Some(info) = DesktopAppInfo::new(r[0].as_str()) {
                    __launcher.model.append(&info);
                }
            }
            if let Some(first) = __launcher.flowbox.get_child_at_index(0) {
                __launcher.flowbox.select_child(&first);
            }
        });
    }
    {
        let _launcher = launcher.clone();
        launcher.borrow().window.connect_key_press_event(move |_window, event| {
            let mut __launcher = _launcher.borrow_mut();
            if let Some(keyval) = event.get_keyval().name() {
                match keyval.as_str() {
                    "Escape" => {
                        gtk::main_quit();
                        return Inhibit(true);
                    },
                    "space" => {
                        if event.get_state().contains(gdk::ModifierType::CONTROL_MASK) || !__launcher.search.has_focus() {
                            if let Some(info) = __launcher.get_selected_desktop_app_info() {
                                if let Err(e) = __launcher.details.set_desktop_app_info(info) {
                                    println!("Something went wrong {}", e);
                                }
                                __launcher.show_details();
                                if let Some(child) = __launcher.details.actioncontainer.get_child_at_index(0) {
                                    __launcher.details.actioncontainer.select_child(&child);
                                }
                                __launcher.details.actioncontainer.grab_focus();
                            }
                            return Inhibit(true);
                        }
                        return Inhibit(false);
                    },
                    "Left" => {
                        __launcher.navigate(gtk::DirectionType::Left);
                        return Inhibit(true);
                    },
                    "Right" => {
                        __launcher.navigate(gtk::DirectionType::Right);
                        return Inhibit(true);
                    },
                    "Up" => {
                        __launcher.navigate(gtk::DirectionType::Up);
                        return Inhibit(true);
                    },
                    "Down" => {
                        __launcher.navigate(gtk::DirectionType::Down);
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
        launcher.borrow().search.connect_activate(move |_entry| {
            let __launcher = _launcher.borrow();
            let selected = __launcher.flowbox.get_selected_children();
            for child in selected {
                child.activate();
                return;
            }
        });
    }
    {
        let _launcher = launcher.clone();
        launcher.borrow().flowbox.connect_child_activated(move |_flowbox, child| {
            let __launcher = _launcher.borrow();
            if let Ok(idx) = u32::try_from(child.get_index()) {
                if let Some(obj) = __launcher.model.get_object(idx) {
                    let info = obj.downcast::<DesktopAppInfo>().expect("Model only contains DesktopAppInfo");
                    let launchctx = get_launch_context().expect("Launch context is available");
                    if let Err(e) = info.launch_uris(&[], Some(&launchctx)) {
                        println!("Failed to launch: {}", e);
                    }
                    gtk::main_quit();
                }
            }
        });
    }
    {
        let _launcher = launcher.clone();
        launcher.borrow().details.actioncontainer.connect_child_activated(move |_flowbox, child| {
            if let Ok(idx) = usize::try_from(child.get_index()) {
                if let Some(info) = _launcher.borrow().details.appinfo.clone() {
                    let actions = info.list_actions();
                    let launchctx = get_launch_context().expect("Launch context is available");
                    info.launch_action(actions[idx].as_str(), Some(&launchctx));
                    gtk::main_quit();
                }
            }
        });
    }
    launcher.borrow().window.connect_destroy(|_window| {
        gtk::main_quit();
    });
    launcher.borrow().window.show_all();
    launcher.borrow().details.container.hide();

    gtk::main();

    Ok(())
}
