use std::{fmt::Debug, mem, sync::Mutex};

enum EntryKind {
    Text(String),
    Button(bool),
    Toggle(bool),
    Slider { value: f32, min: f32, max: f32 },
    Input(String),
}

struct Entry {
    key: String,
    kind: EntryKind,
}

struct PanelState {
    entries: Vec<UiNode>,
    visible: bool,
}

static STATE: Mutex<PanelState> = Mutex::new(PanelState {
    entries: Vec::new(),
    visible: true,
});

enum UiBody {
    Leaf(EntryKind),
    Category(Vec<UiNode>)
}

struct UiNode {
    name: String,
    body: UiBody,
}

fn leaf<'a>(nodes: &'a mut Vec<UiNode>, path: &[&str], make: impl FnOnce() -> EntryKind) -> &'a mut EntryKind {
    let (name, rest) = path.split_first().expect("empty key");
    let i = match nodes.iter().position(|n| n.name == *name) {
        Some(i) => i,
        None => {
            nodes.push(UiNode { name: name.to_string(), body: UiBody::Category(Vec::new()) });
            nodes.len() - 1
        }
    };
    let node = &mut nodes[i];
    if rest.is_empty() {
        if !matches!(node.body, UiBody::Leaf(_)) {
            node.body = UiBody::Leaf(make());
        }
        let UiBody::Leaf(k) = &mut node.body else { unreachable!() };
        k
    } else {
        if !matches!(node.body, UiBody::Category(_)) {
            node.body = UiBody::Category(Vec::new());
        }
        let UiBody::Category(children) = &mut node.body else { unreachable!() };
        leaf(children, rest, make)
    }
}

fn with_entry<T>(key: &str, make_type: impl FnOnce() -> EntryKind, set: impl FnOnce(&mut EntryKind) -> T) -> T {
    let path: Vec<&str> = key.split('/').collect();
    let mut s = STATE.lock().unwrap();

    set(&mut leaf(&mut s.entries, &path, make_type))
}

pub fn text(key: &str, msg: impl Into<String>) {
    with_entry(key, || EntryKind::Text(String::new()), |k| *k = EntryKind::Text(msg.into()));
}

pub fn show(key: &str, item: impl Debug) {
    text(key, format!("{:?}", item));
}

pub fn button(key: &str) -> bool {
    with_entry(key,
        || EntryKind::Button(false),
        |e| match e {
            EntryKind::Button(clicked) => { mem::take(clicked) },
            _ => { *e = EntryKind::Button(false); false }
        }
    )
}

pub fn toggle(key: &str) -> bool {
    with_entry(key,
        || EntryKind::Toggle(false),
        |e| match e {
            EntryKind::Toggle(toggled) => { *toggled },
            _ => { *e = EntryKind::Toggle(false); false }
        }
    )
}

pub fn slider(key: &str, min: f32, max: f32) -> f32 {
    with_entry(key,
        || EntryKind::Slider { value: 0.0, min, max },
        |e| match e {
            EntryKind::Slider { value, .. } => { *value },
            _ => { *e = EntryKind::Slider { value: 0.0, min, max }; 0.0 }
        }
    )
}

pub fn input(key: &str) -> String {
    with_entry(key,
        || EntryKind::Input(String::new()),
        |e| match e {
            EntryKind::Input(input) => input.clone(),
            _ => { *e = EntryKind::Input(String::new()); String::new() }
        }
    )
}

pub fn toggle_visible() {
    let mut s = STATE.lock().unwrap();
    s.visible = !s.visible;
}

fn draw_leaf(ui: &mut egui::Ui, name: &str, kind: &mut EntryKind) {
    match kind {
        EntryKind::Text(msg) => {
            ui.horizontal(|ui| {
                ui.label(name);
                ui.monospace(msg);
            });
        },
        EntryKind::Input(msg) => {
            ui.horizontal(|ui| {
                ui.label(name);
                ui.text_edit_singleline(msg)
            });
        },
        EntryKind::Button(pressed) => {
            *pressed = ui.button(name).clicked()
        },
        EntryKind::Toggle(pressed) => {
            ui.toggle_value(pressed, name);
        },
        EntryKind::Slider { value, min, max } => {
            ui.add(egui::Slider::new(value, *min..=*max).text(name));
        }
    }
}

fn ui_tree(ui: &mut egui::Ui, nodes: &mut [UiNode]) {
    for (name, mut kind) in nodes.iter_mut().filter_map(|node| 
        match node.body { 
            UiBody::Leaf(ref mut k) => Some((node.name.clone(), k)),
            UiBody::Category(_) => None,
        }) {
        draw_leaf(ui, &name, &mut kind);
    }
    for (name, children) in nodes.iter_mut().filter_map(|node|
        match node.body {
            UiBody::Category(ref mut nodes) => Some((node.name.clone(), nodes)),
            UiBody::Leaf(_) => None,
        }) {
        egui::CollapsingHeader::new(name.as_str())
            .default_open(true)
            .show(ui, |ui| ui_tree(ui, children));
    }
}

pub fn ui(ctx: &egui::Context) {
    let mut s = STATE.lock().unwrap();
    let mut visible = s.visible;
    egui::Window::new("debug")
        .open(&mut visible)
        .default_width(200.0)
        .show(ctx, |ui| {
            ui_tree(ui, &mut s.entries);
        });
    s.visible = visible;
}
