use crate::components::Transform;

#[derive(Copy, Clone)]
pub struct Parent {
    pub parent: hecs::Entity,
    pub local_transform: Transform,
}

pub fn update_children(world: &mut hecs::World) {
    for (_entity, (child, mut transform)) in &mut world.query::<(&Parent, &mut Transform)>() {
        let parent_transform = world.get_mut::<Transform>(child.parent).unwrap();
        // TODO: Make a real transformation
        //transform.isometry.translation.vector = parent_transform.isometry.transform_vector(&child.local_transform.isometry.translation.vector);
        transform.isometry.translation = parent_transform.isometry.translation;
        transform.isometry.rotation =
            parent_transform.isometry.rotation * child.local_transform.isometry.rotation;
    }
}
