use std::path::Path;

use crate::config::RuntimeConfig;
use crate::error::{Result, SfError};
use crate::project::Project;

/// Execute the run command.
pub fn execute(file: &Path, headed: bool, fps: Option<u32>, width: Option<u32>, height: Option<u32>) -> Result<()> {
    // Load the project
    let project = Project::load(file)?;

    // Build runtime config
    let mut config = if headed {
        RuntimeConfig::headed()
    } else {
        RuntimeConfig::headless()
    };

    if let Some(f) = fps {
        config = config.with_fps(f);
    }
    if let (Some(w), Some(h)) = (width, height) {
        config = config.with_stage_size(w, h);
    }

    // Merge with project-level config if present
    if let Some(ref project_config) = project.config {
        config = config.merge(project_config);
    }

    // Validate config
    config.validate()?;

    // Run the project
    run_project(&project, &config)?;

    Ok(())
}

/// Run a project with the given configuration.
fn run_project(project: &Project, config: &RuntimeConfig) -> Result<()> {
    if project.is_empty() {
        return Err(SfError::Runtime("Project source is empty, nothing to run".to_string()));
    }

    // In a real implementation, this would compile and execute the project source.
    // For now, we validate and simulate execution.
    println!("Running project: {}", project.name());
    println!("  Mode: {}", if config.headless { "headless" } else { "headed" });
    println!("  FPS: {}", config.fps);
    println!("  Stage: {}x{}", config.stage_width, config.stage_height);
    println!("  Turbo: {}", config.turbo_mode);
    println!("  Frame duration: {}ms", config.frame_duration_ms());
    println!("  Source length: {} bytes", project.source.len());
    println!("  Assets: {} files", project.assets.len());

    if !config.headless {
        // In a real implementation, this would open a window
        println!("  [GUI mode would open display window here]");
    } else {
        println!("  [Headless mode - no display]");
    }

    // Simulate execution completion
    println!("Project execution completed.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_sfl(dir: &TempDir, name: &str, source: &str) -> std::path::PathBuf {
        let path = dir.path().join(format!("{}.sfl", name));
        let mut content = format!("// @name {}\n\n{}", name, source);
        fs::write(&path, &content).unwrap();
        path
    }

    #[test]
    fn test_run_headless() {
        let dir = TempDir::new().unwrap();
        let path = create_test_sfl(&dir, "test_run", "fn main() { print(42); }");
        let result = execute(&path, false, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_with_custom_fps() {
        let dir = TempDir::new().unwrap();
        let path = create_test_sfl(&dir, "test_fps", "fn main() {}");
        let result = execute(&path, false, Some(60), None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_with_custom_stage_size() {
        let dir = TempDir::new().unwrap();
        let path = create_test_sfl(&dir, "test_stage", "fn main() {}");
        let result = execute(&path, false, None, Some(960), Some(720));
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_nonexistent_file() {
        let result = execute(Path::new("/nonexistent.sfl"), false, None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_empty_project() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("empty.sfl");
        // Write a file with only comments - no actual code
        fs::write(&path, "// @name empty\n").unwrap();
        let result = execute(&path, false, None, None, None);
        // Empty source (after stripping comments in load) should cause an error
        assert!(result.is_err());
    }

    #[test]
    fn test_run_headed_flag() {
        let dir = TempDir::new().unwrap();
        let path = create_test_sfl(&dir, "test_headed", "fn main() {}");
        let result = execute(&path, true, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_project_config_merge() {
        let dir = TempDir::new().unwrap();
        let path = create_test_sfl(&dir, "test_merge", "fn main() {}");

        // Test that project config is merged (we can verify by running)
        let result = execute(&path, false, Some(60), Some(960), Some(720));
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_sfp_project() {
        let dir = TempDir::new().unwrap();
        let sfp_path = dir.path().join("test.sfp");

        let mut project = Project::new("TestSfp");
        project.source = "fn main() { print(1); }".to_string();
        project.save_sfp(&sfp_path).unwrap();

        let result = execute(&sfp_path, false, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_runtime_config_build() {
        let config = RuntimeConfig::headless().with_fps(60).with_stage_size(960, 720);
        assert_eq!(config.fps, 60);
        assert_eq!(config.stage_width, 960);
        assert_eq!(config.stage_height, 720);
        assert!(config.headless);
    }

    #[test]
    fn test_runtime_config_headed() {
        let config = RuntimeConfig::headed();
        assert!(!config.headless);
    }
}
