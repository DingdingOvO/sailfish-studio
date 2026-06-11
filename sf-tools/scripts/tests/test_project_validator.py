"""Tests for project_validator module."""

from __future__ import annotations

import sys
import os

# Add scripts directory to path so we can import modules
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from project_validator import (
    VALID_OPCODES,
    check_required_fields,
    validate_project_json,
    validate_sb3_structure,
    validate_sf_structure,
)


class TestValidOpcodes:
    """Tests for the VALID_OPCODES set."""

    def test_opcodes_count(self):
        """VALID_OPCODES should contain at least 50 opcodes."""
        assert len(VALID_OPCODES) >= 50

    def test_motion_opcodes_present(self):
        """Motion category opcodes should be present."""
        assert "motion_forward" in VALID_OPCODES
        assert "motion_turnright" in VALID_OPCODES
        assert "motion_goto" in VALID_OPCODES

    def test_looks_opcodes_present(self):
        """Looks category opcodes should be present."""
        assert "looks_say" in VALID_OPCODES
        assert "looks_show" in VALID_OPCODES
        assert "looks_hide" in VALID_OPCODES

    def test_control_opcodes_present(self):
        """Control category opcodes should be present."""
        assert "control_if" in VALID_OPCODES
        assert "control_if_else" in VALID_OPCODES
        assert "control_repeat" in VALID_OPCODES
        assert "control_forever" in VALID_OPCODES

    def test_data_opcodes_present(self):
        """Data/Variables category opcodes should be present."""
        assert "data_setvariableto" in VALID_OPCODES
        assert "data_changevariableby" in VALID_OPCODES


class TestCheckRequiredFields:
    """Tests for the check_required_fields helper."""

    def test_all_fields_present(self):
        """No errors when all required fields are present."""
        data = {"name": "test", "value": 42}
        errors = check_required_fields(data, ["name", "value"], "root")
        assert errors == []

    def test_missing_single_field(self):
        """One error for a single missing field."""
        data = {"name": "test"}
        errors = check_required_fields(data, ["name", "value"], "root")
        assert len(errors) == 1
        assert "value" in errors[0]

    def test_missing_multiple_fields(self):
        """Multiple errors for multiple missing fields."""
        data = {}
        errors = check_required_fields(data, ["name", "value", "type"], "root")
        assert len(errors) == 3

    def test_path_in_error_message(self):
        """Error message should include the dot-notation path."""
        data = {}
        errors = check_required_fields(data, ["name"], "project.targets[0]")
        assert "project.targets[0].name" in errors[0]

    def test_empty_required_list(self):
        """No errors when required list is empty."""
        data = {}
        errors = check_required_fields(data, [], "root")
        assert errors == []


class TestValidateProjectJson:
    """Tests for validate_project_json."""

    def test_valid_project_passes(self, sample_project):
        """A valid project should produce no errors."""
        errors = validate_project_json(sample_project)
        assert errors == []

    def test_missing_targets_fails(self):
        """Missing 'targets' field should produce an error."""
        data = {"meta": {}}
        errors = validate_project_json(data)
        assert any("targets" in e for e in errors)

    def test_empty_targets_fails(self):
        """Empty targets list should produce an error (no stage)."""
        data = {"targets": []}
        errors = validate_project_json(data)
        assert len(errors) > 0
        assert any("at least one target" in e.lower() for e in errors)

    def test_missing_stage_fails(self):
        """Project without a stage target should produce an error."""
        data = {
            "targets": [
                {
                    "isStage": False,
                    "name": "Sprite1",
                    "blocks": {},
                }
            ]
        }
        errors = validate_project_json(data)
        assert any("stage" in e.lower() for e in errors)

    def test_target_missing_name_fails(self):
        """Target without a 'name' field should produce an error."""
        data = {
            "targets": [
                {
                    "isStage": True,
                    "blocks": {},
                }
            ]
        }
        errors = validate_project_json(data)
        assert any("name" in e for e in errors)

    def test_invalid_opcode_fails(self):
        """Invalid block opcode should produce an error."""
        data = {
            "targets": [
                {
                    "isStage": True,
                    "name": "Stage",
                    "blocks": {
                        "bad_block": {
                            "opcode": "nonexistent_opcode",
                        }
                    },
                }
            ]
        }
        errors = validate_project_json(data)
        assert any("nonexistent_opcode" in e for e in errors)

    def test_valid_opcodes_pass(self, sample_project):
        """Valid opcodes should not produce errors."""
        errors = validate_project_json(sample_project)
        assert not any("opcode" in e.lower() for e in errors)

    def test_targets_not_list_fails(self):
        """Non-list targets should produce an error."""
        data = {"targets": "not a list"}
        errors = validate_project_json(data)
        assert any("must be a list" in e for e in errors)

    def test_target_not_dict_fails(self):
        """Non-dict target should produce an error."""
        data = {"targets": ["not a dict"]}
        errors = validate_project_json(data)
        assert any("must be a dict" in e for e in errors)

    def test_empty_project_dict(self):
        """Completely empty project dict should report missing targets."""
        errors = validate_project_json({})
        assert len(errors) > 0

    def test_multiple_errors_reported(self):
        """Multiple validation errors should all be reported."""
        data = {
            "targets": [
                {
                    "isStage": False,
                    "blocks": {
                        "b1": {"opcode": "invalid_opcode_1"},
                        "b2": {"opcode": "invalid_opcode_2"},
                    },
                }
            ]
        }
        errors = validate_project_json(data)
        # Should have: missing name, no stage, 2 invalid opcodes = 4 errors
        assert len(errors) >= 4

    def test_blocks_not_dict_ignored(self):
        """Targets with non-dict blocks should not crash."""
        data = {
            "targets": [
                {
                    "isStage": True,
                    "name": "Stage",
                    "blocks": "not a dict",
                }
            ]
        }
        errors = validate_project_json(data)
        # Should not crash, and stage + name are valid
        assert not any("opcode" in e.lower() for e in errors)


