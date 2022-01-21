use std::fs::DirBuilder;
use std::path::PathBuf;
use tempdir::TempDir;

#[cfg(test)]
pub fn with_input_and_output_paths(f: fn(PathBuf, PathBuf) -> ()) -> std::io::Result<()> {
    let dir = TempDir::new("meh-utils-rust-in")?;
    let temp_dir_path = dir.path();
    let input_path = temp_dir_path.join("input");
    let output_path = temp_dir_path.join("output");
    DirBuilder::new().create(&input_path)?;
    DirBuilder::new().create(&output_path)?;

    f(input_path, output_path);

    dir.close()
}