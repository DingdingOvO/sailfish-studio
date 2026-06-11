"""Asset management utilities for Sailfish Studio.

Provides tools for scanning, deduplication, and statistics of project assets
(costumes, sounds, backdrops, etc.).
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass
class AssetInfo:
    """Information about a single project asset."""

    name: str
    asset_id: str
    file_extension: str
    size_bytes: int
    md5_hash: str


def scan_assets(project_data: dict) -> list[AssetInfo]:
    """Extract all asset references from a project structure.

    Scans costumes and sounds from all targets in the project.

    Args:
        project_data: The project JSON as a dictionary.

    Returns:
        List of AssetInfo objects for all found assets.
    """
    assets: list[AssetInfo] = []

    targets = project_data.get("targets", [])
    if not isinstance(targets, list):
        return assets

    for target in targets:
        if not isinstance(target, dict):
            continue

        # Scan costumes
        costumes = target.get("costumes", [])
        if isinstance(costumes, list):
            for costume in costumes:
                if isinstance(costume, dict):
                    asset = _extract_asset(costume, "costume")
                    if asset is not None:
                        assets.append(asset)

        # Scan sounds
        sounds = target.get("sounds", [])
        if isinstance(sounds, list):
            for sound in sounds:
                if isinstance(sound, dict):
                    asset = _extract_asset(sound, "sound")
                    if asset is not None:
                        assets.append(asset)

    return assets


def _extract_asset(data: dict[str, Any], asset_type: str) -> AssetInfo | None:
    """Extract an AssetInfo from a costume or sound dict.

    Args:
        data: The costume/sound dictionary.
        asset_type: "costume" or "sound" (for error context).

    Returns:
        AssetInfo if required fields are present, None otherwise.
    """
    name = data.get("name", "")
    asset_id = data.get("assetId", data.get("md5ext", ""))
    file_extension = data.get("dataFormat", "")
    size_bytes = data.get("size", 0)

    # md5ext often contains "hash.ext" format
    md5_hash = data.get("md5ext", data.get("assetId", ""))
    if "." in md5_hash:
        # Extract just the hash part before the extension
        md5_hash = md5_hash.split(".")[0]

    if not asset_id:
        return None

    # If file_extension is empty, try to extract from md5ext
    if not file_extension:
        md5ext = data.get("md5ext", "")
        if "." in md5ext:
            file_extension = md5ext.split(".")[-1]

    return AssetInfo(
        name=name,
        asset_id=asset_id,
        file_extension=file_extension,
        size_bytes=size_bytes,
        md5_hash=md5_hash,
    )


def check_duplicate_assets(assets: list[AssetInfo]) -> list[tuple[AssetInfo, AssetInfo]]:
    """Find duplicate assets by their MD5 hash.

    Two assets are considered duplicates if they have the same md5_hash.
    Assets with empty md5_hash are ignored.

    Args:
        assets: List of AssetInfo objects to check.

    Returns:
        List of (asset1, asset2) tuples where both assets share the same hash.
    """
    duplicates: list[tuple[AssetInfo, AssetInfo]] = []
    seen: dict[str, AssetInfo] = {}

    for asset in assets:
        if not asset.md5_hash:
            continue

        if asset.md5_hash in seen:
            duplicates.append((seen[asset.md5_hash], asset))
        else:
            seen[asset.md5_hash] = asset

    return duplicates


def compute_asset_stats(assets: list[AssetInfo]) -> dict[str, Any]:
    """Compute statistics about a list of assets.

    Args:
        assets: List of AssetInfo objects.

    Returns:
        Dictionary with:
        - total_count: total number of assets
        - total_size_bytes: sum of all asset sizes
        - total_size_formatted: human-readable total size
        - by_extension: dict mapping extension to count and total size
        - by_type: dict mapping "costume"/"sound" category to count
    """
    total_count = len(assets)
    total_size_bytes = sum(a.size_bytes for a in assets)

    by_extension: dict[str, dict[str, Any]] = {}
    for asset in assets:
        ext = asset.file_extension or "unknown"
        if ext not in by_extension:
            by_extension[ext] = {"count": 0, "total_size_bytes": 0}
        by_extension[ext]["count"] += 1
        by_extension[ext]["total_size_bytes"] += asset.size_bytes

    # Categorize by common extensions
    by_type: dict[str, int] = {"image": 0, "audio": 0, "other": 0}
    image_extensions = {"svg", "png", "jpg", "jpeg", "gif", "bmp", "webp"}
    audio_extensions = {"wav", "mp3", "ogg", "flac", "aac", "m4a"}

    for asset in assets:
        ext = (asset.file_extension or "").lower()
        if ext in image_extensions:
            by_type["image"] += 1
        elif ext in audio_extensions:
            by_type["audio"] += 1
        else:
            by_type["other"] += 1

    return {
        "total_count": total_count,
        "total_size_bytes": total_size_bytes,
        "total_size_formatted": format_size(total_size_bytes),
        "by_extension": by_extension,
        "by_type": by_type,
    }


def format_size(size_bytes: int) -> str:
    """Format a byte count as a human-readable file size string.

    Uses binary prefixes (1 KB = 1024 bytes).

    Args:
        size_bytes: Size in bytes.

    Returns:
        Human-readable string like "1.2 MB", "512.0 KB", "42 B".
    """
    if size_bytes < 0:
        return "0 B"

    units = ["B", "KB", "MB", "GB", "TB"]
    size = float(size_bytes)

    for unit in units:
        if abs(size) < 1024.0 or unit == units[-1]:
            if unit == "B":
                return f"{int(size)} {unit}"
            return f"{size:.1f} {unit}"
        size /= 1024.0

    # Should not reach here
    return f"{size_bytes} B"
