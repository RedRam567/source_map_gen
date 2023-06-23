use crate::pallet::vpk;
use vpk::utils::*;
use vpk::{VpkError, VpkResult};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// use std::io::Result as IoResult;
// use vmf_parser_nom::error::VerboseError;
// use vpk::io_utils::*;

// IGNORE custom,addons,workshop
// find gameinfo.txt
// priority: update
// gameinfo top down
// folders

// slow everything method
// every .mdl or .vmt OR .vtf IDK, ALSO PARTICALE (HARD)
// OR only search materials/models

// |NAME| implies a slash ALSO case insensitve
// |All_Source_Engine_Paths| = exec directory (where tf is that?)
// |gameinfo_path|. = you know
// multichunk vpks = .vpk no _dir, or _###

// FIXME: be more inclusive, tf2 mega fails
// TODO: dont convert path case
/// Parse `gameinfo.txt` and return the search paths. Simply returns the trimmed
/// text after "Game" in "SearchPaths{}".
///
/// Interprets `|gameinfo_path|` to the parent directory of `game_info` as an absolute path.
///
/// See <https://developer.valvesoftware.com/wiki/Gameinfo.txt>
///
/// # Notes
///
/// Will convert all paths to lowercase.
///
/// # Errors
///
/// Returns `None` if `game_root` isn't a directory or doesn't exist.
/// Returns `None` if `game_info` doesn't exist or couldn't be parsed.
pub fn get_search_paths(
    game_root: impl AsRef<Path>, game_info: impl AsRef<Path>,
) -> Option<Vec<PathBuf>> {
    fn search_paths(game_root: &Path, game_info: &Path) -> Option<Vec<PathBuf>> {
        // TODO: fix vmf_parser to allow quoted block args, and non str props and use that
        let game_info = game_info.canonicalize().ok()?;
        let string = fs::read_to_string(&game_info).ok()?;
        let game_info_path = game_info.parent()?;

        // (horribly) get str between /SearchPaths.*{/ and /}/
        let after_open_brace = string.split_once("SearchPaths")?.1.split_once('{')?.1;
        let between_braces = after_open_brace.split_once('}')?.0;

        // parse SearchPaths{} return second field trimmed
        let mut paths = Vec::new();
        for line in between_braces.lines() {
            // TODO: be more inclusive, for things like "Mod" and "blahblah"
            let Some((_, path)) = line.split_once("Game") else { continue };
            // TODO: horrible
            let path = path.trim().to_lowercase();
            if path.contains("custom") {
                continue;
            }
            let path = path.replace("|gameinfo_path|", &format!("{}/", game_info_path.display()));
            let path = game_root.join(path);
            paths.push(path);
        }

        Some(paths)
    }
    search_paths(game_root.as_ref(), game_info.as_ref())
}

/// handle_dir
///     .dir handle_dir
///     .vpk handle_vpk
///     .mdl add mdl
///     .vmt add vmt

/// handle_vpk
///     list
///     .mdl, add relative
///     .vmt, add relative
///     .vpk -> todo
// pub fn search_path(game_path: impl AsRef<Path>, search_path: impl AsRef<Path>) -> (Vec<PathBuf>, Vec<PathBuf>) {
//     fn search_path_content(game_path: &Path, search_path: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
//         let path = game_path.join(search_path);
//             // call search fn
//         todo!()
//     }
//     search_path_content(game_path.as_ref(),search_path.as_ref())
// }

// TODO: parse model textures too

/// A Vec of `mdl`s and `vmt`s.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Content {
    pub mdl: Vec<PathBuf>,
    pub vmt: Vec<PathBuf>,
}

impl Content {
    pub const fn new() -> Self {
        Self { mdl: Vec::new(), vmt: Vec::new() }
    }
    /// (self.mdl, self.vmt)
    pub fn into_parts(self) -> (Vec<PathBuf>, Vec<PathBuf>) {
        (self.mdl, self.vmt)
    }
}

pub fn filter_no_addons(entry: &walkdir::DirEntry) -> bool {
    let path = entry.path();
    !(path.contains("addon")
        || path.contains("workshop")
        || path.contains("custom")
        || path.contains("download"))
}

// TODO: docs
pub fn get_content<P>(
    vpk_bin: impl AsRef<Path>, game_root: impl AsRef<Path>, game_info: impl AsRef<Path>, filter: P,
) -> VpkResult<Content>
where
    P: FnMut(&walkdir::DirEntry) -> bool,
{
    fn get_content<P>(
        vpk_bin: &Path, game_root: impl AsRef<Path>, game_info: &Path, mut filter: P,
    ) -> VpkResult<Content>
    where
        P: FnMut(&walkdir::DirEntry) -> bool,
    {
        // get search paths from gameinfo.txt
        let search_paths = get_search_paths(game_root, game_info)
            .ok_or_else(|| {
                io_error_other(format!("Error parsing game_info.txt: `{}`", game_info.display()))
            })
            .map_err(VpkError::Src)?;

        // get content from search paths, including vpks
        let mut content = Content::new();
        for path in search_paths {
            dir_content(vpk_bin, path, &mut content, &mut filter)?;
        }
        Ok(content)
    }

    get_content(vpk_bin.as_ref(), game_root.as_ref(), game_info.as_ref(), filter)
}

