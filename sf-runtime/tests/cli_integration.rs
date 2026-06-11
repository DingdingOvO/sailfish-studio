use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

const BIN: &str = "sf";

fn sf_cmd() -> Command {
    Command::cargo_bin(BIN).unwrap()
}

// ===== Help and Version Tests =====

#[test]
fn test_help_flag() {
    sf_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Sailfish Studio Runtime CLI"));
}

#[test]
fn test_version_flag() {
    sf_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_run_help() {
    sf_cmd()
        .arg("run")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--headed"));
}

#[test]
fn test_pack_help() {
    sf_cmd()
        .arg("pack")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--embed-runtime"));
}

#[test]
fn test_new_help() {
    sf_cmd()
        .arg("new")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--template"));
}

#[test]
fn test_check_help() {
    sf_cmd()
        .arg("check")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--strict"));
}

#[test]
fn test_no_command_shows_help() {
    sf_cmd()
        .assert()
        .failure();
}

#[test]
fn test_unknown_command() {
    sf_cmd()
        .arg("unknown")
        .assert()
        .failure();
}

// ===== Run Command Tests =====

#[test]
fn test_run_sfl_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.sfl");
    fs::write(&path, "// @name TestRun\n\nfn main() { print(42); }").unwrap();

    sf_cmd()
        .arg("run")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Running project: TestRun"));
}

#[test]
fn test_run_headed_flag() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.sfl");
    fs::write(&path, "// @name Headed\n\nfn main() {}").unwrap();

    sf_cmd()
        .arg("run")
        .arg(&path)
        .arg("--headed")
        .assert()
        .success()
        .stdout(predicate::str::contains("headed"));
}

#[test]
fn test_run_headless_default() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.sfl");
    fs::write(&path, "// @name Headless\n\nfn main() {}").unwrap();

    sf_cmd()
        .arg("run")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("headless"));
}

#[test]
fn test_run_custom_fps() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.sfl");
    fs::write(&path, "// @name FpsTest\n\nfn main() {}").unwrap();

    sf_cmd()
        .arg("run")
        .arg(&path)
        .arg("--fps")
        .arg("60")
        .assert()
        .success()
        .stdout(predicate::str::contains("FPS: 60"));
}

#[test]
fn test_run_custom_stage_size() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.sfl");
    fs::write(&path, "// @name StageTest\n\nfn main() {}").unwrap();

    sf_cmd()
        .arg("run")
        .arg(&path)
        .arg("--width")
        .arg("960")
        .arg("--height")
        .arg("720")
        .assert()
        .success()
        .stdout(predicate::str::contains("960x720"));
}

#[test]
fn test_run_nonexistent_file() {
    sf_cmd()
        .arg("run")
        .arg("/nonexistent/file.sfl")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_run_empty_project() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("empty.sfl");
    fs::write(&path, "// @name Empty\n").unwrap();

    sf_cmd()
        .arg("run")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("empty"));
}

