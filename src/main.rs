// #[warn(unused_imports)]
use std::f32::consts::PI;

use bevy::color::palettes::css::WHITE;
// use bevy::core_pipeline::oit::OrderIndependentTransparencySettings;
use bevy::diagnostic::EntityCountDiagnosticsPlugin;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::diagnostic::SystemInformationDiagnosticsPlugin;
use bevy::pbr::CascadeShadowConfig;
// use bevy::render::renderer::{RenderDevice, RenderQueue};

// use bevy::ecs::schedule::ExecutorKind;
// use bevy::window::WindowResolution;

// use bevy::window::WindowMode;
use bevy::{
    // text::FontSmoothing,
    prelude::*,
};

use bevy::{
    color::palettes::tailwind::*,
    // core_pipeline::{
    //     bloom::BloomSettings,
    //     dof::{DepthOfFieldMode, DepthOfFieldSettings},
    //     prepass::{DepthPrepass, NormalPrepass},
    //     tonemapping::Tonemapping,
    //     Skybox,
    // },
    pbr::{
        wireframe::{Wireframe, WireframePlugin},
        // CascadeShadowConfigBuilder, ExtendedMaterial, OpaqueRendererMethod,
    },
    render::{
        mesh::VertexAttributeValues,
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
};
// use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
// use noise::{BasicMulti, NoiseFn, Perlin, Seedable};
// use bevy::prelude::*;
use noise::{BasicMulti, NoiseFn, Perlin};

// https://bevy-cheatbook.github.io/setup/perf.html
use bevy::core::TaskPoolThreadAssignmentPolicy;
use bevy::tasks::available_parallelism;

// https://github.com/bevyengine/bevy/blob/main/docs/profiling.md#gpu-runtime
// https://github.com/bevyengine/bevy/blob/latest/docs/profiling.md
// https://ui.perfetto.dev/
// https://bevy-cheatbook.github.io/pitfalls/performance.html
// https://github.com/PhaestusFox/BevyBasics
// https://github.com/Adamekka/bevy-fps-counter/blob/f2c5bef25b3148c087e058c2ab39df00c11b0f6b/examples/basic.rs
// use bevy_fps_counter::{FpsCounter, FpsCounterPlugin};
// use bevy_fps_counter::FpsCounterPlugin;
// mod water_material;
// use water_material::*;

// text::FontSmoothing

// struct OverlayColor;

// impl OverlayColor {
//     const RED: Color = Color::srgb(1.0, 0.0, 0.0);
//     const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
// }

// #[repr(C)]
// pub enum EPresentMode {
//     AutoVsync = 0,
//     AutoNoVsync = 1,
//     Fifo = 2,
//     FifoRelaxed = 3,
//     Immediate = 4,
//     Mailbox = 5,
// }

fn main() {
    App::new()
        .add_plugins((DefaultPlugins
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                }),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Bevy Game".to_string(),
                    // resolution: WindowResolution::new(800., 600.),
                    // with_scale_factor_override(1.),
                    // resizable: false,
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    // mode: WindowMode::BorderlessFullscreen,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .set(TaskPoolPlugin {
                task_pool_options: TaskPoolOptions {
                    compute: TaskPoolThreadAssignmentPolicy {
                        // set the minimum # of compute threads
                        // to the total number of available threads
                        min_threads: available_parallelism(),
                        max_threads: std::usize::MAX, // unlimited max threads
                        percent: 50.0,                // this value is irrelevant in this case
                    },
                    // keep the defaults for everything else
                    ..default()
                },
            }),))
        // .edit_schedule(Update, |schedule| {
        //     schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        // })
        .add_plugins((WireframePlugin,))
        // .add_plugins((PanOrbitCameraPlugin,))
        // .add_plugins((MaterialPlugin::<
        //     ExtendedMaterial<StandardMaterial, WaterExtension>,
        // >::default(),))
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        // we want Bevy to measure these values for us:
        .add_plugins(EntityCountDiagnosticsPlugin::default())
        .add_plugins(SystemInformationDiagnosticsPlugin::default())
        .add_systems(Startup, startup)
        .add_systems(Update, toggle_wireframe)
        // .add_systems(Update, debug_transform)
        // .add_plugins(FpsCounterPlugin)
        // .add_systems(Startup, debug_renderes)
        // If a UI camera is already in your game remove the next line
        // .add_systems(Startup, |mut commands: Commands| {
        //     commands.spawn_bundle(Camera2dBundle::default());
        // })
        .run();
}

