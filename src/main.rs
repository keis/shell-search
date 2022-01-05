extern crate gio;
extern crate gtk;

use std::rc::Rc;
use std::cell::RefCell;
use std::convert::TryFrom;

use gio::prelude::*;
use gtk::prelude::*;
use gio::{ListModel, ListStore, AppLaunchContext, AppInfo, DesktopAppInfo};
use gdk::{Display};
use gtk::{FlowBox, ScrolledWindow, SearchEntry, Label, Image, Window, WindowType};

use gtk_layer_shell_rs as gtk_layer_shell;

struct LauncherWindow {
    window: Window,
    search: SearchEntry,
    scroll: ScrolledWindow,
    flowbox: FlowBox,
    model: ListModel,
    details: ApplicationDetails,
}

impl LauncherWindow {
    fn new(model: ListModel) -> LauncherWindow {
        let window = Window::new(WindowType::Toplevel);
        setup_layer(&window);
        window.set_default_size(600, 500);
        window.set_resizable(false);
        window.set_decorated(false);
        window.set_opacity(0.9);

        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        window.add(&container);

        let search = SearchEntry::new();
        container.add(&search);

        let scroll = ScrolledWindow::new::<gtk::Adjustment, gtk::Adjustment>(None, None);
        scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scroll.set_vexpand(true);
        container.add(&scroll);

        let flowbox = FlowBox::new();
        flowbox.set_activate_on_single_click(false);
        flowbox.set_valign(gtk::Align::Start);
        flowbox.set_homogeneous(true);
        scroll.add(&flowbox);

        flowbox.bind_model(
            Some(&model),
            |item| {
                let info = item.downcast_ref::<AppInfo>().expect("Model data of wrong type");
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

    fn get_selected_desktop_app_info(&self) -> Option<AppInfo> {
        for child in self.flowbox.get_selected_children() {
            if let Ok(idx) = u32::try_from(child.get_index()) {
                return self.model.get_object(idx).map(|obj| {
                    return obj.downcast::<AppInfo>()
                        .expect("Model only contains AppInfo");
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
    appinfo: Option<AppInfo>,
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

    fn set_desktop_app_info(&mut self, info: AppInfo) -> Result<(), Box<dyn std::error::Error>> {
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
        if let Ok(desktopinfo) = info.downcast::<DesktopAppInfo>() {
            for action in desktopinfo.list_actions() {
                let name = desktopinfo.get_action_name(action.as_str()).unwrap_or(action);
                let label = Label::new(Some(name.as_str()));
                self.actioncontainer.add(&label);
            }
        }
        return Ok(());
    }
}

fn create_launcher_entry(info: &AppInfo) -> Result<gtk::Widget, Box<dyn std::error::Error>> {
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

fn setup_layer(window: &gtk::Window) {
    let window = window.clone();
    gtk_layer_shell::init_for_window(&window);
    gtk_layer_shell::set_layer(&window, gtk_layer_shell::Layer::Overlay);
    gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Top, 200);
    gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Bottom, 200);
    gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Left, 400);
    gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Right, 400);
    gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Left, true);
    gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Right, true);
    gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Top, true);
    gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Bottom, true);
    gtk_layer_shell::set_exclusive_zone(&window, -1);
    gtk_layer_shell::set_keyboard_interactivity(&window, true);
}

fn filter_model<P: Fn(&glib::Object) -> bool>(base: &ListStore, filtered: ListStore, predicate: P) {
    let mut i = 0;
    let mut j = 0;
    while let Some(baseobj) = base.get_object(i) {
        if predicate(&baseobj) {
            if let Some(filteredobj) = filtered.get_object(j) {
                if filteredobj != baseobj {
                    filtered.insert(j, &baseobj);
                }
            } else {
                filtered.insert(j, &baseobj);
            }
            j = j + 1;
        } else {
            if let Some(filteredobj) = filtered.get_object(j) {
                if filteredobj == baseobj {
                    filtered.remove(j);
                }
            }
        }
        i = i + 1;
    }
}

fn appinfo_match(info: &AppInfo, query: &str) -> bool {
    if !info.should_show() {
        return false;
    }
    let lcquery = query.to_lowercase();
    if let Some(name) = info.get_display_name() {
        let lcname = name.as_str().to_lowercase();
        if let Some(_idx) = lcname.find(&lcquery) {
            return true;
        }
    }
    if let Some(desc) = info.get_description() {
        let lcdesc = desc.as_str().to_lowercase();
        if let Some(_idx) = lcdesc.find(&lcquery) {
            return true;
        }
    }
    if let Some(desktopinfo) = info.downcast_ref::<DesktopAppInfo>() {
        for keyword in desktopinfo.get_keywords() {
            let lckeyword = keyword.as_str().to_lowercase();
            if let Some(_idx) = lckeyword.find(&lcquery) {
                return true;
            }
        }
    }
    return false;
}

fn apply_search(base: &ListStore, filtered: ListStore, query: &str) {
    filter_model(base, filtered, |obj| {
        let info = obj.downcast_ref::<AppInfo>().expect("Model of AppInfo");
        return appinfo_match(info, &query);
    });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if gtk::init().is_err() {
        return Err("Failed to initialize GTK.".into());
    }

    let model = ListStore::new(AppInfo::static_type());

    for r in AppInfo::get_all() {
        model.append(&r);
    }

    let filtered = ListStore::new(AppInfo::static_type());
    apply_search(&model, filtered.clone(), "");

    let launcher = Rc::new(RefCell::new(
        LauncherWindow::new(
            filtered.clone().dynamic_cast::<ListModel>().expect("Can cast into interface")
        )
    ));

    {
        let _launcher = launcher.clone();
        launcher.borrow().search.connect_search_changed(move |search| {
            let __launcher = _launcher.borrow();
            let searchtext = search.get_text();
            let query = searchtext.as_str();
            apply_search(&model, filtered.clone(), query);
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
                    let info = obj.downcast::<AppInfo>().expect("Model only contains AppInfo");
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
                    if let Ok(desktopinfo) = info.downcast::<DesktopAppInfo>() {
                        let actions = desktopinfo.list_actions();
                        let launchctx = get_launch_context().expect("Launch context is available");
                        desktopinfo.launch_action(actions[idx].as_str(), Some(&launchctx));
                        gtk::main_quit();
                    }
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