class TestValidateSb3Structure:
    """Tests for validate_sb3_structure."""

    def test_valid_sb3_passes(self, sample_sb3_project):
        """A valid .sb3 project should produce no errors."""
        errors = validate_sb3_structure(sample_sb3_project)
        assert errors == []

    def test_sb3_missing_meta_semver(self):
        """SB3 project with meta but no semver should report error."""
        data = {
            "targets": [
                {
                    "isStage": True,
                    "name": "Stage",
                    "blocks": {},
                }
            ],
            "meta": {"vm": "2.0.0"},
        }
        errors = validate_sb3_structure(data)
        assert any("semver" in e for e in errors)

    def test_sb3_no_meta_passes_base_validation(self, sample_sb3_project):
        """SB3 without meta should still pass base validation."""
        data = {
            "targets": [
                {
                    "isStage": True,
                    "name": "Stage",
                    "blocks": {},
                }
            ]
        }
        errors = validate_sb3_structure(data)
        # No meta field is OK, base validation should pass
        base_errors = [e for e in errors if "semver" not in e]
        assert base_errors == []


class TestValidateSfStructure:
    """Tests for validate_sf_structure."""

    def test_valid_sf_passes(self):
        """A valid .sf project should produce no errors."""
        data = {
            "targets": [
                {
                    "isStage": True,
                    "name": "Stage",
                    "blocks": {},
                    "costumes": [
                        {
                            "name": "bg",
                            "assetId": "abc",
                            "md5ext": "abc.svg",
                            "dataFormat": "svg",
                        }
                    ],
                    "sounds": [],
                }
            ],
            "sfVersion": "1.0.0",
        }
        errors = validate_sf_structure(data)
        assert errors == []

    def test_sf_missing_version(self):
        """SF project without sfVersion should report error."""
        data = {
            "targets": [
                {
                    "isStage": True,
                    "name": "Stage",
                    "blocks": {},
                }
            ]
        }
        errors = validate_sf_structure(data)
        assert any("sfVersion" in e for e in errors)

    def test_sf_costume_missing_asset_id(self):
        """SF project with costume missing assetId should report error."""
        data = {
            "targets": [
                {
                    "isStage": True,
                    "name": "Stage",
                    "blocks": {},
                    "costumes": [
                        {"name": "bg", "md5ext": "bg.svg", "dataFormat": "svg"}
                    ],
                    "sounds": [],
                }
            ],
            "sfVersion": "1.0.0",
        }
        errors = validate_sf_structure(data)
        assert any("assetId" in e for e in errors)

    def test_sf_sound_missing_asset_id(self):
        """SF project with sound missing assetId should report error."""
        data = {
            "targets": [
                {
                    "isStage": True,
                    "name": "Stage",
                    "blocks": {},
                    "costumes": [],
                    "sounds": [
                        {"name": "pop", "dataFormat": "wav"}
                    ],
                }
            ],
            "sfVersion": "1.0.0",
        }
        errors = validate_sf_structure(data)
        assert any("assetId" in e for e in errors)

    def test_sf_sample_project_file(self, sample_project_from_file):
        """Validate the sample_project.json test data file."""
        # The sample project doesn't have sfVersion, so it will have that error
        errors = validate_sf_structure(sample_project_from_file)
        # Should only have the sfVersion error
        assert all("sfVersion" in e for e in errors)
