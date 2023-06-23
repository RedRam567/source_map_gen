//! Simple wrapper around vpk_linux32 to actually get it to work
// TODO: docs
// TODO: extract_once doc
// TODO: options and x_once optional dest dir
// TODO: when is cannonicalize nessessary?

// pub(crate) mod io_utils;

use std::collections::HashSet;
use std::io::Error as IoError;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::{env, fs};

use utils::*;
use rayon::prelude::ParallelIterator;
use rayon::str::ParallelString;

pub(crate) mod utils {
    use std::ffi::OsStr;
    use std::io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult};
    use std::path::Path;

    // weirdish names to prevent collisions and strangness.

    /// Extention trait for a `Path` that check paths, returning as a `std::io::Result`
    /// instead of a bool.
    pub trait PathExt {
        /// Checks if it is a file and exists.
        fn file_exists(&self) -> IoResult<()>;
        /// Checks if it is a directory and exists.
        fn dir_exists(&self) -> IoResult<()>;
        /// Checks if it is a file and has a `.vpk` file extension and exists.
        fn vpk_exists(&self) -> IoResult<()>;
        /// Checks if the path has the extension, without the dot.
        /// See [`std::path::Path::extension()`]
        fn has_ext_res(&self, ext: impl AsRef<OsStr>) -> IoResult<()>;
        /// Checks if the path has the extension, without the dot.
        /// See [`std::path::Path::extension()`]
        fn has_ext(&self, ext: impl AsRef<OsStr>) -> bool {
            self.has_ext_res(ext).is_ok()
        }
        fn is_proper_vpk_ish(&self) -> IoResult<()>;
        fn contains(&self, str: &str) -> bool;
    }

    impl PathExt for Path {
        fn file_exists(&self) -> IoResult<()> {
            if !self.is_file() {
                Err(io_error_other(format!("`{}` isn't a file or doesn't exist", self.display())))
            } else {
                Ok(())
            }
        }

        fn dir_exists(&self) -> IoResult<()> {
            if !self.is_dir() {
                Err(io_error_other(format!("`{}` is not a directory", self.display())))
            } else {
                Ok(())
            }
        }

        fn vpk_exists(&self) -> IoResult<()> {
            self.has_ext_res("vpk")?;
            self.file_exists()?;
            Ok(())
        }

        // hot function
        fn has_ext_res(&self, ext: impl AsRef<OsStr>) -> IoResult<()> {
            fn has_ext(path: &Path, ext: &OsStr) -> IoResult<()> {
                if path.extension() != Some(ext) {
                    Err(io_error_other("invalid extension"))
                    // Err(io_error_other(format!(
                    //     "`{}` must have a .{} extension",
                    //     path.display(),
                    //     ext.to_str().unwrap_or("NON UTF-8 EXTENSION")
                    // )))
                } else {
                    Ok(())
                }
            }
            has_ext(self, ext.as_ref())
        }

        // TODO: FIXME: swap out all use of vpk_exist
        // or has_ext("vpk") with a new `is_good_vpk`
        // Checks if this vpk as a sub-part of a multi-part vpk
        // ("path/to/multipart/vpk_123.vpk").
        fn is_proper_vpk_ish(&self) -> IoResult<()> {
            fn is_bad(path: &Path) -> Option<()> {
                // vpk_name_000.vpk
                let stem = path.file_stem()?.to_str()?;
                // vpk_name_000
                let nums = stem.rsplit_once('_')?.1;
                // 000
                if nums.len() != 3 {
                    return None;
                }
                if nums.bytes().all(|b| b.is_ascii_digit()) {
                    return Some(());
                }
                None
            }
            self.has_ext("vpk");
            if is_bad(self).is_some() {
                Err(io_error_other(format!(
                    "`{}` is a sub-part of a multi-part .vpk file",
                    self.display()
                )))
            } else {
                Ok(())
            }
        }

        fn contains(&self, str: &str) -> bool {
            if let Some(path) = self.to_str() {
                path.contains(str)
            } else {
                false
            }
        }
    }

    /// Copy paste of [`std::io::Error::other()`](https://doc.rust-lang.org/std/io/struct.Error.html#method.other)
    pub fn io_error_other<E>(error: E) -> IoError
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        IoError::new(IoErrorKind::Other, error.into())
    }

    // `String::from_utf8` panics on non-utf8 and `String::from_utf8_lossy().into_owned()`
    // always allocates, even if you have a Vec input.
    /// Converts a `Vec` of bytes into a string, reusing the allocation, converting
    /// invalid bytes to the replacement character: `U+FFFD: �`.
    ///
    /// Adapted from [`rust-url`](https://github.com/servo/rust-url).
    /// See  [`String::from_utf8_lossy`].
    pub fn string_from_utf8_lossy(bytes: Vec<u8>) -> String {
        let str = match String::from_utf8_lossy(&bytes) {
            // valid utf8
            std::borrow::Cow::Borrowed(s) => s,
            // invalid bytes replaced with replacement character
            std::borrow::Cow::Owned(s) => return s,
        };
        debug_assert!(
            str.as_bytes() == bytes,
            "String::from_utf8_lossy changed the bytes somehow O_O"
        );
        // Reuse input allocation
        // SAFETY: safe as from_utf8_lossy already checked it for use
        unsafe { String::from_utf8_unchecked(bytes) }
    }
}

