use std::sync::Mutex;

use glam::Vec3;

use crate::rendering::vertex::Vertex;

pub enum DrawOrder {
    Wireframe(Vec<Vec3>),
}

struct DrawState {
    orders: Vec<DrawOrder>
}

static STATE: Mutex<DrawState> = Mutex::new(DrawState {
    orders: Vec::new(),
});

pub fn draw_wireframe(verts: Vec<Vec3>) {
    let mut s = STATE.lock().unwrap();
    s.orders.push(DrawOrder::Wireframe(verts));
}

pub fn draw_wireframe_indexed(verts: Vec<Vec3>, idxs: Vec<usize>) {
    let verts: Vec<Vec3> = idxs.iter()
        .map(|idx| {
            verts[*idx as usize]
        })
        .collect();
    draw_wireframe(verts);
}

pub fn get_orders() -> Vec<DrawOrder> {
    let mut s = STATE.lock().unwrap();

    return std::mem::take(&mut s.orders)
}
