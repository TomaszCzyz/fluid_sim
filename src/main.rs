use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_inspector_egui::quick::{ResourceInspectorPlugin, WorldInspectorPlugin};
use bevy_window_title_diagnostics::WindowTitleLoggerDiagnosticsPlugin;

const PARTICLE_SIZE: f32 = 20.;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .insert_resource(Gravity { val: 10. })
        .insert_resource(BoundsSize::default())
        .register_type::<Gravity>()
        .register_type::<BoundsSize>()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(ResourceInspectorPlugin::<Gravity>::default())
        .add_plugins(WindowTitleLoggerDiagnosticsPlugin {
            // It is possible to filter Diagnostics same way as default LogDiagnosticsPlugin
            // filter: Some(vec![FrameTimeDiagnosticsPlugin::FPS]),
            ..Default::default()
        })
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Startup, (spawn_camera, spawn_basic_scene))
        .add_systems(Update, (apply_gravity, update_position, resolve_collision, draw_gizmos))
        .run();
}

#[derive(Reflect, Resource, Deref, Default)]
#[reflect(Resource)]
struct Gravity {
    val: f32,
}

#[derive(Reflect, Resource, Deref)]
#[reflect(Resource)]
struct BoundsSize {
    size: Vec2,
}

impl Default for BoundsSize {
    fn default() -> Self {
        Self { size: Vec2::new(200., 300.) }
    }
}

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct WaterAtom;

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::new_with_far(1000.));
}

fn spawn_basic_scene(
    mut commands: Commands,
    mut config: ResMut<GizmoConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    config.line_width = 15.;

    // Circle
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(PARTICLE_SIZE).into()).into(),
            material: materials.add(ColorMaterial::from(Color::BLUE)),
            // transform: Transform::from_translation(Vec3::new(-150., 0., 0.)),
            ..default()
        },
        Velocity(Vec2::ZERO),
        WaterAtom,
    ));
}

fn draw_gizmos(
    mut gizmos: Gizmos,
    bounds: Res<BoundsSize>,
) {
    gizmos.rect_2d(
        Vec2::ZERO,
        0.,
        bounds.size,
        Color::BLACK,
    );
}

fn apply_gravity(
    time: Res<Time>,
    gravity: Res<Gravity>,
    mut query: Query<&mut Velocity>,
) {
    for mut velocity in query.iter_mut() {
        velocity.0 += -Vec2::Y * gravity.val * time.delta_seconds();
    }
}

fn update_position(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity, With<WaterAtom>)>,
) {
    for (mut transform, velocity, _) in &mut query {
        transform.translation += velocity.extend(0.0) * time.delta_seconds();
    }
}

fn resolve_collision(
    bounds_size: Res<BoundsSize>,
    mut query: Query<(&Transform, &mut Velocity, With<WaterAtom>)>,
) {
    let half_bounds_size = bounds_size.size * 0.5  - Vec2::ONE * PARTICLE_SIZE;

    for (transform, mut velocity, _) in query.iter_mut() {
        if transform.translation.x.abs() > half_bounds_size.x {
            // should I update translation?
            // transform.translation.x = half_bounds_size.x * transform.translation.x.signum(); 
            velocity.x *= -1.;
        }
        if transform.translation.y.abs() > half_bounds_size.y {
            velocity.y *= -1.;
        }
    }
}