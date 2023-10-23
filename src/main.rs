use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::window::Window;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::InspectorOptions;
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
use bevy_inspector_egui::quick::{ResourceInspectorPlugin, WorldInspectorPlugin};
use bevy_window_title_diagnostics::WindowTitleLoggerDiagnosticsPlugin;
use rand::prelude::*;

const PARTICLE_SIZE: f32 = 20.;
const WINDOW_WIDTH: f32 = 1920.;
const WINDOW_HEIGHT: f32 = 1080.;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .insert_resource(SimConfig::default())
        .register_type::<SimConfig>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(ResourceInspectorPlugin::<SimConfig>::default())
        .add_plugins(WindowTitleLoggerDiagnosticsPlugin { ..Default::default() })
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Startup, (spawn_camera, spawn_random_scene))
        .add_systems(Update, (apply_gravity, update_position, resolve_collision, draw_gizmos))
        .run();
}

#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct SimConfig {
    #[inspector(min = 0.0, max = 1., speed = 0.05, display = NumberDisplay::Slider)]
    collision_damping: f32,
    bounds_size: Vec2,
    gravity: f32,
    particles_num: u32,
    particles_spacing: f32,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            collision_damping: 1.0,
            bounds_size: Vec2::new(WINDOW_WIDTH * 0.8, WINDOW_HEIGHT * 0.8),
            gravity: 500.,
            particles_num: 100,
            particles_spacing: 2. * PARTICLE_SIZE + 10.,
        }
    }
}

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct WaterAtom;

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::new_with_far(8000.));
}

fn spawn_ordered_scene(
    mut commands: Commands,
    sim_config: Res<SimConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let particles_per_row = f32::sqrt(sim_config.particles_num as f32);
    let particles_per_col = ((sim_config.particles_num as f32 - 1.) / particles_per_row + 1.);

    for i in 0..sim_config.particles_num {
        let x = (i as f32 % particles_per_row - particles_per_row / 2. + 0.5) * sim_config.particles_spacing;
        let y = (i as f32 / particles_per_row - particles_per_col / 2. + 0.5) * sim_config.particles_spacing;

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(PARTICLE_SIZE).into()).into(),
                material: materials.add(ColorMaterial::from(Color::BLUE)),
                transform: Transform::from_translation(Vec3::new(y, x, 0.)),
                ..default()
            },
            Velocity(Vec2::ZERO),
            WaterAtom,
        ));
    }
}

fn spawn_random_scene(
    mut commands: Commands,
    sim_config: Res<SimConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = thread_rng();
    let half_width = (sim_config.bounds_size.x - PARTICLE_SIZE) / 2.;
    let half_height = (sim_config.bounds_size.y - PARTICLE_SIZE) / 2.;

    let x_values = (0..sim_config.particles_num).map(|_| rng.gen_range(-half_width..half_width)).collect::<Vec<f32>>();
    let y_values = (0..sim_config.particles_num).map(|_| rng.gen_range(-half_height..half_height)).collect::<Vec<f32>>();

    for i in 0..(sim_config.particles_num as usize) {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(PARTICLE_SIZE).into()).into(),
                material: materials.add(ColorMaterial::from(Color::BLUE)),
                transform: Transform::from_translation(Vec3::new(x_values[i], y_values[i], 0.)),
                ..default()
            },
            Velocity(Vec2::ZERO),
            WaterAtom,
        ));
    }
}

fn smoothing_kernel(radius: f32, dst: f32) -> f32 {
    let volume = std::f32::consts::PI * radius.powf(8.) / 4.;
    let v = (radius - dst).max(0.);

    v * v * v / volume
}

fn draw_gizmos(
    mut gizmos: Gizmos,
    sim_config: Res<SimConfig>,
) {
    gizmos.rect_2d(
        Vec2::ZERO,
        0.,
        sim_config.bounds_size,
        Color::BLACK,
    );
}

fn apply_gravity(
    time: Res<Time>,
    sim_config: Res<SimConfig>,
    mut query: Query<&mut Velocity>,
) {
    for mut velocity in query.iter_mut() {
        velocity.0 += -Vec2::Y * sim_config.gravity * time.delta_seconds();
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
    sim_config: Res<SimConfig>,
    mut query: Query<(&mut Transform, &mut Velocity, With<WaterAtom>)>,
) {
    let half_bounds_size = sim_config.bounds_size * 0.5 - Vec2::ONE * PARTICLE_SIZE;

    for (mut transform, mut velocity, _) in query.iter_mut() {
        if transform.translation.x.abs() > half_bounds_size.x {
            transform.translation.x = half_bounds_size.x * transform.translation.x.signum();
            velocity.x *= -1. * sim_config.collision_damping;
        }
        if transform.translation.y.abs() > half_bounds_size.y {
            transform.translation.y = half_bounds_size.y * transform.translation.y.signum();
            velocity.y *= -1. * sim_config.collision_damping;
        }
    }
}