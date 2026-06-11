//! Motion blur trail renderer — maintains a short history of positions and
//! computes per-frame alpha values for a "ghosting" effect.

use serde::{Deserialize, Serialize};

// ── Types ────────────────────────────────────────────────────────────────────

/// Configuration for the trail effect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailConfig {
    /// Maximum number of ghost frames kept.
    pub frame_count: usize,
    /// Alpha of the most recent (brightest) ghost frame.
    pub max_alpha: f64,
    /// Alpha of the oldest (faintest) ghost frame.
    pub min_alpha: f64,
}

impl Default for TrailConfig {
    fn default() -> Self {
        Self {
            frame_count: 5,
            max_alpha: 0.3,
            min_alpha: 0.05,
        }
    }
}

impl TrailConfig {
    pub fn new(frame_count: usize, max_alpha: f64, min_alpha: f64) -> Self {
        Self {
            frame_count,
            max_alpha,
            min_alpha,
        }
    }
}

/// A single ghost frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailFrame {
    pub x: f64,
    pub y: f64,
    pub alpha: f64,
}

/// Holds the trail history and configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailState {
    pub frames: Vec<TrailFrame>,
    pub config: TrailConfig,
}

impl TrailState {
    pub fn new(config: TrailConfig) -> Self {
        Self {
            frames: Vec::new(),
            config,
        }
    }

    /// Create state with default config.
    pub fn with_defaults() -> Self {
        Self::new(TrailConfig::default())
    }
}

// ── Trail logic ──────────────────────────────────────────────────────────────

/// Add a new position frame. If the number of frames exceeds the configured
/// `frame_count`, the oldest frame is removed.
pub fn update_trail(state: &mut TrailState, x: f64, y: f64) {
    // Use max_alpha as the initial alpha for the newest frame.
    state.frames.push(TrailFrame {
        x,
        y,
        alpha: state.config.max_alpha,
    });

    // Trim oldest frames if we exceed the limit.
    while state.frames.len() > state.config.frame_count {
        state.frames.remove(0);
    }
}

/// Compute alpha values for all frames using linear interpolation from
/// `max_alpha` (newest) to `min_alpha` (oldest).
///
/// If there is only one frame, its alpha is `max_alpha`.
pub fn compute_trail_alphas(state: &TrailState) -> Vec<f64> {
    let n = state.frames.len();
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![state.config.max_alpha];
    }

    let max = state.config.max_alpha;
    let min = state.config.min_alpha;
    let step = (max - min) / (n - 1) as f64;

    // Oldest frame first → index 0 gets min_alpha.
    (0..n)
        .map(|i| min + step * i as f64)
        .collect()
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_trail_adds_frames_and_trims() {
        let mut state = TrailState::new(TrailConfig::new(3, 0.3, 0.05));

        update_trail(&mut state, 10.0, 20.0);
        assert_eq!(state.frames.len(), 1);

        update_trail(&mut state, 15.0, 25.0);
        update_trail(&mut state, 20.0, 30.0);
        assert_eq!(state.frames.len(), 3, "should be at max frame_count");

        update_trail(&mut state, 25.0, 35.0);
        assert_eq!(state.frames.len(), 3, "should trim to frame_count");

        // Oldest frame should have been removed.
        assert!(
            (state.frames[0].x - 15.0).abs() < 0.01,
            "oldest frame should be the second one added"
        );
    }

    #[test]
    fn compute_trail_alphas_linear_interpolation() {
        let mut state = TrailState::new(TrailConfig::new(5, 0.3, 0.05));

        // Add 5 frames.
        for i in 0..5 {
            update_trail(&mut state, i as f64, i as f64);
        }

        let alphas = compute_trail_alphas(&state);
        assert_eq!(alphas.len(), 5);

        // Oldest → min_alpha, newest → max_alpha.
        assert!(
            (alphas[0] - 0.05).abs() < 0.001,
            "oldest frame should have min_alpha, got {}",
            alphas[0]
        );
        assert!(
            (alphas[4] - 0.3).abs() < 0.001,
            "newest frame should have max_alpha, got {}",
            alphas[4]
        );

        // Linear step.
        let step = (0.3 - 0.05) / 4.0;
        for i in 0..5 {
            let expected = 0.05 + step * i as f64;
            assert!(
                (alphas[i] - expected).abs() < 0.001,
                "alpha[{}] = {}, expected {}",
                i,
                alphas[i],
                expected
            );
        }
    }

    #[test]
    fn compute_trail_alphas_single_frame() {
        let mut state = TrailState::with_defaults();
        update_trail(&mut state, 0.0, 0.0);

        let alphas = compute_trail_alphas(&state);
        assert_eq!(alphas.len(), 1);
        assert!(
            (alphas[0] - state.config.max_alpha).abs() < 0.001,
            "single frame should have max_alpha"
        );
    }

    #[test]
    fn compute_trail_alphas_empty() {
        let state = TrailState::with_defaults();
        let alphas = compute_trail_alphas(&state);
        assert!(alphas.is_empty(), "empty state should produce no alphas");
    }

    #[test]
    fn default_config_values() {
        let cfg = TrailConfig::default();
        assert_eq!(cfg.frame_count, 5);
        assert!((cfg.max_alpha - 0.3).abs() < 0.001);
        assert!((cfg.min_alpha - 0.05).abs() < 0.001);
    }

    #[test]
    fn trail_frame_positions_preserved() {
        let mut state = TrailState::new(TrailConfig::new(10, 0.3, 0.05));
        update_trail(&mut state, 1.0, 2.0);
        update_trail(&mut state, 3.0, 4.0);
        update_trail(&mut state, 5.0, 6.0);

        assert!((state.frames[0].x - 1.0).abs() < 0.01);
        assert!((state.frames[0].y - 2.0).abs() < 0.01);
        assert!((state.frames[2].x - 5.0).abs() < 0.01);
        assert!((state.frames[2].y - 6.0).abs() < 0.01);
    }
}
