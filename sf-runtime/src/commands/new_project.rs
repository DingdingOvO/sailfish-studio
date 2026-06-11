use std::fs;
use std::path::Path;

use crate::error::{Result, SfError};
use crate::project::Project;

/// Available project templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Template {
    /// Empty project with minimal boilerplate.
    Blank,
    /// Game template with game loop and input handling.
    Game,
    /// Animation template with timeline and effects.
    Animation,
}

impl Template {
    /// Parse a template from a string.
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "blank" => Some(Template::Blank),
            "game" => Some(Template::Game),
            "animation" => Some(Template::Animation),
            _ => None,
        }
    }

    /// Get the template name as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Template::Blank => "blank",
            Template::Game => "game",
            Template::Animation => "animation",
        }
    }

    /// Get the template source code.
    pub fn source(&self) -> &'static str {
        match self {
            Template::Blank => BLANK_TEMPLATE,
            Template::Game => GAME_TEMPLATE,
            Template::Animation => ANIMATION_TEMPLATE,
        }
    }
}

const BLANK_TEMPLATE: &str = r#"// Blank project template
// Add your sprites and logic here

fn main() {
    // Your code here
}
"#;

const GAME_TEMPLATE: &str = r#"// Game project template
// Includes basic game loop, input handling, and scoring

var score = 0
var lives = 3
var game_over = false

fn main() {
    init_game()
    game_loop()
}

fn init_game() {
    score = 0
    lives = 3
    game_over = false
}

fn game_loop() {
    while !game_over {
        handle_input()
        update()
        render()
    }
}

fn handle_input() {
    // Handle key press events
    if key_pressed("left") {
        move_left()
    }
    if key_pressed("right") {
        move_right()
    }
}

fn update() {
    // Update game state
}

fn render() {
    // Draw sprites and UI
}

fn move_left() {
    // Move player left
}

fn move_right() {
    // Move player right
}
"#;

const ANIMATION_TEMPLATE: &str = r#"// Animation project template
// Includes timeline and effects

var frame = 0
var total_frames = 60
var fps = 30
var playing = true

fn main() {
    while playing {
        update_frame()
        render_frame()
        frame = frame + 1
        if frame >= total_frames {
            frame = 0
        }
    }
}

fn update_frame() {
    // Update animation state based on current frame
}

fn render_frame() {
    // Draw the current frame
}
"#;

/// Execute the new project command.
pub fn execute(name: &str, template: Option<&str>, base_dir: Option<&Path>) -> Result<()> {
    // Validate project name
    validate_name(name)?;

    let tmpl = template
        .map(|t| Template::from_str_opt(t).ok_or_else(|| SfError::InvalidName(format!(
            "Unknown template '{}'. Available templates: blank, game, animation",
            t
        ))))
        .transpose()?
        .unwrap_or(Template::Blank);

    create_project(name, tmpl, base_dir)
}

/// Validate a project name.
fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(SfError::InvalidName("Project name cannot be empty".to_string()));
    }
    if name.contains('/') || name.contains('\\') {
        return Err(SfError::InvalidName(
            "Project name cannot contain path separators".to_string(),
        ));
    }
    if name.starts_with('.') {
        return Err(SfError::InvalidName(
            "Project name cannot start with a dot".to_string(),
        ));
    }
    if name.contains(char::is_control) {
        return Err(SfError::InvalidName(
            "Project name cannot contain control characters".to_string(),
        ));
    }
    Ok(())
}

