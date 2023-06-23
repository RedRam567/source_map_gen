mod pallet_lib;
/// TODO:LOC: TODO: spin off into crate (include vmf? make giant everything lib?)
pub(crate) mod vpk;

use pallet_lib::{filter_no_addons, get_content, get_search_paths};
use source_map_gen::generation::Bounds;
use source_map_gen::map::{Entity, Map};
use source_map_gen::prelude::*;
use source_map_gen::vmf::ToLower;
use std::fs::{self, OpenOptions};
use std::io::Write;
use vmf_parser_nom::ast::Property;
use vpk::utils::PathExt;

const DEV_FLOOR: &str = "dev/dev_measuregeneric01b";
const HINT: &str = "tools/toolshint";

pub fn main() {
    // let home: PathBuf = std::env::var("HOME").unwrap().into();
    // let game_info = TF2_TF.join("gameinfo.txt");

    const VPK_CMD: &str =
        "/home/redram/.local/share/Steam/steamapps/common/Team Fortress 2/bin/vpk_linux32";

    let game_root: &str = "/home/redram/.local/share/Steam/steamapps/common/Left 4 Dead 2";
    let game_info =
        "/home/redram/.local/share/Steam/steamapps/common/Left 4 Dead 2/left4dead2/gameinfo.txt";

    let search_paths = get_search_paths(game_root, game_info);
    // writeln!()
    // std::fs::write();
    let mut tty = OpenOptions::new().write(true).open("/dev/tty").unwrap();
    writeln!(tty, "{:#?}", search_paths).unwrap();

    let content = get_content(VPK_CMD, game_root, game_info, filter_no_addons).unwrap();

    let bad_mdls = [
        "vgui",
        "models",
        "props",
        "particle",
        "effects",
        "overlay",
        "engine",
        "debug",
        "decal",
        "ti",
        "voice",
        "sprites",
        "cable",
        "skybox",
        "tools/",
        "foliage/",
        "liquids/",
        "dev/",
        "graffiti/",
        "editor/",
        "water",
        "card",
        "sign",
        "city_",
        "console/",
        "tree",
        "detail",
        "growth",
        "fence",
        "rail",
        "ladder",
        "truss",
        "grate",
        "edge",
        "gate",
        "river",
        "gm_forest",
        "trim",
        "curb",
        "puddle",
        "stain",
        "hedges",
        "scape",
        "trans",
        "helicopt",
        "water",
        "sewage",
        "glass",
        "crack",
        "vent",
        "blend",
    ];
    // RIP signs, dev, graffiti, blend, glass
    let (mdls, vmts) = content.into_parts();
    let mut mdls = mdls.iter().filter(|mdl| !(mdl.contains("anim"))).collect::<Vec<_>>();
    let mut vmts = vmts
        .iter()
        .filter(|vmt| !(contains_any(vmt.to_str().unwrap(), &bad_mdls)))
        .collect::<Vec<_>>();
    mdls.sort();
    vmts.sort();

    // mdls of interest
    // anim, gibs, broken (brokenforklight false positive), breakable(_chunk), props_destruction, props_debris, _dmg, _break
    // stadium_bench_custom(abcdefghij) (wtf?), breakable, _part (rare), _break, smash_break
    // NEGATIVE NUMBERS, zombiebreakwall..., _phy, _phyiscichs, _break00X_YY, _break21, gestures,
    // v_, NOT w_ (world), ghostanim (common), hybridphysx, infected_w (gibs)
    // "models/extras/info_speech_australium.mdl" (??)
    // editor, effect, error.mdl

    // vmts of interest:
    // vgui, models, dev, debug, decal, editor, effects, engine, liquids, overlays, sprites, lights,
    // particles (BREAKS HAMMER), tools, voice (small)
    // "materials/customtext/gc textures/..." (wtf??)
    // for mdl in mdls {
    //     println!("{mdl:?}");
    // }
    // for vmt in vmts {
    //     eprintln!("{vmt:?}");
    // }

    const PALLET_SIZE_X: i32 = 128;
    const PALLET_SIZE_Y: i32 = 128;
    const FLOOR_TOP: f32 = -128.0;
    const FLOOR_BOTTOM: f32 = -256.0;
    const CEILING: f32 = 2048.0 + FLOOR_BOTTOM;
    /// Place hint cubes at this grid
    const HINT_SIZE: usize = 128;

    // setpos 1920 -2040 2600; setang 90 90 0
    // -radius_override 1025 -> basicly max range from top @ 2048

    let mut map = Map::default();
    map.defaults_l4d2();
    map.entities.push(Entity {
        props: vec![
            Property::new("classname", "info_player_start"),
            Property::new("angles", "0 0 0"),
            Property::new("spawnflags", "0"),
            // must be 1 unit above or leak somehow O_o
            Property::new("origin", "0 0 1"),
        ],
        // props: vec![
        //     Property::new("classname", "info_player_start"),
        //     Property::new("angles", "0 0 0"),
        //     Property::new("spawnflags", "0"),
        //     // must be 1 unit above or leak somehow O_o
        //     Property::new("origin", "0 0 1"),
        // ],
    });

    let len = vmts.len();
    let row_len = (len as f64).sqrt().ceil() as usize;

    // TODO: just translate
    let floor_x = -PALLET_SIZE_X;
    let floor_y = PALLET_SIZE_Y;
    let floor_size = row_len as i32 * PALLET_SIZE_X;
    map.options.cordon = Some(Bounds::new(
        Vector3::new(floor_x as f32, floor_y as f32, FLOOR_BOTTOM),
        Vector3::new((floor_x + floor_size) as f32, (floor_y - floor_size) as f32, CEILING),
    ));

    let mut row: i32 = 0;
    let mut col: i32;
    if vmts.len() > 1024 {
        eprintln!("Too many unique textures: {}", vmts.len());
    } else {
        eprintln!("Num textures: {}", vmts.len());
    }
    // make floor
    for chunk in vmts.chunks(row_len) {
        col = -1;
        for vmt in chunk {
            let bounds =
                xy_to_bounds(col, row, PALLET_SIZE_X, PALLET_SIZE_Y, FLOOR_TOP, FLOOR_BOTTOM);

            let texture = vmt.to_str().unwrap();
            let texture = texture
                .strip_prefix("materials/")
                .unwrap_or(texture)
                .strip_suffix(".vmt")
                .unwrap_or(texture);
            let solid = pallet_solid(bounds, texture);
            // if col == 2 {
            //     dbg!(&solid);
            //     panic!();
            // }
            map.solids.push(solid);
            col += 1;
        }
        // handle missing bit for niceness
        if chunk.len() < row_len {
            for _ in chunk.len()..row_len {
                let bounds =
                    xy_to_bounds(col, row, PALLET_SIZE_X, PALLET_SIZE_Y, FLOOR_TOP, FLOOR_BOTTOM);
                map.solids.push(pallet_solid(bounds, DEV_FLOOR));
                col += 1;
            }
        }
        row -= 1;
    }

    // make hint brushes
    for col in (-(HINT_SIZE as i32)..=floor_size).step_by(HINT_SIZE) {
        for row in (0..=floor_size).step_by(HINT_SIZE) {
            let row = -row;
            // let bounds = xy_to_bounds(col, row, PALLET_SIZE_X, PALLET_SIZE_Y, CEILING, FLOOR_TOP);
            let start = Vector3::new(col as f32, row as f32, FLOOR_TOP);
            let end = Vector3::new(start.x + HINT_SIZE as f32, start.y + HINT_SIZE as f32, CEILING);
            let bounds = Bounds::new(start, end);
            let solid = Map::cube_str(bounds, [HINT; 6]);
            map.solids.push(solid);
        }
    }

    // dbg!(map);
    // let output_path = "/home/redram/dev/modding/L4D2/custom/maps/output.vmf";
    let output_path =
        "/home/redram/.local/share/Steam/steamapps/common/Left 4 Dead 2/custom/maps/pallet.vmf";
    _ = fs::remove_file(output_path);
    let mut output = OpenOptions::new().write(true).create(true).open(output_path).unwrap();

    let vmf = map.to_lower();

    println!("{}", vmf);
    writeln!(output, "{}", vmf).unwrap();

    // TODO: proc gen
    // TODO: merge vpk, vmt, vmt
}

fn xy_to_bounds(
    col: i32, row: i32, col_size: i32, row_size: i32, top: f32, bottom: f32,
) -> Bounds<f32> {
    let x_start = col * col_size;
    let x_end = x_start + col_size;
    let y_start = row * row_size;
    let y_end = y_start + row_size;

    let bottom = Vector3::new(x_start as f32, y_start as f32, bottom);
    let top = Vector3::new(x_end as f32, y_end as f32, top);
    Bounds::new(top, bottom)
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

// bounds, top texture
fn pallet_solid(bounds: Bounds<f32>, top_texture: &str) -> Solid {
    // let sides = Side::ne
    // Solid::new(sides)
    // proc_gen2::generation::Map::cube_str()

    // let other = "tools/toolsskip"; // skip makes vbsp merge a lot of cubes
    // let other = "tools/toolsnodraw";
    let other = DEV_FLOOR;

    let textures = [top_texture, other, other, other, other, other];
    source_map_gen::map::Map::cube_str(bounds, textures)
}

// read every texture and model
// get model size somehow // easy
// texture every 256x256 or smth on ground
// model ~512 in air above
// "packed" models for idk 4096

// OOO bigger props -> float higher
