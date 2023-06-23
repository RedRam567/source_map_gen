use vmf_parser_nom::ast::Vmf;

use crate::map::Map;
use crate::prelude::Vector3;
use crate::vmf::{block_to_solid, ToLower};
use crate::StrType;

use super::*;

fn make_shape<'a>(
    shape: &str, bounds: &Bounds, sides: u32, mats: [&'a Material<'a>; 6],
    options: &'a SolidOptions,
) -> OneOrVec<Solid<'a>> {
    // TODO: horrible
    // let spike_bounds = &Bounds {max: Vector3 {z: bounds.max.z / 2.0, ..bounds.max }, ..bounds.clone()};
    let spike_bounds = &bounds;
    match shape {
        "cube" => OneOrVec::One(cube(bounds, mats[..].try_into().unwrap(), options)),
        "wedge" => OneOrVec::One(wedge(bounds, mats[..].try_into().unwrap(), options)),
        "spike" => spike(spike_bounds, sides, mats[..].try_into().unwrap(), options),
        "cylinder" => cylinder(bounds, sides, mats[..].try_into().unwrap(), options),
        // "frustum" => frustum(bounds, sides, mats[..].try_into().unwrap(), options),
        "sphere" => sphere(bounds, sides, mats[..].try_into().unwrap(), options),
        str => panic!("unkown shape {}", str),
        // _ => OneOrVec::new()
    }
}

#[test]
#[ignore]
fn test_frustum_cone() {
    let dev_person = Material::new("DEV/DEV_MEASUREWALL01C");
    let mats = [&dev_person; 3];
    let options = &SolidOptions::default().allow_frac();
    let mut map = Map::default();

    let top = ellipse_verts(Vector3::new(512.0, 512.0, 512.0), 256.0, 256.0, 32, options);
    let bottom = ellipse_verts(Vector3::new(0.0, 0.0, 0.0), 1024.0, 512.0, 32, options);
    let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
    map.add_solid(solid);

    let top = ellipse_verts(Vector3::new(0.0, 0.0, 0.0), 1024.0, 512.0, 32, options);
    let bottom = ellipse_verts(Vector3::new(512.0, 512.0, -512.0), 256.0, 256.0, 32, options);
    let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
    map.add_solid(solid);

    let top = ellipse_verts(Vector3::new(0.0, 0.0, 512.0), 1024.0, 0.0, 32, options);
    let bottom = ellipse_verts(Vector3::new(0.0, 0.0, 0.0), 256.0, 256.0, 32, options);
    let solid = Solid::new(prism(top, bottom, false, mats, options).skip(1).collect::<Vec<_>>());
    map.add_solid(solid);

    let mut map = Map::default();
    let top = ellipse_verts(Vector3::new(0.0, 0.0, 512.0), 5120.0, 0.0, 32, options);
    let bottom = ellipse_verts(Vector3::new(0.0, 0.0, 0.0), 128.0, 512.0, 32, options);
    let solid = Solid::new(prism(top, bottom, false, mats, options).skip(1).collect::<Vec<_>>());
    map.add_solid(solid);

    write_test_vmf(map.to_lower());
    panic!("worked")
}

