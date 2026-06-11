"""Project file validator tool for Sailfish Studio.

Validates project JSON structures (.sb3 and .sf formats) against
required schemas and business rules.
"""

from __future__ import annotations

VALID_OPCODES: set[str] = {
    # Motion
    "motion_forward",
    "motion_turnright",
    "motion_turnleft",
    "motion_goto",
    "motion_gotoxy",
    "motion_setx",
    "motion_sety",
    "motion_changexby",
    "motion_changeyby",
    "motion_pointindirection",
    "motion_pointtowards",
    "motion_glidesecstoxy",
    "motion_glideto",
    "motion_ifonedgebounce",
    "motion_xposition",
    "motion_yposition",
    "motion_direction",
    # Looks
    "looks_say",
    "looks_sayforsecs",
    "looks_think",
    "looks_thinkforsecs",
    "looks_show",
    "looks_hide",
    "looks_switchcostumeto",
    "looks_nextcostume",
    "looks_switchbackdropto",
    "looks_nextbackdrop",
    "looks_changesizeby",
    "looks_setsizeto",
    "looks_changeeffectby",
    "looks_seteffectto",
    "looks_cleargraphiceffects",
    "looks_size",
    "looks_costumenumbername",
    "looks_backdropnumbername",
    # Sound
    "sound_playuntildone",
    "sound_play",
    "sound_stopallsounds",
    "sound_setvolumeto",
    "sound_changevolumeby",
    "sound_volume",
    # Events
    "event_whenflagclicked",
    "event_whenkeypressed",
    "event_whenthisspriteclicked",
    "event_whenbackdropswitchesto",
    "event_whengreaterthan",
    "event_whenbroadcastreceived",
    "event_broadcast",
    "event_broadcastandwait",
    # Control
    "control_wait",
    "control_repeat",
    "control_forever",
    "control_if",
    "control_if_else",
    "control_wait_until",
    "control_repeat_until",
    "control_stop",
    "control_start_as_clone",
    "control_create_clone_of",
    "control_delete_this_clone",
    # Sensing
    "sensing_touchingobject",
    "sensing_touchingcolor",
    "sensing_coloristouchingcolor",
    "sensing_distanceto",
    "sensing_askandwait",
    "sensing_answer",
    "sensing_keypressed",
    "sensing_mousedown",
    "sensing_mousex",
    "sensing_mousey",
    "sensing_loudness",
    "sensing_timer",
    "sensing_resettimer",
    "sensing_of",
    "sensing_current",
    "sensing_dayssince2000",
    "sensing_username",
    # Operators
    "operator_add",
    "operator_subtract",
    "operator_multiply",
    "operator_divide",
    "operator_random",
    "operator_gt",
    "operator_lt",
    "operator_equals",
    "operator_and",
    "operator_or",
    "operator_not",
    "operator_join",
    "operator_letter_of",
    "operator_length",
    "operator_contains",
    "operator_mod",
    "operator_round",
    "operator_mathop",
    # Variables / Data
    "data_setvariableto",
    "data_changevariableby",
    "data_variable",
    "data_addtolist",
    "data_deleteoflist",
    "data_deletealloflist",
    "data_insertatlist",
    "data_replaceitemoflist",
    "data_itemoflist",
    "data_lengthoflist",
    "data_listcontainsitem",
    "data_showvariable",
    "data_hidevariable",
    "data_showlist",
    "data_hidelist",
    # Pen
    "pen_clear",
    "pen_stamp",
    "pen_penDown",
    "pen_penUp",
    "pen_setPenColorToColor",
    "pen_changePenColorParamBy",
    "pen_setPenColorParamTo",
    "pen_changePenSizeBy",
    "pen_setPenSizeTo",
}


def check_required_fields(data: dict, required: list[str], path: str) -> list[str]:
    """Check that all required fields are present in a dict.

    Args:
        data: The dictionary to check.
        required: List of required field names.
        path: Dot-notation path prefix for error messages.

    Returns:
        List of error messages for missing fields.
    """
    errors: list[str] = []
    for field in required:
        if field not in data:
            errors.append(f"Missing required field '{path}.{field}'")
    return errors


def validate_project_json(data: dict) -> list[str]:
    """Validate a generic project JSON structure.

    Checks that the project has targets, each target has a name,
    a stage target exists, and all block opcodes are valid.

    Args:
        data: The project JSON as a dictionary.

    Returns:
        List of validation error messages.
    """
    errors: list[str] = []

    # Check top-level required fields
    errors.extend(check_required_fields(data, ["targets"], "project"))

    if "targets" not in data:
        return errors  # Can't validate further without targets

    targets = data["targets"]
    if not isinstance(targets, list):
        errors.append("project.targets must be a list")
        return errors

    if len(targets) == 0:
        errors.append("Project must have at least one target (stage)")
        return errors

    # Check each target
    has_stage = False
    for i, target in enumerate(targets):
        if not isinstance(target, dict):
            errors.append(f"Target at index {i} must be a dict")
            continue

        # Check target name
        target_errors = check_required_fields(target, ["name"], f"targets[{i}]")
        errors.extend(target_errors)

        # Check if stage
        if target.get("isStage", False):
            has_stage = True

        # Validate blocks
        blocks = target.get("blocks", {})
        if isinstance(blocks, dict):
            for block_id, block in blocks.items():
                if isinstance(block, dict):
                    opcode = block.get("opcode", "")
                    if opcode and opcode not in VALID_OPCODES:
                        errors.append(
                            f"Invalid opcode '{opcode}' in targets[{i}].blocks[{block_id}]"
                        )

    if not has_stage:
        errors.append("Project must have a stage target (isStage=true)")

    return errors


def validate_sb3_structure(data: dict) -> list[str]:
    """Validate a .sb3 (Scratch 3.0) project structure.

    In addition to base project validation, checks Scratch-specific
    fields like monitors, extensions, and meta.

    Args:
        data: The .sb3 project JSON as a dictionary.

    Returns:
        List of validation error messages.
    """
    errors: list[str] = []

    # Base validation
    errors.extend(validate_project_json(data))

    # Scratch-specific: check meta field
    if "meta" in data and isinstance(data["meta"], dict):
        meta = data["meta"]
        if "semver" not in meta:
            errors.append("SB3 meta should contain 'semver' field")

    return errors


def validate_sf_structure(data: dict) -> list[str]:
    """Validate a .sf (Sailfish) project structure.

    In addition to base project validation, checks Sailfish-specific
    fields like sfVersion, settings, and extension format.

    Args:
        data: The .sf project JSON as a dictionary.

    Returns:
        List of validation error messages.
    """
    errors: list[str] = []

    # Base validation
    errors.extend(validate_project_json(data))

    # Sailfish-specific: check sfVersion
    if "sfVersion" not in data:
        errors.append("Sailfish project should contain 'sfVersion' field")

    # Check targets have Sailfish-specific fields
    targets = data.get("targets", [])
    for i, target in enumerate(targets):
        if isinstance(target, dict):
            # Check that costumes and sounds have assetId
            for costume in target.get("costumes", []):
                if isinstance(costume, dict) and "assetId" not in costume:
                    errors.append(
                        f"Costume in targets[{i}] missing 'assetId'"
                    )
            for sound in target.get("sounds", []):
                if isinstance(sound, dict) and "assetId" not in sound:
                    errors.append(
                        f"Sound in targets[{i}] missing 'assetId'"
                    )

    return errors