/// Create a new project directory with the given name and template.
/// If `base_dir` is Some, create the project under that directory; otherwise use CWD.
fn create_project(name: &str, template: Template, base_dir: Option<&Path>) -> Result<()> {
    let project_dir = match base_dir {
        Some(dir) => dir.join(name),
        None => Path::new(name).to_path_buf(),
    };

    // Check if directory already exists
    if project_dir.exists() {
        return Err(SfError::AlreadyExists(format!(
            "Directory '{}' already exists",
            project_dir.display()
        )));
    }

    // Create directory structure
    fs::create_dir_all(project_dir.join("assets"))?;
    fs::create_dir_all(project_dir.join("assets/costumes"))?;
    fs::create_dir_all(project_dir.join("assets/sounds"))?;

    // Generate project.sfl
    let mut project = Project::new(name);
    project.source = template.source().to_string();
    project.save_sfl(&project_dir.join("project.sfl"))?;

    // Generate sf.toml config
    let config_content = generate_config(name);
    fs::write(project_dir.join("sf.toml"), config_content)?;

    // Generate .gitignore
    fs::write(project_dir.join(".gitignore"), generate_gitignore())?;

    // Generate README
    fs::write(
        project_dir.join("README.md"),
        generate_readme(name, template),
    )?;

    println!("Created new project: {}", name);
    println!("  Template: {}", template.as_str());
    println!("  Directory: {}", project_dir.display());
    println!();
    println!("To run your project:");
    println!("  sf run {}/project.sfl", name);

    Ok(())
}

/// Generate sf.toml configuration content.
fn generate_config(name: &str) -> String {
    format!(
        r#"[project]
name = "{}"
version = "0.1.0"

[runtime]
fps = 30
stage_width = 480
stage_height = 360
turbo_mode = false
"#,
        name
    )
}

/// Generate .gitignore content.
fn generate_gitignore() -> String {
    r#"# Build output
/build/
/dist/

# OS files
.DS_Store
Thumbs.db

# Editor files
*.swp
*.swo
*~
.vscode/
.idea/

# Package files
*.sfp
"#
    .to_string()
}

