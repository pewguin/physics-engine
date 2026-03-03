use std::collections::{HashMap, HashSet};
use glam::{EulerRot, Quat, Vec3};
use crate::physics::collider::{ColliderShape, CollisionResult, collides_with};
use crate::physics::mesh::Mesh;
use crate::physics::rigid_body::{CollisionType, RigidBody};
use crate::physics::world::World;

pub struct SpatialGrid {
    cell_size: f32,
    dynamic_grid: HashMap<(i32, i32, i32), Vec<u32>>,
    static_grid: HashMap<(i32, i32, i32), Vec<u32>>,
    dynamic_objects: Vec<u32>,
    static_objects: Vec<u32>,
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            dynamic_grid: HashMap::new(),
            static_grid: HashMap::new(),
            dynamic_objects: Vec::new(),
            static_objects: Vec::new(),
        }
    }
    pub fn add_object(&mut self, id: u32, object_static: bool) {
        if object_static {
            self.static_objects.push(id);
        } else {
            self.dynamic_objects.push(id);
        }
    }
    pub fn calculate_all_static_object_occupancies(&mut self, world: &World) {
        for id in self.static_objects.iter().copied().collect::<Vec<u32>>() {
            self.calculate_object_occupancy(world.get_transformed_collider(id).unwrap(), id, true);
        }
    }
    pub fn calculate_dynamic_object_occupancies(&mut self, world: &World) {
        for id in self.dynamic_objects.iter().copied().collect::<Vec<u32>>() {
            self.calculate_object_occupancy(world.get_transformed_collider(id).unwrap(), id, false);
        }
    }
    fn calculate_object_occupancy(&mut self, collider: Box<dyn ColliderShape>, index: u32, object_static: bool) {
        let aabb = collider.get_aabb();
        let min_idx = (aabb.min / self.cell_size).floor().as_ivec3();
        let max_idx = (aabb.max / self.cell_size).floor().as_ivec3();
        for x in min_idx.x..=max_idx.x {
            for y in min_idx.y..=max_idx.y {
                for z in min_idx.z..=max_idx.z {
                    let grid;
                    if object_static {
                        grid = &mut self.static_grid;
                    } else {
                        grid = &mut self.dynamic_grid;
                    }
                    if let Some(occupant_vec) = grid.get_mut(&(x, y, z)) {
                        occupant_vec.push(index);
                    } else {
                        grid.insert((x, y, z), vec![index]);
                    }
                }
            }
        }
    }
    pub fn calculate_collisions(&self, world: &mut World) {
        let mut seen_pairs = HashSet::new();
        for (cell, dyn_ids) in self.dynamic_grid.iter() {
            let mut ids = dyn_ids.clone();
            if let Some(stat_ids) = self.static_grid.get(cell) {
                ids.extend(stat_ids);
            }
            for i in 0..ids.len() {
                for j in i+1..ids.len() {
                    let a = ids[i];
                    let b = ids[j];

                    let key = if a < b { (a, b) } else { (b, a) };
                    if seen_pairs.insert(key) {
                        let col_a =  world.get_transformed_collider(a).unwrap();
                        let col_b = world.get_transformed_collider(b).unwrap();
                        let mut process_collision = |vec: Vec3| {
                            if let Some(mut a_rb) = world.rigid_bodies.remove(&a) {
                                if let Some(b_rb) = world.rigid_bodies.get_mut(&b) {
                                    a_rb.collided(&CollisionType::OtherRigidBody(b_rb));
                                    b_rb.collided(&CollisionType::OtherRigidBody(&a_rb));
                                } else {
                                    a_rb.collided(&CollisionType::Static);
                                }
                                world.rigid_bodies.insert(a, a_rb);
                            } else if let Some(b_rb) = world.rigid_bodies.get_mut(&b) {
                                b_rb.collided(&CollisionType::Static);
                            }
                        };
                        match collides_with(col_a, col_b) {
                            CollisionResult::Ok(vec) => {
                                process_collision(vec);
                            }
                            CollisionResult::NoConvergence(vec) => {
                                process_collision(vec); // panic!("No convergence.");
                            }
                            CollisionResult::NoCollision => {}
                        }
                    }
                }
            }
        }
    }
    fn clear_dynamic_grid(&mut self) {
        self.dynamic_grid.clear();
    }
    pub fn recalculate_grid_and_collisions(&mut self, world: &mut World) {
        self.clear_dynamic_grid();
        self.calculate_dynamic_object_occupancies(world);
        self.calculate_collisions(world);
    }
}