const TERRAIN_XZ_TO_Y_SCALLER: f32 = 2.0;
const TERRAIN_HEIGHT: f32 = 70.0;
const TERRAIN_CHUNK_W: f32 = 1024.0 / TERRAIN_XZ_TO_Y_SCALLER;
const TERRAIN_CHUNK_H: f32 = 1024.0 / TERRAIN_XZ_TO_Y_SCALLER;
const TERRAIN_CHUNK_SUBDIVISIONS: u32 = 32 / TERRAIN_XZ_TO_Y_SCALLER as u32;
const TERRAIN_CHUNK_SCALLER: f64 = 300.0;

fn generate_chunk(
    // mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
    x: f64,
    z: f64,
) -> Mesh {
    let noise: BasicMulti<Perlin> = BasicMulti::<Perlin>::default();

    let mut terrain = Mesh::from(
        Plane3d::default()
            .mesh()
            // .size(1000.0, 1000.0)
            // .subdivisions(20),
            .size(TERRAIN_CHUNK_W, TERRAIN_CHUNK_H)
            .subdivisions(TERRAIN_CHUNK_SUBDIVISIONS),
    );

    if let Some(VertexAttributeValues::Float32x3(positions)) =
        terrain.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    {
        // main terrain topology
        for pos in positions.iter_mut() {
            let xi: f32 = noise.get([
                (pos[0] as f64 + (TERRAIN_CHUNK_W as f64 * x)) as f64 / TERRAIN_CHUNK_SCALLER,
                (pos[2] as f64 + (TERRAIN_CHUNK_H as f64 * z)) as f64 / TERRAIN_CHUNK_SCALLER,
                0. as f64,
            ]) as f32;
            pos[0] += (TERRAIN_CHUNK_W * x as f32) as f32;
            pos[1] = xi * TERRAIN_HEIGHT * 1.0;
            pos[2] += (TERRAIN_CHUNK_H * z as f32) as f32;
        }

        // seconds pass
        // for pos in positions.iter_mut() {
        //     let xi: f32 = noise.get([
        //         pos[0] as f64 / (TERRAIN_CHUNK_SCALLER * 0.1) + (TERRAIN_CHUNK_SCALLER * x),
        //         pos[2] as f64 / (TERRAIN_CHUNK_SCALLER * 0.1) + (TERRAIN_CHUNK_SCALLER * z),
        //         0. as f64,
        //     ]) as f32;
        //     pos[1] += xi * TERRAIN_HEIGHT * 0.1 / TERRAIN_XZ_TO_Y_SCALLER;
        // }

        // third pass
        // for pos in positions.iter_mut() {
        //     let xi: f32 = noise.get([
        //         pos[0] as f64 / (TERRAIN_CHUNK_SCALLER * 0.01) + (TERRAIN_CHUNK_SCALLER * x),
        //         pos[2] as f64 / (TERRAIN_CHUNK_SCALLER * 0.01) + (TERRAIN_CHUNK_SCALLER * z),
        //         0. as f64,
        //     ]) as f32;
        //     pos[1] += xi * TERRAIN_HEIGHT * 0.1 / TERRAIN_XZ_TO_Y_SCALLER * 0.2;
        // }

        for pos in positions.iter_mut() {
            pos[1] *= 2.0;
        }

        let colors: Vec<[f32; 4]> = positions
            .iter()
            .map(|[_, g, _]| {
                // return Color::from(GREEN_400).to_linear().to_f32_array();
                // return Color::BLACK.to_linear().to_f32_array();

                let g = *g / TERRAIN_HEIGHT * 2.;
                // if g > 0.8 {
                if g > 1.1 {
                    Color::from(GRAY_100).to_linear().to_f32_array()
                } else if g > 1.0 {
                    Color::from(GRAY_300).to_linear().to_f32_array()
                } else if g > 0.9 {
                    Color::from(AMBER_800).to_linear().to_f32_array()
                } else if g > 0.8 {
                    Color::from(YELLOW_400).to_linear().to_f32_array()
                } else if g > 0.7 {
                    Color::from(YELLOW_500).to_linear().to_f32_array()
                } else if g > 0.6 {
                    Color::from(AMBER_400).to_linear().to_f32_array()
                } else if g > 0.5 {
                    Color::from(AMBER_500).to_linear().to_f32_array()
                } else if g > 0.4 {
                    Color::from(AMBER_600).to_linear().to_f32_array()
                } else if g > 0.3 {
                    Color::from(AMBER_700).to_linear().to_f32_array()
                } else if g > 0.2 {
                    Color::from(AMBER_800).to_linear().to_f32_array()
                } else if g < -0.2 {
                    Color::from(GREEN_800).to_linear().to_f32_array()
                } else if g < -0.5 {
                    Color::from(ORANGE_400).to_linear().to_f32_array()
                } else if g < -0.6 {
                    Color::from(BLUE_400).to_linear().to_f32_array()
                } else if g < -0.7 {
                    Color::from(WHITE).to_linear().to_f32_array()
                } else if g < -0.8 {
                    Color::from(PURPLE_400).to_linear().to_f32_array()
                } else {
                    Color::from(GREEN_600).to_linear().to_f32_array()
                    // Color::from(RED_600).to_linear().to_f32_array()
                }
            })
            .collect();
        terrain.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        terrain.compute_normals();
        // terrain.translate_by();
    }

    return terrain;
}