// TODO: better, make actual unit test / move to integeration / merge into massive test of every shape
/// doc test code as unit test
#[test]
#[ignore]
fn frustum_cone_doc_test() {
    let dev_person = Material::new("DEV/DEV_MEASUREWALL01C");
    let mats = [&dev_person; 3];
    let options = &SolidOptions::default();
    let mut map = Map::default();
    map.options.cordon = Some(crate::generation::Bounds::new(
        Vector3::new(-5120.0, -5120.0, -5120.0),
        Vector3::new(5120.0, 5120.0, 5120.0),
    ));
    // prevent FindPortalSide errors O_o
    map.defaults_l4d2();
    map.entities[0].props[0].value = "0 0 2048".into();
    dbg!(&map.entities[0].props[0]);

    // a perfect 512x512x512 cylinder
    let top = ellipse_verts(Vector3::new(0.0, 0.0, 256.0), 256.0, 256.0, 32, options);
    let bottom = ellipse_verts(Vector3::new(0.0, 0.0, -256.0), 256.0, 256.0, 32, options);
    let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
    map.add_solid(solid);

    // a 512x512x512 frustum
    let top = ellipse_verts(Vector3::new(0.0, 0.0, 256.0), 128.0, 128.0, 32, options);
    let bottom = ellipse_verts(Vector3::new(0.0, 0.0, -256.0), 256.0, 256.0, 32, options);
    let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
    map.add_solid(solid);

    // a 512x512x512 cone
    let top = std::iter::repeat(Vector3::new(0.0, 0.0, 256.0));
    let bottom = ellipse_verts(Vector3::new(0.0, 0.0, -256.0), 256.0, 256.0, 16, options);
    let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
    map.add_solid(solid);

    // a 512x512x512 upside down cone
    let top = ellipse_verts(Vector3::new(0.0, 0.0, 256.0), 256.0, 256.0, 16, options);
    let bottom = std::iter::repeat(Vector3::new(0.0, 0.0, -256.0));
    let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
    map.add_solid(solid);

    // a weird shape
    // causes vbsp warnings if not enclosed and no enitities (others dont tho)
    let top = ellipse_verts(Vector3::new(512.0, 512.0, 512.0), 256.0, 256.0, 32, options);
    let bottom = ellipse_verts(Vector3::new(0.0, 0.0, 0.0), 1024.0, 512.0, 32, options);
    let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
    map.add_solid(solid);

    // a weird shape with the bottom preferred NO WARNINGS? O_O?
    let top = ellipse_verts(Vector3::new(512.0, 512.0, 512.0), 256.0, 256.0, 32, options);
    let bottom = ellipse_verts(Vector3::new(0.0, 0.0, 0.0), 1024.0, 512.0, 32, options);
    let solid = Solid::new(prism(top, bottom, true, mats, options).collect::<Vec<_>>());
    map.add_solid(solid);

    let vmf = map.to_lower();

    write_test_vmf(vmf);
    panic!("worked")
}

#[test]
#[ignore]
fn shape_test() {
    // shape
    // size
    // size long
    // sides
    // let
    // 16x16 to 512

    let dev_person = Material::new("DEV/DEV_MEASUREWALL01C");
    let mats = [&dev_person; 6];

    const CELL_SIZE: i32 = 512;

    let shapes = ["cube", "wedge", "spike", "cylinder", "sphere"];
    // let shapes = ["cube", "wedge", "spike", "cylinder"];
    // let shapes = ["spike"];
    // let sides = [3, 4, 8, 16, 32, 63];
    // let sides = [3, 4, 8, 16, 32, 63];
    let sides = [3, 4, 8, 16];
    // let sizes = [16, 32, 64, 128, 256, 512];
    let sizes = [512, 256, 128, 64, 32, 16];
    let options = SolidOptions::default();

    let mut x = -1;
    let mut y = -1;
    let mut z = -1;

    // TODO: FIXME: REMEMBER TO ADD BACK SPHERE CAPS
    // TODO: align ignore pos
    let mut map = Map::default();
    for shape in shapes {
        for num_sides in sides {
            z = 0;
            for size in sizes {
                // z += CELL_SIZE;
                z += 0;
                let x = (x + CELL_SIZE * 2 - size * 2) as f32;
                let y = (CELL_SIZE / 2 + y - size / 2) as f32;
                let z = (CELL_SIZE / 2 + z - size / 2) as f32;
                let min = Vector3::new(x, y, z);
                let max = min.clone() + size as f32;
                let bounds = Bounds::new(min, max);
                // map.add_solid(cube(&bounds, mats, &options));
                // map.add_solid(wedge(&bounds, mats[..].try_into().unwrap(), &options));
                map.add_solid2(make_shape(shape, &bounds, num_sides, mats, &options));
            }
            x += CELL_SIZE * 2;
        }
        x = 0;
        y += CELL_SIZE;
    }

    write_test_vmf(map.to_lower());

    panic!("worked")
}

// #[ignore]
// #[test]
// fn cylinder_test() {
//     dbg!();
//     let mut map = Map::default();
//     let options = SolidOptions::default();

//     map.add_solid(cylinder(
//         &Bounds::new(Vector3::new(-16.0, -16.0, 0.0), Vector3::new(16.0, 16.0, 32.0)),
//         4,
//         [&Material::new("DEV/DEV_MEASUREWALL01C"); 3],
//         &options,
//     ));

//     write_test_vmf(map.to_lower());
// }

fn write_test_vmf(vmf: Vmf<StrType<'_>>) {
    const OUTPUT_PATH: &str =
        "/home/redram/.local/share/Steam/steamapps/common/Left 4 Dead 2/custom/maps/output2.vmf";
    _ = std::fs::remove_file(OUTPUT_PATH);
    let mut output =
        std::fs::OpenOptions::new().write(true).create(true).open(OUTPUT_PATH).unwrap();

    use std::io::Write;
    //TODO:FIXME: BORKLEN fancy disp :# breaks dispinfo FOR SOME REASON???
    writeln!(output, "{}", vmf).unwrap();
}

