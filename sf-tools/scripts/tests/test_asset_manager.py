"""Tests for asset_manager module."""

from __future__ import annotations

import sys
import os

# Add scripts directory to path so we can import modules
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from asset_manager import (
    AssetInfo,
    check_duplicate_assets,
    compute_asset_stats,
    format_size,
    scan_assets,
)


class TestAssetInfo:
    """Tests for the AssetInfo dataclass."""

    def test_create_asset_info(self):
        """AssetInfo should be creatable with all fields."""
        asset = AssetInfo(
            name="cat",
            asset_id="abc123",
            file_extension="svg",
            size_bytes=4096,
            md5_hash="abc123",
        )
        assert asset.name == "cat"
        assert asset.asset_id == "abc123"
        assert asset.file_extension == "svg"
        assert asset.size_bytes == 4096
        assert asset.md5_hash == "abc123"

    def test_asset_info_equality(self):
        """Two AssetInfo with same fields should be equal."""
        a1 = AssetInfo("cat", "abc", "svg", 100, "abc")
        a2 = AssetInfo("cat", "abc", "svg", 100, "abc")
        assert a1 == a2

    def test_asset_info_inequality(self):
        """Two AssetInfo with different fields should not be equal."""
        a1 = AssetInfo("cat", "abc", "svg", 100, "abc")
        a2 = AssetInfo("dog", "abc", "svg", 100, "abc")
        assert a1 != a2


class TestScanAssets:
    """Tests for scan_assets."""

    def test_scan_from_sample_project(self, sample_project):
        """Should extract all costumes and sounds from sample project."""
        assets = scan_assets(sample_project)
        # sample_project has: 1 costume on stage, 1 costume on sprite, 1 sound on sprite
        assert len(assets) == 3

    def test_scan_empty_project(self):
        """Empty project should yield no assets."""
        assets = scan_assets({"targets": []})
        assert assets == []

    def test_scan_no_targets_key(self):
        """Project without targets key should yield no assets."""
        assets = scan_assets({})
        assert assets == []

    def test_scan_targets_not_list(self):
        """Project with non-list targets should yield no assets."""
        assets = scan_assets({"targets": "not a list"})
        assert assets == []

    def test_scan_extracts_correct_fields(self):
        """Scanned assets should have correct field values."""
        data = {
            "targets": [
                {
                    "costumes": [
                        {
                            "name": "bg",
                            "assetId": "hash123",
                            "md5ext": "hash123.svg",
                            "dataFormat": "svg",
                            "size": 2048,
                        }
                    ],
                    "sounds": [],
                }
            ]
        }
        assets = scan_assets(data)
        assert len(assets) == 1
        assert assets[0].name == "bg"
        assert assets[0].asset_id == "hash123"
        assert assets[0].file_extension == "svg"
        assert assets[0].size_bytes == 2048

    def test_scan_from_file(self, sample_project_from_file):
        """Should scan assets from the sample_project.json test data."""
        assets = scan_assets(sample_project_from_file)
        # sample_project.json has: 1 stage costume, 2 sprite costumes, 1 sprite sound
        assert len(assets) == 4

    def test_scan_missing_asset_id_skipped(self):
        """Assets without assetId or md5ext should be skipped."""
        data = {
            "targets": [
                {
                    "costumes": [
                        {"name": "bg", "dataFormat": "svg", "size": 100}
                    ],
                    "sounds": [],
                }
            ]
        }
        assets = scan_assets(data)
        assert len(assets) == 0

    def test_scan_md5ext_as_asset_id_fallback(self):
        """When assetId is missing, md5ext should be used as fallback."""
        data = {
            "targets": [
                {
                    "costumes": [
                        {
                            "name": "bg",
                            "md5ext": "fallback_hash.svg",
                            "dataFormat": "svg",
                            "size": 100,
                        }
                    ],
                    "sounds": [],
                }
            ]
        }
        assets = scan_assets(data)
        assert len(assets) == 1
        assert assets[0].asset_id == "fallback_hash.svg"