// vpk_cmd canon

// list vpk output

// create dest_dir
// dest dir canon
// src vpk canon
// crate dest real

// extract one

// src dir canon
// create dest dir
// cmd
// move out to dest

// vpk_cmd canon`
// create -> io
// srcvpk
// srcdir canon
// dest vpk

pub type VpkResult<T> = std::result::Result<T, VpkError>;

#[derive(Debug)]
pub enum VpkError {
    /// The source file or directory doesn't exist.
    Src(IoError),
    /// The destination file or directory doesn't exist.
    Dest(IoError),
    /// Miscellaneous io error. Usually creating file/folders.
    MiscIo(IoError),
    /// Cmd path is invalid.
    CmdPath(IoError),
    /// Error starting a command.
    CmdExec(IoError),
    /// A command exited with a failure exit status.
    CmdFailed(ExitStatus),
}

/// Convience for
impl From<IoError> for VpkError {
    fn from(error: IoError) -> Self {
        VpkError::MiscIo(error)
    }
}

impl std::fmt::Display for VpkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VpkError::Src(err) => write!(f, "Source dir/vpk error: {err}"),
            VpkError::Dest(err) => write!(f, "Destination dir/vpk error: {err}"),
            VpkError::MiscIo(err) => write!(f, "Misc io error: {err}"),
            VpkError::CmdPath(err) => write!(f, "Command path error: {err}"),
            VpkError::CmdExec(err) => write!(f, "Command exec error: {err}"),
            VpkError::CmdFailed(err) => write!(f, "Command failed: {err}"),
        }
    }
}

impl std::error::Error for VpkError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    /// 0 means automatically determine threads.
    /// > 1 means automatically determine threads with `Rayon`
    pub threads: u32,
    pub verbose: bool,
    pub show_progress: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self { threads: 1, verbose: Default::default(), show_progress: Default::default() }
    }
}

// NOTE: on error handling
// all vpk functions verify src exist and create dest folders if they dont exist already
// canonicalize() or parent() return Src or Dest errors
// creating/renaming files/dirs return MiscIo errors

/// Runs `vpk_linux32` with the appropriate `LD_LIBRARY_PATH` enviroment variable
/// set to get it to run.
/// UNTESTED: MIGHT work on Windows with vpk.exe.
///
/// Port of the script on [Valve Developer Community](https://developer.valvesoftware.com/wiki/VPK#Linux_.2F_Unix)
pub fn vpk_cmd(vpk: impl AsRef<Path>) -> VpkResult<Command> {
    fn vpk_cmd(vpk: &Path) -> VpkResult<Command> {
        vpk.file_exists().map_err(VpkError::CmdPath)?;
        let vpk = env::current_dir()?.join(vpk);

        let parent = vpk.parent().ok_or_else(|| {
            VpkError::CmdPath(io_error_other(format!(
                "`{}` has no parent directory",
                vpk.display()
            )))
        })?;
        parent.dir_exists().map_err(VpkError::CmdPath)?;

        let mut cmd = Command::new(&vpk);
        cmd.env("LD_LIBRARY_PATH", parent);
        Ok(cmd)
    }
    vpk_cmd(vpk.as_ref())
}

