use std::{
    fs::FileType,
    path::{Path, PathBuf},
};
use proc_macro_error2::{abort, abort_call_site};
use walkdir::WalkDir;
use proc_macro2::Span as Span2;

pub struct DirEntry {
    pub vfs_path: PathBuf,
    pub real_path: PathBuf,
    pub file_type: FileType,
}

pub fn dir_entries(path: impl AsRef<Path>, span: Span2) -> Vec<DirEntry> {
    let Ok(manifest_path) = std::env::var("CARGO_MANIFEST_DIR") else {
        abort_call_site!("environment variable CARGO_MANIFEST_DIR not defined");
    };
    let base_path = PathBuf::from(manifest_path).join(path);
    let base_path = match base_path.canonicalize() {
        Ok(base_path) => base_path,
        Err(err) => {
            abort!(
                span, "{}", err;
                note = "happened while canonicalizing {:?}", base_path
            );
        }
    };

    let Ok(walker) = WalkDir::new(&base_path)
        .into_iter()
        .collect::<Result<Vec<_>, _>>() else {
            abort!(span, "could not walk directory");
        };
    let entries = walker
        .into_iter()
        .map(|entry| {
            let real_path = entry.path().canonicalize().unwrap();
            let vfs_path = PathBuf::from("/")
                .join(
                    real_path
                        .strip_prefix(&base_path)
                        .expect("constructed from base path"),
                )
                .unixify();

            DirEntry {
                real_path,
                vfs_path,
                file_type: entry.file_type(),
            }
        })
        .collect();

    entries
}

pub trait PathExt {
    fn unixify(&self) -> PathBuf;
}

impl PathExt for Path {
    fn unixify(&self) -> PathBuf {
        self.display().to_string().replace(r"\", "/").into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn dir_entries_works() {
        let dir_entries = dir_entries("example/vfs", Span2::call_site()).unwrap();

        // map the directory entries into a HashSet of tuples for easy comparison
        let actual: Vec<(String, String, bool)> = dir_entries
            .into_iter()
            .map(|entry| {
                (
                    entry.real_path.display().to_string(),
                    entry.vfs_path.display().to_string(),
                    entry.file_type.is_file(),
                )
            })
            .collect();

        let expected = Vec::from_iter(
            [
                ("./example/vfs/", "/", false),
                ("./example/vfs/config.toml", "/config.toml", true),
                ("./example/vfs/data.json", "/data.json", true),
                ("./example/vfs/README.md", "/README.md", true),
                ("./example/vfs/some dir/", "/some dir", false),
                (
                    "./example/vfs/some dir/ellie.txt",
                    "/some dir/ellie.txt",
                    true,
                ),
            ]
            .map(|(absolute_path, vfs_path, is_file)| {
                (
                    fs::canonicalize(absolute_path)
                        .unwrap()
                        .display()
                        .to_string(),
                    vfs_path.to_string(),
                    is_file,
                )
            }),
        );

        assert_eq!(actual, expected);
    }
}