/// Generate README content.
fn generate_readme(name: &str, template: Template) -> String {
    format!(
        r#"# {}

A Sailfish Studio project.

## Template: {}

## Getting Started

Run the project:
```bash
sf run project.sfl
```

Package the project:
```bash
sf pack project.sfl
```

Check for errors:
```bash
sf check project.sfl
```

## Project Structure

- `project.sfl` - Main project source
- `sf.toml` - Project configuration
- `assets/` - Project assets
  - `costumes/` - Sprite costumes
  - `sounds/` - Sound files
"#,
        name,
        template.as_str()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_template_from_str_blank() {
        assert_eq!(Template::from_str_opt("blank"), Some(Template::Blank));
    }

    #[test]
    fn test_template_from_str_game() {
        assert_eq!(Template::from_str_opt("game"), Some(Template::Game));
    }

    #[test]
    fn test_template_from_str_animation() {
        assert_eq!(Template::from_str_opt("animation"), Some(Template::Animation));
    }

    #[test]
    fn test_template_from_str_case_insensitive() {
        assert_eq!(Template::from_str_opt("Game"), Some(Template::Game));
        assert_eq!(Template::from_str_opt("ANIMATION"), Some(Template::Animation));
    }

    #[test]
    fn test_template_from_str_unknown() {
        assert_eq!(Template::from_str_opt("unknown"), None);
    }

    #[test]
    fn test_template_as_str() {
        assert_eq!(Template::Blank.as_str(), "blank");
        assert_eq!(Template::Game.as_str(), "game");
        assert_eq!(Template::Animation.as_str(), "animation");
    }

    #[test]
    fn test_template_source_not_empty() {
        assert!(!Template::Blank.source().is_empty());
        assert!(!Template::Game.source().is_empty());
        assert!(!Template::Animation.source().is_empty());
    }

    #[test]
    fn test_validate_name_ok() {
        assert!(validate_name("my-project").is_ok());
        assert!(validate_name("MyProject").is_ok());
        assert!(validate_name("project123").is_ok());
    }

    #[test]
    fn test_validate_name_empty() {
        assert!(validate_name("").is_err());
    }

    #[test]
    fn test_validate_name_with_slash() {
        assert!(validate_name("bad/name").is_err());
    }

    #[test]
    fn test_validate_name_with_backslash() {
        assert!(validate_name("bad\\name").is_err());
    }

    #[test]
    fn test_validate_name_starts_with_dot() {
        assert!(validate_name(".hidden").is_err());
    }

    #[test]
    fn test_validate_name_with_control_chars() {
        assert!(validate_name("bad\x00name").is_err());
    }

    #[test]
    fn test_create_project_blank() {
        let dir = TempDir::new().unwrap();
        let project_path = dir.path().join("my_project");

        let result = create_project("my_project", Template::Blank, Some(dir.path()));
        assert!(result.is_ok());
        assert!(project_path.exists());
        assert!(project_path.join("project.sfl").exists());
        assert!(project_path.join("sf.toml").exists());
        assert!(project_path.join(".gitignore").exists());
        assert!(project_path.join("README.md").exists());
        assert!(project_path.join("assets").exists());
        assert!(project_path.join("assets/costumes").exists());
        assert!(project_path.join("assets/sounds").exists());
    }

    #[test]
    fn test_create_project_game() {
        let dir = TempDir::new().unwrap();
        let project_path = dir.path().join("game_project");

        let result = create_project("game_project", Template::Game, Some(dir.path()));
        assert!(result.is_ok());

        let sfl_content = fs::read_to_string(project_path.join("project.sfl")).unwrap();
        assert!(sfl_content.contains("game_loop"));
    }

    #[test]
    fn test_create_project_animation() {
        let dir = TempDir::new().unwrap();
        let project_path = dir.path().join("anim_project");

        let result = create_project("anim_project", Template::Animation, Some(dir.path()));
        assert!(result.is_ok());

        let sfl_content = fs::read_to_string(project_path.join("project.sfl")).unwrap();
        assert!(sfl_content.contains("frame"));
    }

    #[test]
    fn test_create_project_default_template() {
        let dir = TempDir::new().unwrap();
        let project_path = dir.path().join("default_project");

        let result = create_project("default_project", Template::Blank, Some(dir.path()));
        assert!(result.is_ok());
        assert!(project_path.exists());
    }

    #[test]
    fn test_create_project_already_exists() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("existing_project")).unwrap();

        let result = create_project("existing_project", Template::Blank, Some(dir.path()));
        assert!(result.is_err());
    }

    #[test]
    fn test_create_project_invalid_template() {
        let result = execute("test", Some("nonexistent"), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_game_template_has_score() {
        assert!(Template::Game.source().contains("score"));
    }

    #[test]
    fn test_game_template_has_lives() {
        assert!(Template::Game.source().contains("lives"));
    }

    #[test]
    fn test_animation_template_has_frame() {
        assert!(Template::Animation.source().contains("frame"));
    }

    #[test]
    fn test_animation_template_has_fps() {
        assert!(Template::Animation.source().contains("fps"));
    }

    #[test]
    fn test_blank_template_has_main() {
        assert!(Template::Blank.source().contains("fn main()"));
    }

    #[test]
    fn test_generate_config() {
        let config = generate_config("TestProject");
        assert!(config.contains("TestProject"));
        assert!(config.contains("fps = 30"));
    }

    #[test]
    fn test_generate_gitignore() {
        let gitignore = generate_gitignore();
        assert!(gitignore.contains(".sfp"));
        assert!(gitignore.contains("build"));
    }

    #[test]
    fn test_generate_readme() {
        let readme = generate_readme("MyProject", Template::Game);
        assert!(readme.contains("MyProject"));
        assert!(readme.contains("game"));
    }

    #[test]
    fn test_execute_validates_name() {
        // Empty name should fail
        let result = execute("", None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_with_blank_template() {
        let dir = TempDir::new().unwrap();
        let result = execute("test_blank_explicit", Some("blank"), Some(dir.path()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_project_with_base_dir() {
        let dir = TempDir::new().unwrap();
        let project_path = dir.path().join("basedir_project");

        let result = create_project("basedir_project", Template::Blank, Some(dir.path()));
        assert!(result.is_ok());
        assert!(project_path.join("project.sfl").exists());

        // Verify sf.toml content
        let config = fs::read_to_string(project_path.join("sf.toml")).unwrap();
        assert!(config.contains("basedir_project"));
    }

    #[test]
    fn test_create_project_without_base_dir() {
        // Test with base_dir = None (uses CWD)
        // This is a basic test that the code path works
        let dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let result = create_project("cwd_project", Template::Blank, None);
        assert!(result.is_ok());
        assert!(dir.path().join("cwd_project/project.sfl").exists());

        std::env::set_current_dir(original_dir).unwrap();
    }
}