#[ignore]
#[test]
fn sphere_test() {
    dbg!();
    let mut map = Map::default();
    let options = SolidOptions { world_align: false, ..SolidOptions::default() };

    let mats = [
        &Material::new("tools/toolsnodraw"),
        &Material::new("tools/toolsnodraw"),
        &Material::new("DEV/DEV_MEASUREWALL01C"),
    ];
    for solid in sphere(
        &Bounds::new(Vector3::new(-256.0, -256.0, 0.0), Vector3::new(256.0, 256.0, 512.0)),
        // &Bounds::new(Vector3::new(-2560.0, -2560.0, 0.0), Vector3::new(2560.0, 2560.0, 5120.0)),
        8,
        mats,
        &options,
    )
    .into_vec()
    {
        map.add_solid(solid);
    }

    write_test_vmf(map.to_lower());

    panic!("done")
}

#[ignore]
#[test]
fn sphere_disp_test() {
    dbg!();
    const TRUTH_PATH: &str = "/home/redram/Documents/disp_test.vmf";
    let truth_input = std::fs::read_to_string(TRUTH_PATH).unwrap();
    let truth_vmf =
        vmf_parser_nom::parse::<&str, vmf_parser_nom::error::VerboseError<_>>(&truth_input)
            .unwrap();
    let block = &truth_vmf.blocks[3].blocks[0];
    // eprintln!("{}", block);
    let truth_sphere = block_to_solid(block);
    // dbg!(truth_sphere);
    // panic!("e");

    let mut map = Map::default();
    let options = SolidOptions { world_align: false, ..SolidOptions::default() };
    let sphere_options = SphereOptions { size: Displacement::power_to_len(3) };

    let mats = [
        // &Material::new("tools/toolsnodraw"),
        // &Material::new("tools/toolsnodraw"),
        &Material::new("DEV/DEV_MEASUREWALL01C"),
    ];
    const SIZE_X: f32 = 512.0;
    const SIZE_Y: f32 = 1024.0;
    const SIZE_Z: f32 = 128.0;
    // const SIZE_X: f32 = 128.0;
    // const SIZE_Y: f32 = 128.0;
    // const SIZE_Z: f32 = 128.0;
    let sphere = sphere_disp(
        // &Bounds::new(Vector3::new(-256.0, -256.0, -256.0), Vector3::new(256.0, 256.0, 256.0)),
        &Bounds::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(SIZE_X, SIZE_Y, SIZE_Z)),
        // &Bounds::new(Vector3::new(-2560.0, -2560.0, 0.0), Vector3::new(2560.0, 2560.0, 5120.0)),
        mats,
        &options,
        &sphere_options,
    );

    let sphere = sphere.into_vec()[0].clone();

    map.add_solid(sphere.clone());

    let vmf = map.to_lower();

    // println!("{:#}", vmf);
    write_test_vmf(vmf);

    let mut i = 0;
    for (truth, output) in truth_sphere.sides.iter().zip(sphere.sides.iter()) {
        // for (truth, output) in truth.d.iter().zip(sp)
        let truth_normals = &truth.disp.as_ref().unwrap().normals;
        let output_normals = &output.disp.as_ref().unwrap().normals;
        for (truth, output) in truth_normals.inner.iter().zip(output_normals.inner.iter()) {
            // Verify signs / diffs
            // Seems to not matter thats is way different tho? still ellipsoidal in correct way
            // let dif = truth.clone() - output;
            // eprintln!("truth {:<40} output {:<40} dif {:<40}", truth.to_string(), output.to_string(), dif.to_string());
            // eprintln!("truth {:<40} output {:<40} dif {:<40}", truth.to_string(), output.to_string(), dif.to_string());
            if i % 15 == 0 {
                eprintln!();
            }
            let truth = [
                truth.x.is_sign_negative() as usize,
                truth.y.is_sign_negative() as usize,
                truth.z.is_sign_negative() as usize,
            ];
            let output = [
                output.x.is_sign_negative() as usize,
                output.y.is_sign_negative() as usize,
                output.z.is_sign_negative() as usize,
            ];
            for (t, o) in truth.into_iter().zip(output) {
                if t == o {
                    print!("_");
                } else {
                    print!("{}", &"+-"[o.. o + 1]);
                }
            }
            print!("  ");

            // eprint!(
            //     "{}{}{} {}{}{}  ",
            //     &"+-"[truth.x.is_sign_negative() as usize..truth.x.is_sign_negative() as usize + 1],
            //     &"+-"[truth.y.is_sign_negative() as usize..truth.y.is_sign_negative() as usize + 1],
            //     &"+-"[truth.z.is_sign_negative() as usize..truth.z.is_sign_negative() as usize + 1],
            //     &"+-"[output.x.is_sign_negative() as usize
            //         ..output.x.is_sign_negative() as usize + 1],
            //     &"+-"[output.y.is_sign_negative() as usize
            //         ..output.y.is_sign_negative() as usize + 1],
            //     &"+-"[output.z.is_sign_negative() as usize
            //         ..output.z.is_sign_negative() as usize + 1],
            // );
            i += 1;
        }
    }

    // TODO ADD SOLID2 STUFF, RECONCILE RANODM FLAOTING FNS
    unimplemented!("cube stretched to sphere as disp")
}

