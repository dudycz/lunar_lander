//! Shows how to render simple primitive shapes with a single color.
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
// Debug egui plugin
// use bevy_inspector_egui::WorldInspectorPlugin;
use std::f32::consts::FRAC_PI_2;
use std::f32::consts::PI;

use bevy_rapier2d::prelude::*;

// Asset constants
const LANDER_SPRITE_SHEET: &str = "statek_sheet.png";
const LANDER_SIZE: (f32, f32) = (162., 162.);
const SPRITE_SIZE: (f32, f32) = (162., 260.);
const LANDER_ROTATION_SPEED: f32 = 2.0 * PI / 180.0;
const LANDER_ORIENTATION: f32 = 90.0 * PI / 180.0;
const FONT: &str = "GemunuLibre-ExtraLight.ttf";
const GRAVITY: f32 = 1.0;
const BOOST_FORCE: f32 = 20.0;
const FUEL_AT_START: u32 = 300;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "Lunar lander".to_string(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        // Debug plugins
        //.add_plugin(RapierDebugRenderPlugin::default())
        //.add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup)
        .add_system(text_update_system)
        .add_system(update_lander_rotation_transform)
        .add_system(keyboard_events)
        .run();
}

#[derive(Component)]
struct LanderAngle(f32);

impl LanderAngle {
    fn direction(&self) -> Vec2 {
        let (y, x) = (self.0 + FRAC_PI_2).sin_cos();
        Vec2::new(x, y)
    }
}

#[derive(Component)]
struct Fuel(u32);

#[derive(Component)]
struct VelocityText;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(Camera2dBundle::default());

    // Load sprite sheet
    let texture_handle = asset_server.load(LANDER_SPRITE_SHEET);
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::new(SPRITE_SIZE.0, SPRITE_SIZE.1),
        2,
        1,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    // Spawn lander
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::default()
                .with_scale(Vec3::splat(0.2))
                .with_translation(Vec3::new(-500.0, 200.0, 1.0)),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .with_children(|children| {
            children
                .spawn()
                .insert(Collider::cuboid(LANDER_SIZE.0 / 2.0, LANDER_SIZE.1 / 2.0))
                .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 10.0, 0.0)));
        })
        .insert(LanderAngle(LANDER_ORIENTATION))
        .insert(Fuel(FUEL_AT_START))
        .insert(Velocity::linear(Vec2::new(200.0, 0.0)))
        .insert(GravityScale(GRAVITY))
        .insert(ExternalForce::default())
        .insert(LockedAxes::ROTATION_LOCKED);

    // Prepare font
    let font = asset_server.load(FONT);
    let text_style = TextStyle {
        font,
        font_size: 25.0,
        color: Color::WHITE,
    };

    // Spawn flight metrics
    commands
        .spawn_bundle(
            TextBundle::from_sections([
                TextSection::from_style(text_style.clone()),
                TextSection::from_style(text_style.clone()),
                TextSection::from_style(text_style.clone()),
            ])
            .with_style(Style {
                align_self: AlignSelf::FlexEnd,
                ..default()
            }),
        )
        .insert(VelocityText);

    // Spawn ground
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Box::new(2.0, 0.1, 1.0)))
                .into(),
            transform: Transform::default()
                .with_scale(Vec3::splat(50.))
                .with_translation(Vec3::new(0.0, -250.0, 1.0)),
            material: materials.add(ColorMaterial::from(Color::GRAY)),
            ..default()
        })
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(1.0, 0.1));
}

fn text_update_system(
    mut query: Query<&mut Text, With<VelocityText>>,
    mut lander: Query<(&mut Fuel, &mut Velocity)>,
) {
    // position transform.translation.y
    let mut x = 0.;
    let mut y = 0.;
    let mut f = 0;
    for (fuel, velocity) in &mut lander {
        x = velocity.linvel.x;
        y = velocity.linvel.y;
        f = fuel.0;
    }

    for mut text in &mut query {
        text.sections[0].value = format!("VERTICAL VELOCITY: {:.0}", y.abs());
        text.sections[1].value = format!("\nHORIZONTAL VELOCITY: {:.0}", x.abs());
        text.sections[2].value = format!("\nFUEL: {:.0}", f);
    }
}

fn update_lander_rotation_transform(mut query: Query<(&LanderAngle, &mut Transform)>) {
    for (rotation_angle, mut transform) in &mut query {
        transform.rotation = Quat::from_rotation_z(rotation_angle.0);
    }
}

fn keyboard_events(
    keys: Res<Input<KeyCode>>,
    mut query: Query<(
        &mut LanderAngle,
        &mut Fuel,
        &mut ExternalForce,
        &mut TextureAtlasSprite,
    )>,
) {
    for (mut rotation_angle, mut fuel, mut thrust, mut sprite) in &mut query {
        if keys.pressed(KeyCode::Left) {
            rotation_angle.0 += LANDER_ROTATION_SPEED;
        } else if keys.pressed(KeyCode::Right) {
            rotation_angle.0 -= LANDER_ROTATION_SPEED;
        }

        if keys.pressed(KeyCode::Up) {
            if fuel.0 > 0 {
                fuel.0 -= 1;
                sprite.index = 1;
                thrust.force = rotation_angle.direction() * BOOST_FORCE;
            } else {
                sprite.index = 0;
                thrust.force = Vec2::new(0., 0.);
            }
        }
        if keys.just_released(KeyCode::Up) {
            sprite.index = 0;
            thrust.force = Vec2::new(0., 0.);
        }
    }
}
