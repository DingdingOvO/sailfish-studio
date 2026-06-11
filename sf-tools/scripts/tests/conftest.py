"""Shared pytest fixtures for sf-tools tests."""

from __future__ import annotations

import json
import os
import pytest


@pytest.fixture
def sample_project() -> dict:
    """Return a valid sample project dictionary.

    Contains a stage target and a sprite target with valid blocks.
    """
    return {
        "targets": [
            {
                "isStage": True,
                "name": "Stage",
                "variables": {},
                "lists": {},
                "blocks": {
                    "block1": {
                        "opcode": "event_whenflagclicked",
                        "next": "block2",
                        "parent": None,
                        "inputs": {},
                        "fields": {},
                    },
                    "block2": {
                        "opcode": "looks_say",
                        "next": None,
                        "parent": "block1",
                        "inputs": {
                            "MESSAGE": [1, "Hello, Sailfish!"]
                        },
                        "fields": {},
                    },
                },
                "costumes": [
                    {
                        "name": "backdrop1",
                        "assetId": "abc123",
                        "md5ext": "abc123.svg",
                        "dataFormat": "svg",
                        "size": 4096,
                    }
                ],
                "sounds": [],
                "currentCostume": 0,
            },
            {
                "isStage": False,
                "name": "Sprite1",
                "variables": {"var1": ["my var", 0]},
                "lists": {},
                "blocks": {
                    "block3": {
                        "opcode": "event_whenflagclicked",
                        "next": "block4",
                        "parent": None,
                        "inputs": {},
                        "fields": {},
                    },
                    "block4": {
                        "opcode": "motion_forward",
                        "next": None,
                        "parent": "block3",
                        "inputs": {
                            "STEPS": [1, 10]
                        },
                        "fields": {},
                    },
                },
                "costumes": [
                    {
                        "name": "costume1",
                        "assetId": "def456",
                        "md5ext": "def456.svg",
                        "dataFormat": "svg",
                        "size": 8192,
                    }
                ],
                "sounds": [
                    {
                        "name": "pop",
                        "assetId": "ghi789",
                        "md5ext": "ghi789.wav",
                        "dataFormat": "wav",
                        "size": 12345,
                    }
                ],
                "currentCostume": 0,
            },
        ],
        "meta": {
            "semver": "3.0.0",
            "vm": "0.1.0",
            "agent": "Sailfish Studio",
        },
    }


@pytest.fixture
def sample_sb3_project() -> dict:
    """Return a valid .sb3 (Scratch 3.0) project dictionary.

    Includes Scratch-specific fields like monitors and extensions.
    """
    return {
        "targets": [
            {
                "isStage": True,
                "name": "Stage",
                "variables": {},
                "lists": {},
                "blocks": {
                    "block1": {
                        "opcode": "event_whenflagclicked",
                        "next": None,
                        "parent": None,
                        "inputs": {},
                        "fields": {},
                    }
                },
                "costumes": [
                    {
                        "name": "backdrop1",
                        "assetId": "aaa111",
                        "md5ext": "aaa111.svg",
                        "dataFormat": "svg",
                        "size": 2048,
                    }
                ],
                "sounds": [],
                "currentCostume": 0,
            }
        ],
        "monitors": [],
        "extensions": [],
        "meta": {
            "semver": "3.0.0",
            "vm": "2.0.0",
            "agent": "Scratch 3.0",
        },
    }


@pytest.fixture
def sample_sfl_source() -> str:
    """Return a valid .sfl source code string.

    Contains variable declarations, function definitions, and control structures.
    """
    return (
        "var counter: int = 0\n"
        "\n"
        "fn greet(name: str) -> str:\n"
        "    return \"Hello, \" + name\n"
        "end\n"
        "\n"
        "fn main() -> void:\n"
        "    var x: int = 10\n"
        "    if x > 5:\n"
        "        say(greet(\"world\"))\n"
        "    end\n"
        "end\n"
    )


@pytest.fixture
def test_data_dir() -> str:
    """Return the path to the test_data directory."""
    return os.path.join(os.path.dirname(__file__), "..", "test_data")


@pytest.fixture
def sample_project_json_path(test_data_dir: str) -> str:
    """Return the path to the sample_project.json file."""
    return os.path.join(test_data_dir, "sample_project.json")


@pytest.fixture
def sample_project_from_file(sample_project_json_path: str) -> dict:
    """Load and return the sample project from the JSON file."""
    with open(sample_project_json_path, "r") as f:
        return json.load(f)