#[test]
fn sphere2() {
    unimplemented!("cube stretch to sphere as brush")
}

#[ignore]
#[test]
fn test_order() {
    dbg!();
    // const TRUTH_PATH: &str = "/home/redram/Documents/disp_test.vmf";
    // let truth_input = std::fs::read_to_string(TRUTH_PATH).unwrap();
    // let truth_vmf =
    //     vmf_parser_nom::parse::<&str, vmf_parser_nom::error::VerboseError<_>>(&truth_input)
    //         .unwrap();
    // let block = &truth_vmf.blocks[3].blocks[0];
    // eprintln!("{}", block);
    // let truth_sphere = block_to_solid(block);
    // dbg!(truth_sphere);
    // panic!("e");

    let mut map = Map::default();
    let options = SolidOptions { world_align: false, ..SolidOptions::default() };
    let sphere_options = SphereOptions { size: Displacement::power_to_len(2) };

    let mats = [
        // &Material::new("tools/toolsnodraw"),
        // &Material::new("tools/toolsnodraw"),
        &Material::new("DEV/DEV_MEASUREWALL01C"),
    ];
    const SIZE: f32 = 128.0;
    let sphere = sphere_disp(
        // &Bounds::new(Vector3::new(-256.0, -256.0, -256.0), Vector3::new(256.0, 256.0, 256.0)),
        &Bounds::new(Vector3::new(-SIZE, -SIZE, -SIZE), Vector3::new(SIZE, SIZE, SIZE)),
        // &Bounds::new(Vector3::new(-2560.0, -2560.0, 0.0), Vector3::new(2560.0, 2560.0, 5120.0)),
        mats,
        &options,
        &sphere_options,
    );

    let mut sphere = sphere.into_vec()[0].clone();

    for side in sphere.sides.iter_mut() {
        // dbg!(&(side.disp.as_ref()).unwrap().plane);
        dbg!(&(side.disp.as_ref()).unwrap().bottom_right);
        let mut i = 0f32;
        let side_normal = side.plane.normal();
        let normals = &mut side.disp.as_mut().unwrap().normals;
        for normal in normals.inner.iter_mut() {
            *normal = side_normal.clone();
        }
        let distances = &mut side.disp.as_mut().unwrap().distances;
        for distance in distances.inner.iter_mut() {
            *distance = i;
            i += 1.0 + i.sqrt();
        }
    }

    // dbg!(&sphere);

    // let disp = sphere.sides[0].disp.as_ref().unwrap();
    // let disp_info = disp.clone().into_disp_info();
    // let disp_info_block = disp_info.to_lower();
    // println!("{}", disp_info_block);

    map.add_solid(sphere.clone());

    let vmf = map.to_lower();

    // println!("{:#}", vmf);
    write_test_vmf(vmf);

    // for (truth, output) in truth_sphere.sides.iter().zip(sphere.sides.iter()) {
    //     // for (truth, output) in truth.d.iter().zip(sp)
    //     let truth_normals = &truth.disp.as_ref().unwrap().normals;
    //     let output_normals = &output.disp.as_ref().unwrap().normals;
    //     for (truth, output) in truth_normals.inner.iter().zip(output_normals.inner.iter()) {
    //         let dif = truth.clone() - output;
    //         eprintln!("truth {:<40} output {:<40} dif {:<40}", truth.to_string(), output.to_string(), dif.to_string());
    //     }
    // }

    // TODO ADD SOLID2 STUFF, RECONCILE RANODM FLAOTING FNS
    unimplemented!("test order")
}