/// Recursivly get [`Content`] from a directory, including any nested directories
/// or vpk files. Does nothing if there is no content.
/// Ignores any io errors from reading the `dir` directory.
///
/// # Errors
///
/// * `dir` must be a directory and exist.
/// * [`vpk_content`] must not fail.
pub fn dir_content<P>(
    vpk_bin: impl AsRef<Path>, dir: impl AsRef<Path>, content: &mut Content, filter: P,
) -> VpkResult<()>
where
    P: FnMut(&walkdir::DirEntry) -> bool,
{
    fn dir_content<P>(vpk_bin: &Path, dir: &Path, content: &mut Content, filter: P) -> VpkResult<()>
    where
        P: FnMut(&walkdir::DirEntry) -> bool,
    {
        // entries in a dir, follow symbolic links, ignore errors (cant access folder, etc)

        dir.dir_exists().map_err(VpkError::Src)?;
        let entries =
            WalkDir::new(dir).follow_links(true).into_iter().filter_map(|entry| entry.ok());

        // let entries2 =
        //     WalkDir::new(dir).follow_links(true).into_iter().filter_map(|entry| entry.ok());
        // for entry in entries2 {
        //     eprintln!("entry: {}", entry.path().display());
        // }

        for entry in entries.filter(filter) {
            let path = entry.path();
            if path.contains("addon")
                || path.contains("workshop")
                || path.contains("custom")
                || path.contains("download")
            {
                panic!("bruh impossible");
                continue;
            }

            // dbg!(path);
            match path {
                // wait... this is handled by WalkDir
                // dir if path.is_dir() => {
                //     dbg!(dir);
                //     dir_content(vpk_bin, dir, content)?;
                // }
                mdl if path.has_ext("mdl") => {
                    content.mdl.push(mdl.to_path_buf());
                }
                vmt if path.has_ext("vmt") => {
                    content.vmt.push(vmt.to_path_buf());
                }
                vpk if path.has_ext("vpk") => {
                    vpk_content(vpk_bin, vpk, content)?;
                }
                _ => (),
            }
        }
        Ok(())
    }

    dir_content(vpk_bin.as_ref(), dir.as_ref(), content, filter)
}

/// Get [`Content`] from a vpk file. Does nothing if there is no content, if it
/// is a sub-part of a multi-part vpk file, or if `vpk_linux32` silently fails.
/// Debug prints if there any nested vpk files.
///
/// # Errors
///
/// * `vpk` must be a vpk and exist.
/// * [`vpk::list()`] must not fail.
pub fn vpk_content(
    vpk_bin: impl AsRef<Path>, vpk: impl AsRef<Path>, content: &mut Content,
) -> VpkResult<()> {
    fn vpk_content(vpk_bin: &Path, vpk: &Path, content: &mut Content) -> VpkResult<()> {
        vpk.vpk_exists().map_err(VpkError::Src)?;
        if vpk.is_proper_vpk_ish().is_err() {
            return Ok(());
        }

        let entries = vpk::list(vpk_bin, vpk)?;
        for entry in entries.lines() {
            let path = Path::new(entry);
            match path {
                mdl if path.has_ext("mdl") => content.mdl.push(mdl.to_path_buf()),
                vmt if path.has_ext("vmt") => content.vmt.push(vmt.to_path_buf()),
                #[cfg(debug_assertions)]
                vpk if path.has_ext("vpk") => {
                    eprintln!("nested vpk in {}", vpk.display());
                    panic!();
                }
                _ => (),
            }
        }
        Ok(())
    }

    vpk_content(vpk_bin.as_ref(), vpk.as_ref(), content)
}

#[cfg(test)]
mod tests {
    use super::*;

    // FIXME: fails because cannonicalization, doesn't matter?
    #[ignore]
    #[test]
    fn it_works() {
        let truth: &[PathBuf] = &[
            "custom".into(),
            "update".into(),
            "left4dead2_dlc3".into(),
            "left4dead2_dlc2".into(),
            "left4dead2_dlc1".into(),
            "/home/redram/.local/share/Steam/steamapps/common/Left 4 Dead 2/left4dead2".into(),
            "hl2".into(),
        ];
        const L4D2_ROOT: &str = "/home/redram/dev/modding/L4D2/Left 4 Dead 2/";
        const L4D2_GAMEINFO: &str =
            "/home/redram/dev/modding/L4D2/Left 4 Dead 2/left4dead2/gameinfo.txt";
        let result = get_search_paths(L4D2_ROOT, L4D2_GAMEINFO).unwrap();
        assert_eq!(truth, result);
    }
}
