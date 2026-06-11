use std::fs;
use std::io::Write;
use std::path::Path;

use crate::error::{Result, SfError};
use crate::project::{PackageManifest, Project};

/// Execute the pack command.
pub fn execute(file: &Path, output: Option<&Path>, embed_runtime: bool) -> Result<()> {
    // Determine if the input is a file or directory
    if file.is_dir() {
        pack_directory(file, output, embed_runtime)
    } else {
        pack_file(file, output, embed_runtime)
    }
}

/// Pack a single project file (.sfl) into a .sfp package.
fn pack_file(file: &Path, output: Option<&Path>, embed_runtime: bool) -> Result<()> {
    let project = Project::load(file)?;

    let output_path = match output {
        Some(p) => p.to_path_buf(),
        None => file.with_extension("sfp"),
    };

    pack_project(&project, &output_path, embed_runtime)
}

/// Pack a project directory into a .sfp package.
fn pack_directory(dir: &Path, output: Option<&Path>, embed_runtime: bool) -> Result<()> {
    // Look for project.sfl in the directory
    let sfl_path = dir.join("project.sfl");
    if !sfl_path.exists() {
        return Err(SfError::ProjectNotFound(
            "project.sfl not found in directory".to_string(),
        ));
    }

    let mut project = Project::load_sfl(&sfl_path)?;

    // Collect all assets from the directory
    let assets_dir = dir.join("assets");
    if assets_dir.exists() {
        collect_assets(&assets_dir, &mut project, dir)?;
    }

    let output_path = match output {
        Some(p) => p.to_path_buf(),
        None => dir.with_extension("sfp"),
    };

    pack_project(&project, &output_path, embed_runtime)
}

/// Collect asset files from a directory.
fn collect_assets(assets_dir: &Path, project: &mut Project, base_dir: &Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(assets_dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let relative = entry
                .path()
                .strip_prefix(base_dir)
                .map_err(|_| SfError::Package("Failed to compute relative path".to_string()))?;
            project.add_asset(relative.to_string_lossy().to_string());
        }
    }
    Ok(())
}

/// Pack a project into a .sfp file.
fn pack_project(project: &Project, output_path: &Path, embed_runtime: bool) -> Result<()> {
    // Build manifest
    let mut manifest = if embed_runtime {
        PackageManifest::with_embedded_runtime(project.meta.clone())
    } else {
        PackageManifest::new(project.meta.clone())
    };

    manifest.add_file("project.sfl");
    manifest.add_file("manifest.json");
    for asset in &project.assets {
        manifest.add_file(asset);
    }

    // Create the ZIP archive
    let file = fs::File::create(output_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // Write manifest.json
    let manifest_json = manifest.to_json()?;
    zip.start_file("manifest.json", options)?;
    zip.write_all(manifest_json.as_bytes())?;

    // Write project.sfl
    zip.start_file("project.sfl", options)?;
    zip.write_all(project.source.as_bytes())?;

    // Write assets (placeholder content for now)
    if !project.assets.is_empty() {
        zip.add_directory("assets/", options)?;
        for asset in &project.assets {
            zip.start_file(asset, options)?;
            zip.write_all(b"")?;
        }
    }

    // If embed_runtime, add runtime metadata
    if embed_runtime {
        zip.start_file("runtime.json", options)?;
        let runtime_info = serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "embed_runtime": true
        });
        zip.write_all(runtime_info.to_string().as_bytes())?;
    }

    zip.finish()?;

    println!("Packed project '{}' to {}", project.name(), output_path.display());
    println!("  Embed runtime: {}", embed_runtime);
    println!("  Files: {} (manifest + source + assets)", project.assets.len() + 2);

    Ok(())
}

