//! Opcode definitions and dispatch for the Sailfish VM.
//!
//! Defines all supported opcodes as an enum, provides parsing from strings,
//! and categorizes opcodes for organizational purposes.

use crate::project::Value;
use crate::runtime::RuntimeState;
use thiserror::Error;

pub mod control;
pub mod events;
pub mod looks;
pub mod motion;
pub mod operators;
pub mod pen;
pub mod sensing;
pub mod sound;
pub mod variables;

/// Errors that can occur during opcode execution.
#[derive(Error, Debug)]
pub enum OpcodeError {
    #[error("unknown opcode: {0}")]
    UnknownOpcode(String),
    #[error("invalid argument for {opcode}: {message}")]
    InvalidArgument { opcode: String, message: String },
    #[error("missing argument for {opcode}: {name}")]
    MissingArgument { opcode: String, name: String },
    #[error("runtime error: {0}")]
    RuntimeError(String),
}

/// The category of an opcode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpcodeCategory {
    Motion,
    Looks,
    Sound,
    Events,
    Control,
    Sensing,
    Operators,
    Variables,
    Pen,
}

impl std::fmt::Display for OpcodeCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpcodeCategory::Motion => write!(f, "motion"),
            OpcodeCategory::Looks => write!(f, "looks"),
            OpcodeCategory::Sound => write!(f, "sound"),
            OpcodeCategory::Events => write!(f, "events"),
            OpcodeCategory::Control => write!(f, "control"),
            OpcodeCategory::Sensing => write!(f, "sensing"),
            OpcodeCategory::Operators => write!(f, "operators"),
            OpcodeCategory::Variables => write!(f, "variables"),
            OpcodeCategory::Pen => write!(f, "pen"),
        }
    }
}

/// All supported opcodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Opcode {
    // Motion
    MotionForward,
    MotionTurnRight,
    MotionTurnLeft,
    MotionGoto,
    MotionGotoxy,
    MotionSetX,
    MotionSetY,
    MotionChangeXBy,
    MotionChangeYBy,
    MotionPointInDirection,
    MotionPointTowards,
    MotionGlideTo,
    MotionGlideSecsToxy,
    MotionXPosition,
    MotionYPosition,
    MotionDirection,
    MotionBounceOffEdge,
    MotionSetRotationStyle,

    // Looks
    LooksSay,
    LooksSayForSecs,
    LooksThink,
    LooksThinkForSecs,
    LooksSwitchCostumeTo,
    LooksNextCostume,
    LooksSwitchBackdropTo,
    LooksNextBackdrop,
    LooksChangeSizeBy,
    LooksSetSizeTo,
    LooksChangeEffectBy,
    LooksSetEffectTo,
    LooksShow,
    LooksHide,
    LooksGoToFrontBack,
    LooksGoForwardBackwardLayers,
    LooksCostumeNumberName,
    LooksBackdropNumberName,
    LooksSize,

    // Sound
    SoundPlayUntilDone,
    SoundPlay,
    SoundStopAllSounds,
    SoundChangeEffectBy,
    SoundSetEffectTo,
    SoundChangeVolumeBy,
    SoundSetVolumeTo,
    SoundVolume,

    // Events
    EventWhenFlagClicked,
    EventWhenKeyPressed,
    EventWhenBackdropSwitchesTo,
    EventWhenBroadcastReceived,
    EventBroadcast,
    EventBroadcastAndWait,
    EventWhenGreaterThan,
    EventWhenTimerGreaterThan,
    EventWhenLoudnessGreaterThan,
    EventWhenVideoMotionGreaterThan,
    EventWhenCloneCreated,
    EventWhenStageClicked,
    EventWhenThisSpriteClicked,
    EventWhenTouchingObject,

    // Control
    ControlWait,
    ControlRepeat,
    ControlForever,
    ControlIf,
    ControlIfElse,
    ControlWaitUntil,
    ControlRepeatUntil,
    ControlStop,
    ControlStartAsClone,
    ControlCreateCloneOf,
    ControlDeleteThisClone,
    ControlRunWithoutScreenRefresh,

    // Sensing
    SensingAskAndWait,
    SensingAnswer,
    SensingKeyPressed,
    SensingMouseDown,
    SensingMouseX,
    SensingMouseY,
    SensingSetDragMode,
    SensingLoudness,
    SensingTimer,
    SensingResetTimer,
    SensingOf,
    SensingCurrent,
    SensingDaysSince2000,
    SensingUsername,
    SensingTouchingObject,
    SensingTouchingColor,
    SensingColorIsTouchingColor,
    SensingDistanceTo,

    // Operators
    OperatorAdd,
    OperatorSubtract,
    OperatorMultiply,
    OperatorDivide,
    OperatorRandom,
    OperatorGt,
    OperatorLt,
    OperatorEquals,
    OperatorAnd,
    OperatorOr,
    OperatorNot,
    OperatorJoin,
    OperatorLetterOf,
    OperatorLength,
    OperatorContains,
    OperatorMod,
    OperatorRound,
    OperatorMathop,

    // Variables
    DataSetVariableTo,
    DataChangeVariableBy,
    DataVariable,
    DataShowVariable,
    DataHideVariable,
    DataAddToList,
    DataDeleteOfList,
    DataDeleteAllOfList,
    DataInsertAtList,
    DataReplaceItemOfList,
    DataItemOfList,
    DataLengthOfList,
    DataListContainsItem,
    DataShowList,
    DataHideList,

    // Pen
    PenClear,
    PenStamp,
    PenPenDown,
    PenPenUp,
    PenSetPenColorToColor,
    PenChangePenColorParamBy,
    PenSetPenColorParamTo,
    PenChangePenSizeBy,
    PenSetPenSizeTo,
}