// #[derive(Component)]
// struct CameraMarker;

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // mut water_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, WaterExtension>>>,
    // asset_server: ResMut<AssetServer>,
) {
    commands.spawn((
        Camera3d::default(),
        // OrderIndependentTransparencySettings {
        //     layer_count: 16,
        //     ..default()
        // },
        Transform::from_xyz(-94.60196, 162.97789, 306.44165),
    ));

    // commands.spawn((
    //     (
    //         Camera3dBundle {
    //             transform: Transform::from_xyz(-94.60196, 162.97789, 306.44165)
    //                 .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
    //             camera: Camera {
    //                 hdr: true,
    //                 ..default()
    //             },
    //             tonemapping: Tonemapping::TonyMcMapface,
    //             ..default()
    //         },
    //         CameraMarker,
    //     ),
    //     // Skybox {
    //     //     brightness: 5000.0,
    //     //     image: asset_server.load(
    //     //         "skybox_cubemap/skybox_specular.ktx2",
    //     //     ),
    //     // },
    //     // EnvironmentMapLight {
    //     //     diffuse_map: asset_server
    //     //         .load("skybox_cubemap/skybox_diffuse.ktx2"),
    //     //     specular_map: asset_server.load(
    //     //         "skybox_cubemap/skybox_specular.ktx2",
    //     //     ),
    //     //     intensity: 2000.0,
    //     // },
    //     // Skybox {
    //     //     brightness: 1000.0,
    //     //     image: asset_server
    //     //         .load("kloppenheim_06_puresky_4k_diffuse/kloppenheim_06_puresky_4k_specular.ktx2"),
    //     // },
    //     // EnvironmentMapLight {
    //     //     diffuse_map: asset_server
    //     //         .load("kloppenheim_06_puresky_4k_diffuse/kloppenheim_06_puresky_4k_diffuse.ktx2"),
    //     //     specular_map: asset_server
    //     //         .load("kloppenheim_06_puresky_4k_diffuse/kloppenheim_06_puresky_4k_specular.ktx2"),
    //     //     intensity: 1000.0,
    //     // },
    //     // BloomSettings::NATURAL,
    //     PanOrbitCamera::default(),
    //     DepthOfFieldSettings {
    //         mode: DepthOfFieldMode::Gaussian,
    //         focal_distance: 40.,
    //         aperture_f_stops: 1.0 / 8.0,
    //         ..default()
    //     },
    //     DepthPrepass,
    //     NormalPrepass,
    // ));

    // commands.spawn(PerfUiAllEntries::default());
    // commands.spawn(PerfUiPlugin::default());

    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        CascadeShadowConfig {
            // first_cascade_far_bound: 4.0,
            // maximum_distance: 10.0,
            // bounds: Vec<f32>,
            overlap_proportion: 10.0,
            minimum_distance: 2.0,
            ..default()
        },
        // ..default()
    ));

    // let mut terrains: Vec<Mesh> = vec![];

    for x in -2..=2 {
        for z in -2..=2 {
            let terrain: Mesh = generate_chunk(x as f64, z as f64);

            commands.spawn((
                Mesh3d(meshes.add(terrain)),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    perceptual_roughness: 0.9,
                    ..default()
                })),
                // Transform::from_translation(Vec3::new(-200., 0., 0.)),
                Terrain,
            ));

            // commands.spawn((
            //     PbrBundle {
            //         mesh: meshes.add(terrain),
            //         material: materials.add(StandardMaterial {
            //             base_color: Color::WHITE,
            //             perceptual_roughness: 0.9,
            //             ..default()
            //         }),
            //         ..default()
            //     },
            //     Terrain,
            // ));

            // water
            let water = Mesh::from(
                Plane3d::default()
                    .mesh()
                    .size(TERRAIN_CHUNK_W, TERRAIN_CHUNK_H)
                    .subdivisions(TERRAIN_CHUNK_SUBDIVISIONS),
            );

            commands.spawn((
                Mesh3d(meshes.add(water)),
                MeshMaterial3d(materials.add(Color::srgb(255.0, 0.0, 0.0))),
                Transform::from_xyz(
                    (TERRAIN_CHUNK_W * x as f32) as f32,
                    -(TERRAIN_HEIGHT / 2.) + TERRAIN_HEIGHT * 6. / 16.,
                    (TERRAIN_CHUNK_W * z as f32) as f32,
                ),
            ));

            // commands.spawn((
            //     // Mesh3d(meshes.add(Circle::new(100.0))),
            //     // MeshMaterial3d(materials.add(Color::srgb(7.5, 0.0, 7.5))),
            //     // Transform::from_translation(Vec3::new(-200., 0., 0.)),
            //     Mesh3d(meshes.add(water)),
            //     MeshMaterial3d(water_materials.add(ExtendedMaterial {
            //         base: StandardMaterial {
            //             clearcoat: 0.5,
            //             clearcoat_perceptual_roughness: 0.3,
            //             // clearcoat_normal_texture: Some(asset_server.load_with_settings(
            //             //     "textures/ScratchedGold-Normal.png",
            //             //     |settings: &mut ImageLoaderSettings| settings.is_srgb = false,
            //             // )),
            //             metallic: 0.4,
            //             base_color: BLUE_400.into(),
            //             perceptual_roughness: 0.8,
            //             // ** clearcoat: 1.0,
            //             // ** clearcoat_perceptual_roughness: 0.3,
            //             // ** // clearcoat_normal_texture: Some(asset_server.load_with_settings(
            //             // ** //     "textures/ScratchedGold-Normal.png",
            //             // ** //     |settings: &mut ImageLoaderSettings| settings.is_srgb = false,
            //             // ** // )),
            //             // ** metallic: 0.9,
            //             // ** base_color: BLUE_400.into(),
            //             // ** perceptual_roughness: 0.2,

            //             // can be used in forward or deferred mode.
            //             opaque_render_method: OpaqueRendererMethod::Auto,
            //             // in deferred mode, only the PbrInput can be modified (uvs, color and other material properties),
            //             // in forward mode, the output can also be modified after lighting is applied.
            //             // see the fragment shader `extended_material.wgsl` for more info.
            //             // Note: to run in deferred mode, you must also add a `DeferredPrepass` component to the camera and either
            //             // change the above to `OpaqueRendererMethod::Deferred` or add the `DefaultOpaqueRendererMethod` resource.
            //             alpha_mode: AlphaMode::Blend,
            //             ..default()
            //         },
            //         extension: WaterExtension { quantize_steps: 30 },
            //     })),
            //     Transform::from_xyz(
            //         (TERRAIN_CHUNK_W * x as f32) as f32,
            //         -(TERRAIN_HEIGHT / 2.) + TERRAIN_HEIGHT * 6. / 16.,
            //         (TERRAIN_CHUNK_W * z as f32) as f32,
            //     ),
            // ));

            // commands.spawn(MaterialMeshBundle {
            //     mesh: meshes.add(water),
            //     transform: Transform::from_xyz(
            //         (TERRAIN_CHUNK_W * x as f32) as f32,
            //         -(TERRAIN_HEIGHT / 2.) + TERRAIN_HEIGHT * 6. / 16.,
            //         (TERRAIN_CHUNK_W * z as f32) as f32,
            //     ),
            //     material: water_materials.add(ExtendedMaterial {
            //         base: StandardMaterial {
            //             clearcoat: 0.5,
            //             clearcoat_perceptual_roughness: 0.3,
            //             // clearcoat_normal_texture: Some(asset_server.load_with_settings(
            //             //     "textures/ScratchedGold-Normal.png",
            //             //     |settings: &mut ImageLoaderSettings| settings.is_srgb = false,
            //             // )),
            //             metallic: 0.4,
            //             base_color: BLUE_400.into(),
            //             perceptual_roughness: 0.8,
            //             // ** clearcoat: 1.0,
            //             // ** clearcoat_perceptual_roughness: 0.3,
            //             // ** // clearcoat_normal_texture: Some(asset_server.load_with_settings(
            //             // ** //     "textures/ScratchedGold-Normal.png",
            //             // ** //     |settings: &mut ImageLoaderSettings| settings.is_srgb = false,
            //             // ** // )),
            //             // ** metallic: 0.9,
            //             // ** base_color: BLUE_400.into(),
            //             // ** perceptual_roughness: 0.2,

            //             // can be used in forward or deferred mode.
            //             opaque_render_method: OpaqueRendererMethod::Auto,
            //             // in deferred mode, only the PbrInput can be modified (uvs, color and other material properties),
            //             // in forward mode, the output can also be modified after lighting is applied.
            //             // see the fragment shader `extended_material.wgsl` for more info.
            //             // Note: to run in deferred mode, you must also add a `DeferredPrepass` component to the camera and either
            //             // change the above to `OpaqueRendererMethod::Deferred` or add the `DefaultOpaqueRendererMethod` resource.
            //             alpha_mode: AlphaMode::Blend,
            //             ..default()
            //         },
            //         extension: WaterExtension { quantize_steps: 30 },
            //     }),
            //     ..default()
            // });
        }
    }
}