/// Unpack a .sfp package to a directory.
pub fn unpack(sfp_path: &Path, output_dir: &Path) -> Result<Project> {
    let project = Project::load_sfp(sfp_path)?;

    fs::create_dir_all(output_dir)?;

    // Write project.sfl
    let sfl_path = output_dir.join("project.sfl");
    fs::write(&sfl_path, &project.source)?;

    // Create assets directory
    let assets_dir = output_dir.join("assets");
    if !project.assets.is_empty() {
        fs::create_dir_all(&assets_dir)?;
    }

    // Extract assets from the zip
    let file = fs::File::open(sfp_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut zip_file = archive.by_index(i)?;
        let outpath = match zip_file.enclosed_name() {
            Some(path) => output_dir.join(path),
            None => continue,
        };

        if zip_file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            std::io::copy(&mut zip_file, &mut outfile)?;
        }
    }

    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_sfl(dir: &TempDir, name: &str) -> std::path::PathBuf {
        let path = dir.path().join(format!("{}.sfl", name));
        let content = format!("// @name {}\n\nfn main() {{ print(42); }}", name);
        fs::write(&path, content).unwrap();
        path
    }

    fn create_test_project_dir(dir: &TempDir, name: &str) -> std::path::PathBuf {
        let project_dir = dir.path().join(name);
        fs::create_dir_all(project_dir.join("assets")).unwrap();
        let sfl_content = format!("// @name {}\n\nfn main() {{}}", name);
        fs::write(project_dir.join("project.sfl"), sfl_content).unwrap();
        fs::write(project_dir.join("assets/sprite.png"), "PNG_DATA").unwrap();
        project_dir
    }

    #[test]
    fn test_pack_sfl_file() {
        let dir = TempDir::new().unwrap();
        let sfl_path = create_test_sfl(&dir, "test_pack");
        let output = dir.path().join("output.sfp");

        let result = execute(&sfl_path, Some(&output), false);
        assert!(result.is_ok());
        assert!(output.exists());
    }

    #[test]
    fn test_pack_sfl_file_embed_runtime() {
        let dir = TempDir::new().unwrap();
        let sfl_path = create_test_sfl(&dir, "test_embed");
        let output = dir.path().join("output.sfp");

        let result = execute(&sfl_path, Some(&output), true);
        assert!(result.is_ok());
        assert!(output.exists());
    }

    #[test]
    fn test_pack_directory() {
        let dir = TempDir::new().unwrap();
        let project_dir = create_test_project_dir(&dir, "MyProject");
        let output = dir.path().join("output.sfp");

        let result = execute(&project_dir, Some(&output), false);
        assert!(result.is_ok());
        assert!(output.exists());
    }

    #[test]
    fn test_pack_directory_no_project_sfl() {
        let dir = TempDir::new().unwrap();
        let empty_dir = dir.path().join("empty");
        fs::create_dir_all(&empty_dir).unwrap();

        let output = dir.path().join("output.sfp");
        let result = execute(&empty_dir, Some(&output), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_pack_default_output_path() {
        let dir = TempDir::new().unwrap();
        let sfl_path = create_test_sfl(&dir, "test_default");
        let expected_output = dir.path().join("test_default.sfp");

        let result = execute(&sfl_path, None, false);
        assert!(result.is_ok());
        assert!(expected_output.exists());
    }

    #[test]
    fn test_pack_nonexistent_file() {
        let result = execute(Path::new("/nonexistent.sfl"), None, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_unpack() {
        let dir = TempDir::new().unwrap();

        // First, pack a project
        let sfl_path = create_test_sfl(&dir, "test_unpack");
        let sfp_path = dir.path().join("packed.sfp");
        execute(&sfl_path, Some(&sfp_path), false).unwrap();

        // Then unpack it
        let unpack_dir = dir.path().join("unpacked");
        let result = unpack(&sfp_path, &unpack_dir);
        assert!(result.is_ok());

        let project = result.unwrap();
        assert_eq!(project.name(), "test_unpack");
        assert!(unpack_dir.join("project.sfl").exists());
        assert!(unpack_dir.join("manifest.json").exists());
    }

    #[test]
    fn test_pack_unpack_roundtrip() {
        let dir = TempDir::new().unwrap();

        // Create and pack a project
        let mut project = Project::new("RoundTrip");
        project.source = "fn main() { print(\"hello\"); }".to_string();
        project.add_asset("assets/icon.png");

        let sfp_path = dir.path().join("roundtrip.sfp");
        pack_project(&project, &sfp_path, false).unwrap();

        // Unpack and verify
        let unpack_dir = dir.path().join("unpacked");
        let loaded = unpack(&sfp_path, &unpack_dir).unwrap();

        assert_eq!(loaded.name(), "RoundTrip");
        assert_eq!(loaded.source, "fn main() { print(\"hello\"); }");
    }

    #[test]
    fn test_pack_with_embed_runtime_creates_runtime_json() {
        let dir = TempDir::new().unwrap();
        let sfl_path = create_test_sfl(&dir, "test_embed_rt");
        let output = dir.path().join("output.sfp");

        execute(&sfl_path, Some(&output), true).unwrap();

        // Verify runtime.json is in the archive
        let file = fs::File::open(&output).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        assert!(archive.by_name("runtime.json").is_ok());
    }

    #[test]
    fn test_pack_without_embed_runtime_no_runtime_json() {
        let dir = TempDir::new().unwrap();
        let sfl_path = create_test_sfl(&dir, "test_no_embed");
        let output = dir.path().join("output.sfp");

        execute(&sfl_path, Some(&output), false).unwrap();

        let file = fs::File::open(&output).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        assert!(archive.by_name("runtime.json").is_err());
    }

    #[test]
    fn test_collect_assets() {
        let dir = TempDir::new().unwrap();
        let assets_dir = dir.path().join("assets");
        fs::create_dir_all(&assets_dir).unwrap();
        fs::write(assets_dir.join("sprite.png"), "PNG").unwrap();
        fs::write(assets_dir.join("sound.mp3"), "MP3").unwrap();

        let mut project = Project::new("Test");
        collect_assets(&assets_dir, &mut project, dir.path()).unwrap();

        assert!(project.assets.iter().any(|a| a.contains("sprite.png")));
        assert!(project.assets.iter().any(|a| a.contains("sound.mp3")));
    }

    #[test]
    fn test_pack_project_with_assets() {
        let dir = TempDir::new().unwrap();

        let mut project = Project::new("AssetProject");
        project.source = "fn main() {}".to_string();
        project.add_asset("assets/sprite.png");
        project.add_asset("assets/sound.mp3");

        let output = dir.path().join("output.sfp");
        let result = pack_project(&project, &output, false);
        assert!(result.is_ok());

        // Verify assets are in the archive
        let file = fs::File::open(&output).unwrap();
        let archive = zip::ZipArchive::new(file).unwrap();
        let names: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();
        assert!(names.contains(&"assets/sprite.png".to_string()));
    }

    #[test]
    fn test_manifest_in_package() {
        let dir = TempDir::new().unwrap();
        let sfl_path = create_test_sfl(&dir, "test_manifest");
        let output = dir.path().join("output.sfp");

        execute(&sfl_path, Some(&output), false).unwrap();

        // Read and verify manifest
        let file = fs::File::open(&output).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut manifest_file = archive.by_name("manifest.json").unwrap();
        let mut manifest_str = String::new();
        std::io::Read::read_to_string(&mut manifest_file, &mut manifest_str).unwrap();

        let manifest: PackageManifest = serde_json::from_str(&manifest_str).unwrap();
        assert_eq!(manifest.meta.name, "test_manifest");
    }
}