/// Parse an opcode string into an Opcode enum variant.
pub fn from_opcode_str(s: &str) -> Option<Opcode> {
    match s {
        // Motion
        "motion_forward" => Some(Opcode::MotionForward),
        "motion_turnright" => Some(Opcode::MotionTurnRight),
        "motion_turnleft" => Some(Opcode::MotionTurnLeft),
        "motion_goto" => Some(Opcode::MotionGoto),
        "motion_gotoxy" => Some(Opcode::MotionGotoxy),
        "motion_setx" => Some(Opcode::MotionSetX),
        "motion_sety" => Some(Opcode::MotionSetY),
        "motion_changexby" => Some(Opcode::MotionChangeXBy),
        "motion_changeyby" => Some(Opcode::MotionChangeYBy),
        "motion_pointindirection" => Some(Opcode::MotionPointInDirection),
        "motion_pointtowards" => Some(Opcode::MotionPointTowards),
        "motion_glideto" => Some(Opcode::MotionGlideTo),
        "motion_glidesecstoxy" => Some(Opcode::MotionGlideSecsToxy),
        "motion_xposition" => Some(Opcode::MotionXPosition),
        "motion_yposition" => Some(Opcode::MotionYPosition),
        "motion_direction" => Some(Opcode::MotionDirection),
        "motion_bounceoffedge" => Some(Opcode::MotionBounceOffEdge),
        "motion_setrotationstyle" => Some(Opcode::MotionSetRotationStyle),

        // Looks
        "looks_say" => Some(Opcode::LooksSay),
        "looks_sayforsecs" => Some(Opcode::LooksSayForSecs),
        "looks_think" => Some(Opcode::LooksThink),
        "looks_thinkforsecs" => Some(Opcode::LooksThinkForSecs),
        "looks_switchcostumeto" => Some(Opcode::LooksSwitchCostumeTo),
        "looks_nextcostume" => Some(Opcode::LooksNextCostume),
        "looks_switchbackdropto" => Some(Opcode::LooksSwitchBackdropTo),
        "looks_nextbackdrop" => Some(Opcode::LooksNextBackdrop),
        "looks_changesizeby" => Some(Opcode::LooksChangeSizeBy),
        "looks_setsizeto" => Some(Opcode::LooksSetSizeTo),
        "looks_changeeffectby" => Some(Opcode::LooksChangeEffectBy),
        "looks_seteffectto" => Some(Opcode::LooksSetEffectTo),
        "looks_show" => Some(Opcode::LooksShow),
        "looks_hide" => Some(Opcode::LooksHide),
        "looks_gotofrontback" => Some(Opcode::LooksGoToFrontBack),
        "looks_goforwardbackwardlayers" => Some(Opcode::LooksGoForwardBackwardLayers),
        "looks_costumenumbername" => Some(Opcode::LooksCostumeNumberName),
        "looks_backdropnumbername" => Some(Opcode::LooksBackdropNumberName),
        "looks_size" => Some(Opcode::LooksSize),

        // Sound
        "sound_playuntildone" => Some(Opcode::SoundPlayUntilDone),
        "sound_play" => Some(Opcode::SoundPlay),
        "sound_stopallsounds" => Some(Opcode::SoundStopAllSounds),
        "sound_changeeffectby" => Some(Opcode::SoundChangeEffectBy),
        "sound_seteffectto" => Some(Opcode::SoundSetEffectTo),
        "sound_changevolumeby" => Some(Opcode::SoundChangeVolumeBy),
        "sound_setvolumeto" => Some(Opcode::SoundSetVolumeTo),
        "sound_volume" => Some(Opcode::SoundVolume),

        // Events
        "event_whenflagclicked" => Some(Opcode::EventWhenFlagClicked),
        "event_whenkeypressed" => Some(Opcode::EventWhenKeyPressed),
        "event_whenbackdropswitchesto" => Some(Opcode::EventWhenBackdropSwitchesTo),
        "event_whenbroadcastreceived" => Some(Opcode::EventWhenBroadcastReceived),
        "event_broadcast" => Some(Opcode::EventBroadcast),
        "event_broadcastandwait" => Some(Opcode::EventBroadcastAndWait),
        "event_whengreaterthan" => Some(Opcode::EventWhenGreaterThan),
        "event_whentimergreaterthan" => Some(Opcode::EventWhenTimerGreaterThan),
        "event_whenloudnessgreaterthan" => Some(Opcode::EventWhenLoudnessGreaterThan),
        "event_whenvideomotiongreaterthan" => Some(Opcode::EventWhenVideoMotionGreaterThan),
        "event_whenclonecreated" => Some(Opcode::EventWhenCloneCreated),
        "event_whenstageclicked" => Some(Opcode::EventWhenStageClicked),
        "event_whenthisspriteclicked" => Some(Opcode::EventWhenThisSpriteClicked),
        "event_whentouchingobject" => Some(Opcode::EventWhenTouchingObject),

        // Control
        "control_wait" => Some(Opcode::ControlWait),
        "control_repeat" => Some(Opcode::ControlRepeat),
        "control_forever" => Some(Opcode::ControlForever),
        "control_if" => Some(Opcode::ControlIf),
        "control_if_else" => Some(Opcode::ControlIfElse),
        "control_wait_until" => Some(Opcode::ControlWaitUntil),
        "control_repeat_until" => Some(Opcode::ControlRepeatUntil),
        "control_stop" => Some(Opcode::ControlStop),
        "control_start_as_clone" => Some(Opcode::ControlStartAsClone),
        "control_create_clone_of" => Some(Opcode::ControlCreateCloneOf),
        "control_delete_this_clone" => Some(Opcode::ControlDeleteThisClone),
        "control_runwithoutscreenrefresh" => Some(Opcode::ControlRunWithoutScreenRefresh),

        // Sensing
        "sensing_askandwait" => Some(Opcode::SensingAskAndWait),
        "sensing_answer" => Some(Opcode::SensingAnswer),
        "sensing_keypressed" => Some(Opcode::SensingKeyPressed),
        "sensing_mousedown" => Some(Opcode::SensingMouseDown),
        "sensing_mousex" => Some(Opcode::SensingMouseX),
        "sensing_mousey" => Some(Opcode::SensingMouseY),
        "sensing_setdragmode" => Some(Opcode::SensingSetDragMode),
        "sensing_loudness" => Some(Opcode::SensingLoudness),
        "sensing_timer" => Some(Opcode::SensingTimer),
        "sensing_resettimer" => Some(Opcode::SensingResetTimer),
        "sensing_of" => Some(Opcode::SensingOf),
        "sensing_current" => Some(Opcode::SensingCurrent),
        "sensing_dayssince2000" => Some(Opcode::SensingDaysSince2000),
        "sensing_username" => Some(Opcode::SensingUsername),
        "sensing_touchingobject" => Some(Opcode::SensingTouchingObject),
        "sensing_touchingcolor" => Some(Opcode::SensingTouchingColor),
        "sensing_coloristouchingcolor" => Some(Opcode::SensingColorIsTouchingColor),
        "sensing_distanceto" => Some(Opcode::SensingDistanceTo),

        // Operators
        "operator_add" => Some(Opcode::OperatorAdd),
        "operator_subtract" => Some(Opcode::OperatorSubtract),
        "operator_multiply" => Some(Opcode::OperatorMultiply),
        "operator_divide" => Some(Opcode::OperatorDivide),
        "operator_random" => Some(Opcode::OperatorRandom),
        "operator_gt" => Some(Opcode::OperatorGt),
        "operator_lt" => Some(Opcode::OperatorLt),
        "operator_equals" => Some(Opcode::OperatorEquals),
        "operator_and" => Some(Opcode::OperatorAnd),
        "operator_or" => Some(Opcode::OperatorOr),
        "operator_not" => Some(Opcode::OperatorNot),
        "operator_join" => Some(Opcode::OperatorJoin),
        "operator_letter_of" => Some(Opcode::OperatorLetterOf),
        "operator_length" => Some(Opcode::OperatorLength),
        "operator_contains" => Some(Opcode::OperatorContains),
        "operator_mod" => Some(Opcode::OperatorMod),
        "operator_round" => Some(Opcode::OperatorRound),
        "operator_mathop" => Some(Opcode::OperatorMathop),

        // Variables
        "data_setvariableto" => Some(Opcode::DataSetVariableTo),
        "data_changevariableby" => Some(Opcode::DataChangeVariableBy),
        "data_variable" => Some(Opcode::DataVariable),
        "data_showvariable" => Some(Opcode::DataShowVariable),
        "data_hidevariable" => Some(Opcode::DataHideVariable),
        "data_addtolist" => Some(Opcode::DataAddToList),
        "data_deleteoflist" => Some(Opcode::DataDeleteOfList),
        "data_deletealloflist" => Some(Opcode::DataDeleteAllOfList),
        "data_insertatlist" => Some(Opcode::DataInsertAtList),
        "data_replaceitemoflist" => Some(Opcode::DataReplaceItemOfList),
        "data_itemoflist" => Some(Opcode::DataItemOfList),
        "data_lengthoflist" => Some(Opcode::DataLengthOfList),
        "data_listcontainsitem" => Some(Opcode::DataListContainsItem),
        "data_showlist" => Some(Opcode::DataShowList),
        "data_hidelist" => Some(Opcode::DataHideList),

        // Pen
        "pen_clear" => Some(Opcode::PenClear),
        "pen_stamp" => Some(Opcode::PenStamp),
        "pen_penDown" => Some(Opcode::PenPenDown),
        "pen_penUp" => Some(Opcode::PenPenUp),
        "pen_setPenColorToColor" => Some(Opcode::PenSetPenColorToColor),
        "pen_changePenColorParamBy" => Some(Opcode::PenChangePenColorParamBy),
        "pen_setPenColorParamTo" => Some(Opcode::PenSetPenColorParamTo),
        "pen_changePenSizeBy" => Some(Opcode::PenChangePenSizeBy),
        "pen_setPenSizeTo" => Some(Opcode::PenSetPenSizeTo),

        _ => None,
    }
}