#[test]
fn test_run_sfp_file() {
    let dir = TempDir::new().unwrap();
    let sfp_path = dir.path().join("packed.sfp");

    // Create a .sfp file first using the pack command
    let sfl_path = dir.path().join("test.sfl");
    fs::write(&sfl_path, "// @name SfpRun\n\nfn main() { print(1); }").unwrap();

    sf_cmd()
        .arg("pack")
        .arg(&sfl_path)
        .arg("-o")
        .arg(&sfp_path)
        .assert()
        .success();

    sf_cmd()
        .arg("run")
        .arg(&sfp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Running project"));
}

// ===== Pack Command Tests =====

#[test]
fn test_pack_sfl_file() {
    let dir = TempDir::new().unwrap();
    let sfl_path = dir.path().join("test.sfl");
    fs::write(&sfl_path, "// @name PackTest\n\nfn main() {}").unwrap();

    let output = dir.path().join("output.sfp");

    sf_cmd()
        .arg("pack")
        .arg(&sfl_path)
        .arg("-o")
        .arg(&output)
        .assert()
        .success();

    assert!(output.exists());
}

#[test]
fn test_pack_embed_runtime() {
    let dir = TempDir::new().unwrap();
    let sfl_path = dir.path().join("test.sfl");
    fs::write(&sfl_path, "// @name EmbedTest\n\nfn main() {}").unwrap();

    let output = dir.path().join("output.sfp");

    sf_cmd()
        .arg("pack")
        .arg(&sfl_path)
        .arg("-o")
        .arg(&output)
        .arg("--embed-runtime")
        .assert()
        .success()
        .stdout(predicate::str::contains("Embed runtime: true"));
}

#[test]
fn test_pack_default_output() {
    let dir = TempDir::new().unwrap();
    let sfl_path = dir.path().join("myproject.sfl");
    fs::write(&sfl_path, "// @name DefaultOut\n\nfn main() {}").unwrap();

    sf_cmd()
        .arg("pack")
        .arg(&sfl_path)
        .assert()
        .success();

    let expected_output = dir.path().join("myproject.sfp");
    assert!(expected_output.exists());
}

#[test]
fn test_pack_directory() {
    let dir = TempDir::new().unwrap();
    let project_dir = dir.path().join("myproject");
    fs::create_dir_all(project_dir.join("assets")).unwrap();
    fs::write(
        project_dir.join("project.sfl"),
        "// @name DirPack\n\nfn main() {}",
    )
    .unwrap();

    let output = dir.path().join("output.sfp");

    sf_cmd()
        .arg("pack")
        .arg(&project_dir)
        .arg("-o")
        .arg(&output)
        .assert()
        .success();
}

#[test]
fn test_pack_nonexistent_file() {
    sf_cmd()
        .arg("pack")
        .arg("/nonexistent.sfl")
        .assert()
        .failure();
}

// ===== New Command Tests =====

#[test]
fn test_new_blank_project() {
    let dir = TempDir::new().unwrap();
    let project_path = dir.path().join("my_blank_project");

    sf_cmd()
        .arg("new")
        .arg("my_blank_project")
        .arg("--dir")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created new project"));

    assert!(project_path.join("project.sfl").exists());
    assert!(project_path.join("sf.toml").exists());
    assert!(project_path.join(".gitignore").exists());
    assert!(project_path.join("assets").exists());
}

#[test]
fn test_new_game_project() {
    let dir = TempDir::new().unwrap();

    sf_cmd()
        .arg("new")
        .arg("my_game")
        .arg("--template")
        .arg("game")
        .arg("--dir")
        .arg(dir.path())
        .assert()
        .success();

    let sfl = fs::read_to_string(dir.path().join("my_game/project.sfl")).unwrap();
    assert!(sfl.contains("game_loop"));
}

#[test]
fn test_new_animation_project() {
    let dir = TempDir::new().unwrap();

    sf_cmd()
        .arg("new")
        .arg("my_anim")
        .arg("--template")
        .arg("animation")
        .arg("--dir")
        .arg(dir.path())
        .assert()
        .success();

    let sfl = fs::read_to_string(dir.path().join("my_anim/project.sfl")).unwrap();
    assert!(sfl.contains("frame"));
}

#[test]
fn test_new_project_already_exists() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join("existing")).unwrap();

    sf_cmd()
        .arg("new")
        .arg("existing")
        .arg("--dir")
        .arg(dir.path())
        .assert()
        .failure();
}

#[test]
fn test_new_invalid_template() {
    let dir = TempDir::new().unwrap();

    sf_cmd()
        .arg("new")
        .arg("bad_template")
        .arg("--template")
        .arg("nonexistent")
        .arg("--dir")
        .arg(dir.path())
        .assert()
        .failure();
}

// ===== Check Command Tests =====

#[test]
fn test_check_valid_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("valid.sfl");
    fs::write(&path, "fn main() {\n    print(42);\n}\n").unwrap();

    sf_cmd()
        .arg("check")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("No issues found"));
}

#[test]
fn test_check_file_with_errors() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("bad.sfl");
    fs::write(&path, "fn other() {\n    var x = \"unclosed\n}\n").unwrap();

    sf_cmd()
        .arg("check")
        .arg(&path)
        .assert()
        .success(); // Check doesn't fail, just reports
}

#[test]
fn test_check_strict_mode() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("strict.sfl");
    fs::write(&path, "fn main() {\n\tvar q = 42;   \n}\n").unwrap();

    sf_cmd()
        .arg("check")
        .arg(&path)
        .arg("--strict")
        .assert()
        .success();
}

#[test]
fn test_check_nonexistent_file() {
    sf_cmd()
        .arg("check")
        .arg("/nonexistent.sfl")
        .assert()
        .failure();
}

#[test]
fn test_check_empty_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("empty.sfl");
    fs::write(&path, "").unwrap();

    sf_cmd()
        .arg("check")
        .arg(&path)
        .assert()
        .success();
}

#[test]
fn test_check_sfp_file() {
    let dir = TempDir::new().unwrap();

    // Create a .sfp first
    let sfl_path = dir.path().join("test.sfl");
    fs::write(&sfl_path, "fn main() {\n    print(42);\n}\n").unwrap();
    let sfp_path = dir.path().join("test.sfp");

    sf_cmd()
        .arg("pack")
        .arg(&sfl_path)
        .arg("-o")
        .arg(&sfp_path)
        .assert()
        .success();

    sf_cmd()
        .arg("check")
        .arg(&sfp_path)
        .assert()
        .success();
}

// ===== Error Display Tests =====

#[test]
fn test_error_io_shown() {
    sf_cmd()
        .arg("run")
        .arg("/nonexistent.sfl")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_missing_file_argument() {
    sf_cmd()
        .arg("run")
        .assert()
        .failure();
}

#[test]
fn test_missing_name_argument() {
    sf_cmd()
        .arg("new")
        .assert()
        .failure();
}