// fn debug_projection(query_camera: Query<&Projection, With<MyCameraMarker>>) {
//     let projection = query_camera.single();
//     match projection {
//         Projection::Perspective(persp) => {
//             // we have a perspective projection
//         }
//         Projection::Orthographic(ortho) => {
//             // we have an orthographic projection
//         }
//     }
// }

// static mut X: i32 = 0;
// fn debug_transform(query_camera: Query<&Transform, With<CameraMarker>>) {
//     unsafe {
//         X += 1;
//         if X % 100 == 0 {
//             let transform = query_camera.single();
//             println!(
//                 "cam: (:x, :y, :z) = ({}, {}, {})",
//                 transform.translation.x, transform.translation.y, transform.translation.z
//             );
//         }
//     }
// }

// fn debug_renderes(_render_device: Res<RenderDevice>, _render_queue: Res<RenderQueue>) {
//     // Access GPU information via render_device or render_queue
//     // Example: log device limits
//     // let limits = render_device.limits();
//     // info!("Max texture size: {}", limits.max_texture_dimension_2d);
// }

#[derive(Component)]
struct Terrain;

fn toggle_wireframe(
    mut commands: Commands,
    landscapes_wireframes: Query<Entity, (With<Terrain>, With<Wireframe>)>,
    landscapes: Query<Entity, (With<Terrain>, Without<Wireframe>)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    // for terrain in &landscapes {
    //     commands.entity(terrain).insert(Wireframe);
    // }

    if input.just_pressed(KeyCode::Space) {
        for terrain in &landscapes {
            commands.entity(terrain).insert(Wireframe);
        }
        for terrain in &landscapes_wireframes {
            commands.entity(terrain).remove::<Wireframe>();
        }
    }
}