impl Opcode {
    /// Get the category of this opcode.
    pub fn category(&self) -> OpcodeCategory {
        match self {
            // Motion
            Opcode::MotionForward
            | Opcode::MotionTurnRight
            | Opcode::MotionTurnLeft
            | Opcode::MotionGoto
            | Opcode::MotionGotoxy
            | Opcode::MotionSetX
            | Opcode::MotionSetY
            | Opcode::MotionChangeXBy
            | Opcode::MotionChangeYBy
            | Opcode::MotionPointInDirection
            | Opcode::MotionPointTowards
            | Opcode::MotionGlideTo
            | Opcode::MotionGlideSecsToxy
            | Opcode::MotionXPosition
            | Opcode::MotionYPosition
            | Opcode::MotionDirection
            | Opcode::MotionBounceOffEdge
            | Opcode::MotionSetRotationStyle => OpcodeCategory::Motion,

            // Looks
            Opcode::LooksSay
            | Opcode::LooksSayForSecs
            | Opcode::LooksThink
            | Opcode::LooksThinkForSecs
            | Opcode::LooksSwitchCostumeTo
            | Opcode::LooksNextCostume
            | Opcode::LooksSwitchBackdropTo
            | Opcode::LooksNextBackdrop
            | Opcode::LooksChangeSizeBy
            | Opcode::LooksSetSizeTo
            | Opcode::LooksChangeEffectBy
            | Opcode::LooksSetEffectTo
            | Opcode::LooksShow
            | Opcode::LooksHide
            | Opcode::LooksGoToFrontBack
            | Opcode::LooksGoForwardBackwardLayers
            | Opcode::LooksCostumeNumberName
            | Opcode::LooksBackdropNumberName
            | Opcode::LooksSize => OpcodeCategory::Looks,

            // Sound
            Opcode::SoundPlayUntilDone
            | Opcode::SoundPlay
            | Opcode::SoundStopAllSounds
            | Opcode::SoundChangeEffectBy
            | Opcode::SoundSetEffectTo
            | Opcode::SoundChangeVolumeBy
            | Opcode::SoundSetVolumeTo
            | Opcode::SoundVolume => OpcodeCategory::Sound,

            // Events
            Opcode::EventWhenFlagClicked
            | Opcode::EventWhenKeyPressed
            | Opcode::EventWhenBackdropSwitchesTo
            | Opcode::EventWhenBroadcastReceived
            | Opcode::EventBroadcast
            | Opcode::EventBroadcastAndWait
            | Opcode::EventWhenGreaterThan
            | Opcode::EventWhenTimerGreaterThan
            | Opcode::EventWhenLoudnessGreaterThan
            | Opcode::EventWhenVideoMotionGreaterThan
            | Opcode::EventWhenCloneCreated
            | Opcode::EventWhenStageClicked
            | Opcode::EventWhenThisSpriteClicked
            | Opcode::EventWhenTouchingObject => OpcodeCategory::Events,

            // Control
            Opcode::ControlWait
            | Opcode::ControlRepeat
            | Opcode::ControlForever
            | Opcode::ControlIf
            | Opcode::ControlIfElse
            | Opcode::ControlWaitUntil
            | Opcode::ControlRepeatUntil
            | Opcode::ControlStop
            | Opcode::ControlStartAsClone
            | Opcode::ControlCreateCloneOf
            | Opcode::ControlDeleteThisClone
            | Opcode::ControlRunWithoutScreenRefresh => OpcodeCategory::Control,

            // Sensing
            Opcode::SensingAskAndWait
            | Opcode::SensingAnswer
            | Opcode::SensingKeyPressed
            | Opcode::SensingMouseDown
            | Opcode::SensingMouseX
            | Opcode::SensingMouseY
            | Opcode::SensingSetDragMode
            | Opcode::SensingLoudness
            | Opcode::SensingTimer
            | Opcode::SensingResetTimer
            | Opcode::SensingOf
            | Opcode::SensingCurrent
            | Opcode::SensingDaysSince2000
            | Opcode::SensingUsername
            | Opcode::SensingTouchingObject
            | Opcode::SensingTouchingColor
            | Opcode::SensingColorIsTouchingColor
            | Opcode::SensingDistanceTo => OpcodeCategory::Sensing,

            // Operators
            Opcode::OperatorAdd
            | Opcode::OperatorSubtract
            | Opcode::OperatorMultiply
            | Opcode::OperatorDivide
            | Opcode::OperatorRandom
            | Opcode::OperatorGt
            | Opcode::OperatorLt
            | Opcode::OperatorEquals
            | Opcode::OperatorAnd
            | Opcode::OperatorOr
            | Opcode::OperatorNot
            | Opcode::OperatorJoin
            | Opcode::OperatorLetterOf
            | Opcode::OperatorLength
            | Opcode::OperatorContains
            | Opcode::OperatorMod
            | Opcode::OperatorRound
            | Opcode::OperatorMathop => OpcodeCategory::Operators,

            // Variables
            Opcode::DataSetVariableTo
            | Opcode::DataChangeVariableBy
            | Opcode::DataVariable
            | Opcode::DataShowVariable
            | Opcode::DataHideVariable
            | Opcode::DataAddToList
            | Opcode::DataDeleteOfList
            | Opcode::DataDeleteAllOfList
            | Opcode::DataInsertAtList
            | Opcode::DataReplaceItemOfList
            | Opcode::DataItemOfList
            | Opcode::DataLengthOfList
            | Opcode::DataListContainsItem
            | Opcode::DataShowList
            | Opcode::DataHideList => OpcodeCategory::Variables,

            // Pen
            Opcode::PenClear
            | Opcode::PenStamp
            | Opcode::PenPenDown
            | Opcode::PenPenUp
            | Opcode::PenSetPenColorToColor
            | Opcode::PenChangePenColorParamBy
            | Opcode::PenSetPenColorParamTo
            | Opcode::PenChangePenSizeBy
            | Opcode::PenSetPenSizeTo => OpcodeCategory::Pen,
        }
    }

