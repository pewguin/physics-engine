use std::sync::Mutex;

use glam::Vec3;

pub struct Snapshot {
    pub verts: Vec<Vec3>,
    pub idx: Vec<usize>,
}

static FINISHED: Mutex<Option<Vec<Snapshot>>> = Mutex::new(None);

pub fn finished(snapshots: Vec<Snapshot>) {
    *FINISHED.lock().unwrap() = Some(snapshots);
}

pub fn take_finished() -> Option<Vec<Snapshot>> {
    FINISHED.lock().unwrap().take()
}