/// Get a list of all the files in a vpk, separated by a newline.
///
/// See [`String::from_utf8_lossy`].
///
/// # Note
/// Converts invalid UTF-8 bytes to the replacement character `U+FFFD`.
///
/// # Errors
/// * `cmd_path` must be the path to vpk_linux32.
/// * `vpk` must exist.
/// * Returns `Err(VpkError::CmdFailed)` if the listing failed, although it seems
/// to just silently fail.
pub fn list(cmd_path: impl AsRef<Path>, vpk: impl AsRef<Path>) -> VpkResult<String> {
    fn list(path: &Path, vpk: &Path) -> VpkResult<String> {
        vpk.vpk_exists().map_err(VpkError::Src)?;
        let output =
            vpk_cmd(path)?.args(["l".as_ref(), vpk]).output().map_err(VpkError::CmdExec)?;

        // vpk_linux32 seems to just silently fail
        if !output.status.success() {
            return Err(VpkError::CmdFailed(output.status));
        }

        Ok(string_from_utf8_lossy(output.stdout))
    }
    list(cmd_path.as_ref(), vpk.as_ref())
}

/// Extract a vpk to a folder, one file at a time. If you dont do this, it breaks
/// and gives weird output, filenames, and/or crashes.
pub fn extract_all(
    cmd_path: impl AsRef<Path>, src_vpk: impl AsRef<Path>, dest_dir: impl AsRef<Path>,
    options: Options,
) -> VpkResult<()> {
    fn extract_all(
        cmd_path: &Path, src_vpk: &Path, dest_dir: &Path, options: Options,
    ) -> VpkResult<()> {
        let files = list(cmd_path, src_vpk)?;

        // `vpk x VPK FILE` doesnt create folder structure itself.
        // This collects all folders and sub/parent folders so we can make all
        // folders in one shot, to minimize calls to the file system.
        let parents: HashSet<&Path> = files
            .lines()
            .flat_map(|file| {
                Path::new(file)
                    .ancestors()
                    .skip(1)
                    .filter(|parent| !parent.as_os_str().is_empty())
            })
            .collect();

        // TODO: prevent unnessessary dir creation for nested dirs
        for dir in parents {
            if options.verbose {
                eprintln!("creating dir `{}`", dir.display());
            }
            fs::create_dir_all(dest_dir.join(dir))?;
        }

        // extract files one by one
        fs::create_dir_all(dest_dir)?;
        let src_vpk = src_vpk.canonicalize().map_err(VpkError::Src)?;
        files
            .par_lines()
            .map(|file| extract_one(cmd_path, &src_vpk, dest_dir, file))
            .collect()
    }

    extract_all(cmd_path.as_ref(), src_vpk.as_ref(), dest_dir.as_ref(), options)
}