    /// Execute this opcode against the runtime.
    pub fn execute(
        &self,
        runtime: &mut RuntimeState,
        args: &Value,
    ) -> Result<Value, OpcodeError> {
        match self.category() {
            OpcodeCategory::Motion => motion::execute(self, runtime, args),
            OpcodeCategory::Looks => looks::execute(self, runtime, args),
            OpcodeCategory::Sound => sound::execute(self, runtime, args),
            OpcodeCategory::Events => events::execute(self, runtime, args),
            OpcodeCategory::Control => control::execute(self, runtime, args),
            OpcodeCategory::Sensing => sensing::execute(self, runtime, args),
            OpcodeCategory::Operators => operators::execute(self, runtime, args),
            OpcodeCategory::Variables => variables::execute(self, runtime, args),
            OpcodeCategory::Pen => pen::execute(self, runtime, args),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_opcode_str_motion() {
        assert_eq!(from_opcode_str("motion_forward"), Some(Opcode::MotionForward));
        assert_eq!(from_opcode_str("motion_turnright"), Some(Opcode::MotionTurnRight));
        assert_eq!(from_opcode_str("motion_turnleft"), Some(Opcode::MotionTurnLeft));
        assert_eq!(from_opcode_str("motion_goto"), Some(Opcode::MotionGoto));
        assert_eq!(from_opcode_str("motion_xposition"), Some(Opcode::MotionXPosition));
    }

    #[test]
    fn test_from_opcode_str_looks() {
        assert_eq!(from_opcode_str("looks_say"), Some(Opcode::LooksSay));
        assert_eq!(from_opcode_str("looks_think"), Some(Opcode::LooksThink));
        assert_eq!(from_opcode_str("looks_show"), Some(Opcode::LooksShow));
        assert_eq!(from_opcode_str("looks_hide"), Some(Opcode::LooksHide));
    }

    #[test]
    fn test_from_opcode_str_sound() {
        assert_eq!(from_opcode_str("sound_play"), Some(Opcode::SoundPlay));
        assert_eq!(from_opcode_str("sound_stopallsounds"), Some(Opcode::SoundStopAllSounds));
    }

    #[test]
    fn test_from_opcode_str_events() {
        assert_eq!(from_opcode_str("event_whenflagclicked"), Some(Opcode::EventWhenFlagClicked));
        assert_eq!(from_opcode_str("event_whenkeypressed"), Some(Opcode::EventWhenKeyPressed));
        assert_eq!(from_opcode_str("event_broadcast"), Some(Opcode::EventBroadcast));
    }

    #[test]
    fn test_from_opcode_str_control() {
        assert_eq!(from_opcode_str("control_wait"), Some(Opcode::ControlWait));
        assert_eq!(from_opcode_str("control_repeat"), Some(Opcode::ControlRepeat));
        assert_eq!(from_opcode_str("control_forever"), Some(Opcode::ControlForever));
        assert_eq!(from_opcode_str("control_if"), Some(Opcode::ControlIf));
        assert_eq!(from_opcode_str("control_if_else"), Some(Opcode::ControlIfElse));
    }

    #[test]
    fn test_from_opcode_str_sensing() {
        assert_eq!(from_opcode_str("sensing_askandwait"), Some(Opcode::SensingAskAndWait));
        assert_eq!(from_opcode_str("sensing_timer"), Some(Opcode::SensingTimer));
        assert_eq!(from_opcode_str("sensing_resettimer"), Some(Opcode::SensingResetTimer));
    }

    #[test]
    fn test_from_opcode_str_operators() {
        assert_eq!(from_opcode_str("operator_add"), Some(Opcode::OperatorAdd));
        assert_eq!(from_opcode_str("operator_subtract"), Some(Opcode::OperatorSubtract));
        assert_eq!(from_opcode_str("operator_multiply"), Some(Opcode::OperatorMultiply));
        assert_eq!(from_opcode_str("operator_random"), Some(Opcode::OperatorRandom));
    }

    #[test]
    fn test_from_opcode_str_variables() {
        assert_eq!(from_opcode_str("data_setvariableto"), Some(Opcode::DataSetVariableTo));
        assert_eq!(from_opcode_str("data_changevariableby"), Some(Opcode::DataChangeVariableBy));
    }

    #[test]
    fn test_from_opcode_str_pen() {
        assert_eq!(from_opcode_str("pen_clear"), Some(Opcode::PenClear));
        assert_eq!(from_opcode_str("pen_stamp"), Some(Opcode::PenStamp));
        assert_eq!(from_opcode_str("pen_penDown"), Some(Opcode::PenPenDown));
        assert_eq!(from_opcode_str("pen_penUp"), Some(Opcode::PenPenUp));
    }

    #[test]
    fn test_from_opcode_str_unknown() {
        assert_eq!(from_opcode_str("nonexistent_opcode"), None);
        assert_eq!(from_opcode_str(""), None);
    }

    #[test]
    fn test_opcode_category() {
        assert_eq!(Opcode::MotionForward.category(), OpcodeCategory::Motion);
        assert_eq!(Opcode::LooksSay.category(), OpcodeCategory::Looks);
        assert_eq!(Opcode::SoundPlay.category(), OpcodeCategory::Sound);
        assert_eq!(Opcode::EventWhenFlagClicked.category(), OpcodeCategory::Events);
        assert_eq!(Opcode::ControlWait.category(), OpcodeCategory::Control);
        assert_eq!(Opcode::SensingTimer.category(), OpcodeCategory::Sensing);
        assert_eq!(Opcode::OperatorAdd.category(), OpcodeCategory::Operators);
        assert_eq!(Opcode::DataSetVariableTo.category(), OpcodeCategory::Variables);
        assert_eq!(Opcode::PenClear.category(), OpcodeCategory::Pen);
    }

    #[test]
    fn test_opcode_category_display() {
        assert_eq!(format!("{}", OpcodeCategory::Motion), "motion");
        assert_eq!(format!("{}", OpcodeCategory::Looks), "looks");
        assert_eq!(format!("{}", OpcodeCategory::Sound), "sound");
        assert_eq!(format!("{}", OpcodeCategory::Events), "events");
        assert_eq!(format!("{}", OpcodeCategory::Control), "control");
        assert_eq!(format!("{}", OpcodeCategory::Sensing), "sensing");
        assert_eq!(format!("{}", OpcodeCategory::Operators), "operators");
        assert_eq!(format!("{}", OpcodeCategory::Variables), "variables");
        assert_eq!(format!("{}", OpcodeCategory::Pen), "pen");
    }

    #[test]
    fn test_opcode_count() {
        // Ensure we have a substantial number of opcodes
        let test_opcodes = [
            "motion_forward", "motion_turnright", "motion_turnleft", "motion_goto",
            "motion_gotoxy", "motion_setx", "motion_sety", "motion_changexby",
            "motion_changeyby", "motion_pointindirection", "motion_xposition",
            "motion_yposition", "motion_direction",
            "looks_say", "looks_think", "looks_show", "looks_hide",
            "looks_switchcostumeto", "looks_nextcostume", "looks_changesizeby",
            "looks_setsizeto", "looks_size",
            "sound_play", "sound_stopallsounds",
            "event_whenflagclicked", "event_whenkeypressed", "event_broadcast",
            "control_wait", "control_repeat", "control_forever",
            "control_if", "control_if_else", "control_stop",
            "sensing_askandwait", "sensing_timer", "sensing_resettimer",
            "sensing_keypressed", "sensing_mousex", "sensing_mousey",
            "operator_add", "operator_subtract", "operator_multiply",
            "operator_divide", "operator_random", "operator_not",
            "data_setvariableto", "data_changevariableby", "data_variable",
            "pen_clear", "pen_stamp", "pen_penDown", "pen_penUp",
        ];
        let count = test_opcodes.iter().filter(|op| from_opcode_str(op).is_some()).count();
        assert_eq!(count, test_opcodes.len(), "All test opcodes should parse");
        assert!(count >= 50, "Should have at least 50 opcodes, got {}", count);
    }
}
