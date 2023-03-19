use bevy::prelude::*;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ColliderSettings { show_debugs: false })
            .add_system(colliders_position_update_system)
            .add_system(collider_debug_switch_system)
            .add_system(collider_added_debug_system)
            .add_system(collider_debug_update_system);
    }
}

#[derive(Resource)]
struct ColliderSettings {
    show_debugs: bool,
}

#[derive(Component)]
pub struct RectCollider {
    pub owner: Option<Entity>,
    enabled: bool,
    center: Vec2,
    offset: Vec2,
    half_extends: Vec2,
}

#[derive(Component)]
struct DebugColliderView {
    collider: Entity,
}

impl RectCollider {
    pub fn new(owner: Option<Entity>, offset: Vec2, width: f32, height: f32) -> Self {
        Self {
            owner,
            enabled: true,
            center: Vec2::ZERO,
            offset,
            half_extends: Vec2::new(f32::abs(width) * 0.5, f32::abs(height) * 0.5),
        }
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn size(&self) -> Vec2 {
        Vec2::new(self.half_extends.x * 2.0, self.half_extends.y * 2.0)
    }

    pub fn set_center(&mut self, value: Vec2) {
        self.center = value;
    }

    pub fn aabb_collides_with(&self, other: &RectCollider) -> bool {
        if !self.enabled || !other.enabled {
            return false;
        }

        let self_pos = self.center + self.offset;
        let self_xs = self_pos.x - self.half_extends.x;
        let self_xe = self_pos.x + self.half_extends.x;
        let self_ys = self_pos.y - self.half_extends.y;
        let self_ye = self_pos.y + self.half_extends.y;

        let other_pos = other.center + other.offset;
        let other_xs = other_pos.x - other.half_extends.x;
        let other_xe = other_pos.x + other.half_extends.x;
        let other_ys = other_pos.y - other.half_extends.y;
        let other_ye = other_pos.y + other.half_extends.y;

        self_xs < other_xe && self_xe > other_xs && self_ys < other_ye && self_ye > other_ys
    }
}

fn collider_debug_switch_system(
    keyboard: Res<Input<KeyCode>>,
    mut collider_settings: ResMut<ColliderSettings>,
) {
    if keyboard.just_pressed(KeyCode::F6) {
        collider_settings.show_debugs = !collider_settings.show_debugs;
    }
}

fn collider_added_debug_system(
    mut commands: Commands,
    colliders_added: Query<(Entity, &RectCollider), Added<RectCollider>>,
) {
    for (entity, collidder) in colliders_added.iter() {
        commands
            .spawn(SpriteBundle {
                visibility: Visibility::INVISIBLE,
                sprite: Sprite {
                    custom_size: collidder.size().into(),
                    color: Color::Rgba {
                        red: 0.0,
                        green: 0.0,
                        blue: 1.0,
                        alpha: 0.5,
                    },
                    ..default()
                },
                ..default()
            })
            .insert(DebugColliderView { collider: entity });
    }
}

fn collider_debug_update_system(
    mut commands: Commands,
    collider_settings: Res<ColliderSettings>,
    colliders: Query<(Entity, &RectCollider)>,
    mut colliders_debug: Query<(
        Entity,
        &mut Sprite,
        &mut Transform,
        &mut Visibility,
        &DebugColliderView,
    )>,
) {
    for (entity, mut sprite, mut transform, mut visibility, collider_debug) in
        colliders_debug.iter_mut()
    {
        if let Ok((collider_entity, collider)) = colliders.get(collider_debug.collider) {
            visibility.is_visible = collider_settings.show_debugs;
            sprite.custom_size = collider.size().into();
            let collider_pos = collider.center + collider.offset;
            transform.translation = Vec3::new(collider_pos.x, collider_pos.y, 10.0);

            let mut color = Color::Rgba {
                red: 0.0,
                green: 0.0,
                blue: 1.0,
                alpha: 0.5,
            };
            for (other_entity, other) in colliders.iter() {
                if collider_entity == other_entity {
                    continue;
                }

                if collider.aabb_collides_with(other) {
                    color.set_r(1.0);
                    break;
                }
            }
            sprite.color = color;
        } else {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn colliders_position_update_system(mut colliders: Query<(&mut RectCollider, &GlobalTransform)>) {
    for (mut collider, transform) in colliders.iter_mut() {
        let new_pos = transform.translation().truncate();
        collider.center = new_pos;
    }
}
