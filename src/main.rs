use std::io::Read;

use proc_gen2::generation::{region::Room, Bounds};
use proc_gen2::map::{Map, Vector3};
use rhai::{Engine, Dynamic};

fn main() {
    let _map = Map::default();
    // let bounds = Bounds::new(Vector3::new(-128.0, -192.0, -128.0), Vector3::new(384f32, 320.0, -64.0));
    // let cube = Map::cube_dev1(bounds);
    // map.add_solid(cube);

    let mut map = Map::default();
    let room = Room::new(Bounds {
        min: Vector3::new(-512.0, -512.0, -512.0),
        max: Vector3::new(512.0, 512.0, 512.0),
    });
    map.options.cordon = Some(Bounds {
        min: Vector3::new(-5120.0, -5120.0, -5120.0),
        max: Vector3::new(5120.0, 5120.0, 5120.0),
    });
    room.construct(&mut map);

    // let mut state = IdInfo::default();
    // let vmf = map.to_vmf(&mut state);
    // println!("{vmf}");

    // let vmf = vmf_parser_nom::parse::<&str, ()>("abc123").unwrap();

    let engine = Engine::new();
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let output = engine.eval::<Dynamic>(&input).unwrap();
    eprintln!("{output}");
}

// // extern crate kiss3d;

// use std::thread::sleep;
// use std::time::{Duration, Instant};

// use kiss3d::camera::{self, Camera, FirstPerson};
// use kiss3d::event::Key;
// use kiss3d::light::Light;
// use kiss3d::nalgebra::{Point3, Translation3, UnitQuaternion, Vector3};
// use kiss3d::scene::SceneNode;
// use kiss3d::window::{Window, State};

// //<https://docs.speedtree.com/doku.php?id=coordinate_systems>

// // kiss = right handed y up
// // source = right handed z up
// fn hammer_to_kiss(v: Point3<f32>) -> Point3<f32> {
//     Point3::new(v.y, v.z, v.x)
// }

// /// hammer to kiss coords
// fn zright_to_yright(v: Point3<f32>) -> Point3<f32> {
//     Point3::new(v.y, v.z, v.x)
// }

// /// kiss to hammer coords
// fn yright_to_zright(v: Point3<f32>) -> Point3<f32> {
//     Point3::new(v.z, v.x, v.y)
// }

// /// hammer to kiss coords
// fn zright_to_yright_t(v: Translation3<f32>) -> Translation3<f32> {
//     Translation3::new(v.y, v.z, v.x)
// }

// // TODO: not horrible
// // TODO: acutal proc gen
// // TODO:     themeing from files
// // TODO:     scale, rotate, cylinder
// // TODO: disp in kiss
// // TODO: vmf tools: add cylinder with many sides
// // TODO: input graph/blocks
// // TODO: topdown/front/side tui

// struct AppState {
//     scene: SceneNode,
//     rot: UnitQuaternion<f32>,
//     camera: FirstPerson
// }

// impl State for AppState {
//     fn step(&mut self, _: &mut Window) {
//         self.scene.prepend_to_local_rotation(&self.rot)
//     }

//     fn cameras_and_effect(
//         &mut self,
//     ) -> (
//         Option<&mut dyn Camera>,
//         Option<&mut dyn kiss3d::planar_camera::PlanarCamera>,
//         Option<&mut dyn kiss3d::post_processing::PostProcessingEffect>,
//     ) {
//         (Some(&mut self.camera), None, None)
//     }
// }

// fn main() {
//     // let camera_pos = Point3::new(5.0, 2.0, 2.0);
//     // -50, 250, 20
//     // 2, 3, 1
//     let camera_pos = Point3::new(2.0, -5.0, 3.0);
//     // let camera_pos = hammer_to_kiss(Point3::new(2.0, -5.0, 2.0));
//     let origin = Point3::new(0.0, 0.0, 0.0);
//     let mut camera = camera::FirstPerson::new(camera_pos, origin);
//     camera.rebind_up_key(Some(Key::W));
//     camera.rebind_left_key(Some(Key::A));
//     camera.rebind_down_key(Some(Key::S));
//     camera.rebind_right_key(Some(Key::D));
//     camera.set_pitch_step(0.005/2.0);
//     camera.set_yaw_step(0.005/2.0);
//     // set coord system to right handed z up, from right handed y up.
//     // I think kiss is always right handed
//     camera.set_up_axis(Vector3::new(0.0, 0.0, 1.0));

//     // TODO: set_up_axis_dir

//     let mut window = Window::new("Kiss3d: cube");

//     let mut c_white = window.add_cube(0.75, 0.75, 0.75);
//     c_white.set_color(1.0, 1.0, 1.0);

//     let mut scene = c_white;

//     let mut c_red = scene.add_cube(0.5, 0.5, 0.5);
//     c_red.set_color(1.0, 0.0, 0.0);
//     // c_red.set_local_translation(zright_to_yright_t(Translation3::new(1.0, 0.0, 0.0)));
//     c_red.set_local_translation(Translation3::new(1.0, 0.0, 0.0));

//     let mut c_green = scene.add_cube(0.5, 0.5, 0.5);
//     c_green.set_color(0.0, 1.0, 0.0);
//     // c_green.set_local_translation(zright_to_yright_t(Translation3::new(0.0, 1.0, 0.0)));
//     c_green.set_local_translation(Translation3::new(0.0, 1.0, 0.0));

//     let mut c_blue = scene.add_cube(0.5, 0.5, 0.5);
//     c_blue.set_color(0.0, 0.0, 1.0);
//     // c_blue.set_local_translation(zright_to_yright_t(Translation3::new(0.0, 0.0, 1.0)));
//     c_blue.set_local_translation(Translation3::new(0.0, 0.0, 1.0));

//     window.set_light(Light::StickToCamera);

//     // let mut scene: SceneNode = c_white;
//     // scene.add_child(c_red);
//     // scene.add_child(c_green);
//     // scene.add_child(c_blue);

//     let rot = UnitQuaternion::from_axis_angle(&Vector3::z_axis(), 0.014);
//     let state = AppState { scene, rot, camera };

//     window.render_loop(state);

//     // window.render_loop_with_camera(state)

//     // while window.render_with_camera(&mut camera) {
//     //     let now = Instant::now();
//     //     c_white.prepend_to_local_rotation(&rot);
//     //     c_red.prepend_to_local_rotation(&rot);
//     //     c_green.prepend_to_local_rotation(&rot);
//     //     c_blue.prepend_to_local_rotation(&rot);

//     //     let frametime = now.elapsed();
//     //     let location = camera.eye();
//     //     eprint!(
//     //         "\rframetime:{frametime: <10?} fps: {:<10}, at {location}",
//     //         1.0 / frametime.as_secs_f32()
//     //     );
//     //     const SIXTIETH: Duration = Duration::from_nanos(16666666);
//     //     let sleep_time = SIXTIETH - frametime;
//     //     sleep(sleep_time)
//     // }
// }