// TODO: docs and optional
// TODO: inputs must be valid (keep unchecked version around for extract_all)
fn extract_one(
    cmd_path: impl AsRef<Path>, src_vpk: impl AsRef<Path>, dest_dir: impl AsRef<Path>,
    entry: impl AsRef<Path>,
) -> VpkResult<()> {
    fn extract_one(
        cmd_path: &Path, src_vpk: &Path, dest_dir: &Path, entry: &Path,
    ) -> VpkResult<()> {
        let status = vpk_cmd(cmd_path)?
            .args(["x".as_ref(), src_vpk, entry])
            .current_dir(dest_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(VpkError::CmdExec)?;
        if !status.success() {
            return Err(VpkError::CmdFailed(status));
        }
        Ok(())
    }

    extract_one(cmd_path.as_ref(), src_vpk.as_ref(), dest_dir.as_ref(), entry.as_ref())
}

// TODO: copy folder to prevent clobbering
/// Archive the contents of `src_dir` into a vpk at its parent folder or
/// move into `dest_vpk` if it is `Some`. Creates any nessessary folders.
///
/// # Note
/// `vpk_valve32` always outputs to `src_dir/../src_dir.vpk`, so will overwrite anything there.
///
/// # Errors
/// * `src_dir` must be a exist and be directory. `dest_vpk` must have a `.vpk` extension.
/// * Errors if `dest_vpk` is on a different file system than `src_dir`.See [`fs::rename`]
/// * Can error when creating `dest_dir` parent folders
/// * Can error for all other file operations.
pub fn archive(
    cmd_path: impl AsRef<Path>, src_dir: impl AsRef<Path>, dest_vpk: Option<impl AsRef<Path>>,
) -> VpkResult<()> {
    // creates `dest_vpk` folders
    // archives `src_dir`
    // moves `src_dir.vpk` into `dest_vpk`
    fn archive(cmd_path: &Path, src_dir: &Path, dest_vpk: Option<&Path>) -> VpkResult<()> {
        // Create nessessary folders for dest_vpk
        if let Some(dest_vpk) = dest_vpk {
            // dont use is_vpk, doesnt exist yet
            dest_vpk.has_ext_res("vpk").map_err(VpkError::Dest)?;

            let dest_vpk = env::current_dir()?.join(dest_vpk);
            let dest_dir = dest_vpk
                .parent()
                .ok_or_else(|| {
                    io_error_other(format!("`{}` has no parent dir", dest_vpk.display()))
                })
                .map_err(VpkError::Dest)?;
            fs::create_dir_all(dest_dir)?;
        }

        // Archive path/to/src_dir -> path/to/src_dir.vpk
        src_dir.dir_exists().map_err(VpkError::Src)?;
        let src_dir = src_dir.canonicalize().map_err(VpkError::Src)?;
        let mut cmd = vpk_cmd(cmd_path).unwrap();
        let status = cmd
            // .args([Path::new("a"), &src_dir])
            .args([&src_dir])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(VpkError::CmdExec)?;

        if !status.success() {
            return Err(VpkError::CmdFailed(status));
        }

        // Move out_vpk to dest_vpk
        if let Some(dest_vpk) = dest_vpk {
            // path/to/src_dir -> path/to/src_dir.vpk
            let mut out_vpk = src_dir;
            out_vpk.set_extension("vpk");

            // std::fs really needs a move function
            // fs::rename renames OR moves (errors if cross-filessytems)
            fs::rename(out_vpk, dest_vpk)?;
        }

        Ok(())
    }

    // Some(T) -> Some(T.as_ref()) rigmarole
    let dest_vpk = dest_vpk.as_ref().map(|v| v.as_ref());
    archive(cmd_path.as_ref(), src_dir.as_ref(), dest_vpk)
}

#[cfg(test)]
mod tests {
    // At least right now: the truth folder holds the ground truth, and is never used apparently.
    // truth.vpk is made from that and is tested.

    // FIXME:
    #[cfg(not(unix))]
    compile_error!("can only be tested on Linux/Unix right now");

    use std::path::PathBuf;

    use lazy_static::lazy_static;

    use super::*;

    const TRUTH_DIR: &str = "test/truth";
    const TRUTH_VPK: &str = "test/truth.vpk";

    // nothing in std::fs is const >:(
    lazy_static! {
        static ref HOME: PathBuf = std::env::var("HOME").unwrap().into();
        static ref TF2: PathBuf = HOME.join(".local/share/Steam/steamapps/common/Team Fortress 2");
        static ref TF2_TF: PathBuf = TF2.join("tf");
        static ref VPK_CMD: PathBuf = TF2.join("bin/vpk_linux32");
        static ref TEST_DIR: &'static Path = Path::new("test");
    }

    // lazy_static outputs magic types
    // fixes using VPK_CMD as argument (normally only impls Deref<PathBuf>)
    impl AsRef<Path> for VPK_CMD {
        fn as_ref(&self) -> &Path {
            self.as_path()
        }
    }

    fn assert_env() {
        assert!(TF2.is_dir());
        assert!(TF2_TF.is_dir());
        assert!(VPK_CMD.is_file());

        let cwd = std::env::current_dir().unwrap();
        let cwd_test = cwd.join("test");
        assert_eq!(
            TEST_DIR.canonicalize().unwrap(),
            cwd_test.canonicalize().unwrap(),
            "bad current working directory"
        );
        assert!(TEST_DIR.is_dir());
        assert!(cwd_test.is_dir());

        assert!(Path::new(TRUTH_DIR).is_dir());
        assert!(Path::new(TRUTH_VPK).is_file());
    }

    // get base name of path as String
    fn filename(path: impl AsRef<Path>) -> String {
        path.as_ref().file_name().unwrap().to_string_lossy().into_owned()
    }

    #[test]
    fn list_test_tf2() {
        assert_env();
        eprintln!("TODO: horrible test as tf2 can just update");
        let tf2_vpk = TF2_TF.join("tf2_sound_misc_dir.vpk");

        let list = list(&VPK_CMD, tf2_vpk).unwrap();

        assert_eq!(3227, list.lines().count());
    }

    #[test]
    fn list_test() {
        assert_env();
        let list = list(&VPK_CMD, TRUTH_VPK).unwrap();
        let truth = "file1.txt\n\
            file2.txt\n\
            folder1/folder1_file1.txt\n\
            folder1/folder2/folder2_file2.txt\n\
            hello_world/hello_world.txt\n";

        assert_eq!(truth, list);
    }

    #[test]
    fn extract_test() {
        assert_env();
        let test_dir = &TEST_DIR.join("test_extract");
        let dest_dir = &test_dir.join("dest");
        _ = fs::remove_dir_all(test_dir);

        eprintln!("extracting...\t{} -> {}", filename(TRUTH_VPK), filename(dest_dir));
        extract_all(&VPK_CMD, TRUTH_VPK, dest_dir, Options::default()).unwrap();

        eprintln!("checking contents");

        assert_eq!("file1", fs::read_to_string(dest_dir.join("file1.txt")).unwrap());
        assert_eq!("file2", fs::read_to_string(dest_dir.join("file2.txt")).unwrap());
        assert_eq!(
            "folder1_file1",
            fs::read_to_string(dest_dir.join("folder1/folder1_file1.txt")).unwrap()
        );
        assert_eq!(
            "folder2_file2",
            fs::read_to_string(dest_dir.join("folder1/folder2/folder2_file2.txt")).unwrap()
        );
        assert_eq!(
            "Hello World!",
            fs::read_to_string(dest_dir.join("hello_world/hello_world.txt")).unwrap()
        );

        _ = fs::remove_dir_all(test_dir);
    }

    #[test]
    fn archive_test() {
        assert_env();
        let test_dir = &TEST_DIR.join("test_archive");
        let dest_dir = &test_dir.join("dest");
        let dest_vpk = &dest_dir.with_extension("vpk");
        let src_dir = &test_dir.join("src");
        let src_vpk = &src_dir.with_extension("vpk");

        _ = fs::remove_dir_all(test_dir);

        // set up src_dir to be archived to avoid
        // archive() overwriting truth.vpk

        eprintln!("copying... \t{} -> {}", filename(TRUTH_VPK), filename(src_vpk));
        let src_vpk_path: &Path = src_vpk.as_ref();
        assert!(!src_vpk_path.exists());
        _ = fs::create_dir_all(src_vpk_path.parent().unwrap());
        fs::copy(TRUTH_VPK, src_vpk).unwrap_or_else(|_| {
            panic!(
                "error copying `{TRUTH_VPK}` to `{src_vpk:?}` for this test. \
                    they might exist already"
            )
        });

        eprintln!("extracting...\t{} -> {}", filename(src_vpk), filename(src_dir));
        extract_all(&VPK_CMD, src_vpk, src_dir, Options::default()).unwrap();
        _ = fs::remove_file(src_vpk);

        // archive

        eprintln!("archiving...\t{} -> {}", filename(src_dir), filename(dest_vpk));
        archive(&VPK_CMD, src_dir, Some(dest_vpk)).unwrap();
        eprintln!("Setup done");

        // extract and test

        eprintln!("extracting...\t{} -> {}", filename(dest_vpk), filename(dest_dir));
        extract_all(&VPK_CMD, dest_vpk, dest_dir, Options::default()).unwrap();
        eprintln!("testing {}...", filename(dest_dir));

        // tests:

        assert_eq!("file1", fs::read_to_string(dest_dir.join("file1.txt")).unwrap());
        assert_eq!("file2", fs::read_to_string(dest_dir.join("file2.txt")).unwrap());
        assert_eq!(
            "folder1_file1",
            fs::read_to_string(dest_dir.join("folder1/folder1_file1.txt")).unwrap()
        );
        assert_eq!(
            "folder2_file2",
            fs::read_to_string(dest_dir.join("folder1/folder2/folder2_file2.txt")).unwrap()
        );
        assert_eq!(
            "Hello World!",
            fs::read_to_string(dest_dir.join("hello_world/hello_world.txt")).unwrap()
        );

        _ = fs::remove_dir_all(test_dir);
    }
}