class TestCheckDuplicateAssets:
    """Tests for check_duplicate_assets."""

    def test_no_duplicates(self):
        """Assets with different hashes should produce no duplicates."""
        assets = [
            AssetInfo("a", "id1", "svg", 100, "hash1"),
            AssetInfo("b", "id2", "png", 200, "hash2"),
        ]
        duplicates = check_duplicate_assets(assets)
        assert duplicates == []

    def test_with_duplicates(self):
        """Assets with same hash should be reported as duplicates."""
        assets = [
            AssetInfo("cat1", "id1", "svg", 100, "same_hash"),
            AssetInfo("cat2", "id2", "svg", 100, "same_hash"),
        ]
        duplicates = check_duplicate_assets(assets)
        assert len(duplicates) == 1
        assert duplicates[0][0].name == "cat1"
        assert duplicates[0][1].name == "cat2"

    def test_empty_hash_ignored(self):
        """Assets with empty md5_hash should be ignored."""
        assets = [
            AssetInfo("a", "id1", "svg", 100, ""),
            AssetInfo("b", "id2", "svg", 100, ""),
        ]
        duplicates = check_duplicate_assets(assets)
        assert duplicates == []

    def test_multiple_duplicates(self):
        """Multiple pairs of duplicates should all be found."""
        assets = [
            AssetInfo("a", "id1", "svg", 100, "hash_a"),
            AssetInfo("b", "id2", "svg", 100, "hash_a"),
            AssetInfo("c", "id3", "png", 200, "hash_b"),
            AssetInfo("d", "id4", "png", 200, "hash_b"),
        ]
        duplicates = check_duplicate_assets(assets)
        assert len(duplicates) == 2

    def test_empty_list(self):
        """Empty asset list should produce no duplicates."""
        duplicates = check_duplicate_assets([])
        assert duplicates == []


class TestComputeAssetStats:
    """Tests for compute_asset_stats."""

    def test_empty_assets(self):
        """Empty asset list should produce zero stats."""
        stats = compute_asset_stats([])
        assert stats["total_count"] == 0
        assert stats["total_size_bytes"] == 0
        assert stats["total_size_formatted"] == "0 B"

    def test_total_count(self):
        """total_count should match number of assets."""
        assets = [
            AssetInfo("a", "1", "svg", 100, "h1"),
            AssetInfo("b", "2", "png", 200, "h2"),
            AssetInfo("c", "3", "wav", 300, "h3"),
        ]
        stats = compute_asset_stats(assets)
        assert stats["total_count"] == 3

    def test_total_size_bytes(self):
        """total_size_bytes should be sum of all asset sizes."""
        assets = [
            AssetInfo("a", "1", "svg", 100, "h1"),
            AssetInfo("b", "2", "png", 200, "h2"),
        ]
        stats = compute_asset_stats(assets)
        assert stats["total_size_bytes"] == 300

    def test_by_extension(self):
        """by_extension should group and count assets by extension."""
        assets = [
            AssetInfo("a", "1", "svg", 100, "h1"),
            AssetInfo("b", "2", "svg", 200, "h2"),
            AssetInfo("c", "3", "wav", 300, "h3"),
        ]
        stats = compute_asset_stats(assets)
        assert stats["by_extension"]["svg"]["count"] == 2
        assert stats["by_extension"]["svg"]["total_size_bytes"] == 300
        assert stats["by_extension"]["wav"]["count"] == 1

    def test_by_type_categorization(self):
        """by_type should categorize assets as image/audio/other."""
        assets = [
            AssetInfo("a", "1", "svg", 100, "h1"),
            AssetInfo("b", "2", "png", 200, "h2"),
            AssetInfo("c", "3", "wav", 300, "h3"),
            AssetInfo("d", "4", "mp3", 400, "h4"),
        ]
        stats = compute_asset_stats(assets)
        assert stats["by_type"]["image"] == 2
        assert stats["by_type"]["audio"] == 2
        assert stats["by_type"]["other"] == 0

    def test_total_size_formatted(self):
        """total_size_formatted should be a human-readable string."""
        assets = [
            AssetInfo("a", "1", "svg", 1024, "h1"),
            AssetInfo("b", "2", "png", 2048, "h2"),
        ]
        stats = compute_asset_stats(assets)
        assert stats["total_size_formatted"] == "3.0 KB"


class TestFormatSize:
    """Tests for format_size."""

    def test_bytes(self):
        """Small sizes should be shown in bytes."""
        assert format_size(0) == "0 B"
        assert format_size(42) == "42 B"
        assert format_size(1023) == "1023 B"

    def test_kilobytes(self):
        """Sizes >= 1024 should be shown in KB."""
        assert format_size(1024) == "1.0 KB"
        assert format_size(1536) == "1.5 KB"

    def test_megabytes(self):
        """Sizes >= 1024*1024 should be shown in MB."""
        assert format_size(1024 * 1024) == "1.0 MB"
        assert format_size(int(1.5 * 1024 * 1024)) == "1.5 MB"

    def test_gigabytes(self):
        """Sizes >= 1024^3 should be shown in GB."""
        assert format_size(1024 * 1024 * 1024) == "1.0 GB"

    def test_negative_size(self):
        """Negative sizes should return '0 B'."""
        assert format_size(-1) == "0 B"

    def test_large_terabytes(self):
        """Very large sizes should show in TB."""
        size = 2 * 1024 * 1024 * 1024 * 1024
        assert format_size(size) == "2.0 TB"
