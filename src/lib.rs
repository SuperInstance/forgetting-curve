//! # forgetting-curve
//!
//! Ebbinghaus forgetting curve implementation with SM-2 spaced repetition
//! scheduling and retention prediction.

use std::collections::HashMap;

/// Core forgetting curve: `R = e^(-t / S)`.
///
/// `R` is the retrievability (recall probability), `t` is time elapsed
/// since last review, and `S` is memory stability (strength).
#[derive(Debug, Clone, Copy)]
pub struct ForgettingCurve;

impl ForgettingCurve {
    /// Compute recall probability `R` at time `t` given stability `S`.
    pub fn retrievability(t: f64, stability: f64) -> f64 {
        if stability <= 0.0 {
            return 0.0;
        }
        (-t / stability).exp()
    }

    /// Compute the time at which retrievability drops to `target_r`.
    /// `t = -S * ln(target_r)`
    pub fn time_to_decay(stability: f64, target_r: f64) -> f64 {
        if target_r <= 0.0 || target_r >= 1.0 || stability <= 0.0 {
            return 0.0;
        }
        -stability * target_r.ln()
    }
}

/// Memory strength tracker: initial strength with rehearsal boosts.
#[derive(Debug, Clone)]
pub struct MemoryStrength {
    /// Current stability value.
    pub stability: f64,
    /// Number of rehearsals applied.
    pub rehearsals: u32,
}

impl MemoryStrength {
    /// Create a new memory strength with initial stability.
    pub fn new(initial: f64) -> Self {
        Self {
            stability: initial,
            rehearsals: 0,
        }
    }

    /// Default initial stability of 1.0.
    pub fn default_initial() -> f64 {
        1.0
    }

    /// Apply a rehearsal: multiplies stability by `factor`.
    /// Factor should be > 1.0 for meaningful boost.
    pub fn rehearse(&mut self, factor: f64) {
        self.stability *= factor;
        self.rehearsals += 1;
    }

    /// Decay stability by a multiplier (0.0–1.0) representing natural decay.
    pub fn decay(&mut self, factor: f64) {
        self.stability *= factor;
    }
}

/// SM-2 quality grade (0–5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    /// Complete blackout.
    Zero = 0,
    /// Incorrect; correct answer remembered after seeing it.
    One = 1,
    /// Incorrect; correct answer seemed easy to recall.
    Two = 2,
    /// Correct with serious difficulty.
    Three = 3,
    /// Correct after hesitation.
    Four = 4,
    /// Perfect response.
    Five = 5,
}

impl Quality {
    /// Convert to numeric value.
    pub fn value(&self) -> u32 {
        *self as u32
    }
}

/// SM-2 spaced repetition scheduler.
#[derive(Debug, Clone)]
pub struct SpacedRepetition {
    /// Easiness factor (≥ 1.3).
    pub easiness: f64,
    /// Current interval in days.
    pub interval: f64,
    /// Number of repetitions.
    pub repetition: u32,
}

impl SpacedRepetition {
    /// Create a new SM-2 scheduler with defaults.
    pub fn new() -> Self {
        Self {
            easiness: 2.5,
            interval: 0.0,
            repetition: 0,
        }
    }

    /// SM-2 easiness factor update: `EF' = EF + (0.1 - (5-q)*(0.08+(5-q)*0.02))`
    pub fn update_easiness(&mut self, q: Quality) {
        let q = q.value() as f64;
        self.easiness += 0.1 - (5.0 - q) * (0.08 + (5.0 - q) * 0.02);
        if self.easiness < 1.3 {
            self.easiness = 1.3;
        }
    }

    /// Review with the given quality grade; returns the next interval in days.
    pub fn review(&mut self, q: Quality) -> f64 {
        self.update_easiness(q);
        if q.value() < 3 {
            // Reset
            self.repetition = 0;
            self.interval = 1.0;
        } else {
            self.repetition += 1;
            self.interval = match self.repetition {
                1 => 1.0,
                2 => 6.0,
                _ => self.interval * self.easiness,
            };
        }
        self.interval
    }

    /// Return the next scheduled interval.
    pub fn next_interval(&self) -> f64 {
        self.interval
    }
}

impl Default for SpacedRepetition {
    fn default() -> Self {
        Self::new()
    }
}

/// Predicts recall probability at a given future time.
#[derive(Debug, Clone)]
pub struct RetentionPredictor {
    /// Current stability.
    pub stability: f64,
}

impl RetentionPredictor {
    /// Create a predictor with the given stability.
    pub fn new(stability: f64) -> Self {
        Self { stability }
    }

    /// Predict recall probability at time `t`.
    pub fn predict(&self, t: f64) -> f64 {
        ForgettingCurve::retrievability(t, self.stability)
    }

    /// Predict when recall drops below `threshold`.
    pub fn time_below_threshold(&self, threshold: f64) -> f64 {
        ForgettingCurve::time_to_decay(self.stability, threshold)
    }
}

/// Tracks decay across a store of named memories.
#[derive(Debug, Clone)]
pub struct MemoryDecayTracker {
    /// Map of memory name → (stability, last_review_time).
    pub store: HashMap<String, (f64, f64)>,
}

impl MemoryDecayTracker {
    /// Create an empty tracker.
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    /// Add a memory with initial stability and review time 0.
    pub fn add(&mut self, name: &str, stability: f64) {
        self.store.insert(name.to_string(), (stability, 0.0));
    }

    /// Record a review at `now`; apply rehearsal factor.
    pub fn review(&mut self, name: &str, now: f64, factor: f64) {
        if let Some((s, _)) = self.store.get_mut(name) {
            *s *= factor;
            self.store.get_mut(name).unwrap().1 = now;
        }
    }

    /// Get the predicted recall for a memory at time `t`.
    pub fn recall_at(&self, name: &str, t: f64) -> Option<f64> {
        self.store.get(name).map(|&(s, last)| {
            let elapsed = t - last;
            ForgettingCurve::retrievability(elapsed, s)
        })
    }

    /// Return names of memories whose recall at time `t` is below `threshold`.
    pub fn below_threshold(&self, t: f64, threshold: f64) -> Vec<String> {
        self.store
            .iter()
            .filter(|(_, &(s, last))| {
                ForgettingCurve::retrievability(t - last, s) < threshold
            })
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Number of tracked memories.
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}

impl Default for MemoryDecayTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retrievability_at_t_zero() {
        let r = ForgettingCurve::retrievability(0.0, 1.0);
        assert!((r - 1.0).abs() < 1e-9);
    }

    #[test]
    fn retrievability_decays() {
        let r1 = ForgettingCurve::retrievability(1.0, 1.0);
        let r2 = ForgettingCurve::retrievability(2.0, 1.0);
        assert!(r1 > r2);
    }

    #[test]
    fn retrievability_zero_stability() {
        let r = ForgettingCurve::retrievability(1.0, 0.0);
        assert_eq!(r, 0.0);
    }

    #[test]
    fn time_to_decay_basic() {
        // t = -S * ln(target_r) = -1.0 * ln(0.5) = ln(2)
        let t = ForgettingCurve::time_to_decay(1.0, 0.5);
        assert!((t - 2.0_f64.ln()).abs() < 1e-9);
    }

    #[test]
    fn memory_strength_rehearse() {
        let mut ms = MemoryStrength::new(1.0);
        ms.rehearse(2.5);
        assert!((ms.stability - 2.5).abs() < 1e-9);
        assert_eq!(ms.rehearsals, 1);
    }

    #[test]
    fn memory_strength_decay() {
        let mut ms = MemoryStrength::new(2.0);
        ms.decay(0.5);
        assert!((ms.stability - 1.0).abs() < 1e-9);
    }

    #[test]
    fn sm2_first_review_easy() {
        let mut sm = SpacedRepetition::new();
        let interval = sm.review(Quality::Four);
        assert_eq!(interval, 1.0);
        // EF' = 2.5 + (0.1 - (5-4)*(0.08+(5-4)*0.02)) = 2.5 + (0.1 - 0.1) = 2.5
        assert!((sm.easiness - 2.5).abs() < 1e-9);
    }

    #[test]
    fn sm2_second_review() {
        let mut sm = SpacedRepetition::new();
        sm.review(Quality::Four);
        let interval = sm.review(Quality::Four);
        assert_eq!(interval, 6.0);
    }

    #[test]
    fn sm2_reset_on_fail() {
        let mut sm = SpacedRepetition::new();
        sm.review(Quality::Five);
        sm.review(Quality::Five);
        let interval = sm.review(Quality::Zero);
        assert_eq!(interval, 1.0);
        assert_eq!(sm.repetition, 0);
    }

    #[test]
    fn sm2_easiness_floor() {
        let mut sm = SpacedRepetition::new();
        sm.easiness = 1.31;
        sm.update_easiness(Quality::Zero);
        assert!(sm.easiness >= 1.3);
    }

    #[test]
    fn retention_predictor() {
        let rp = RetentionPredictor::new(10.0);
        let r = rp.predict(0.0);
        assert!((r - 1.0).abs() < 1e-9);
        let r2 = rp.predict(10.0);
        assert!((r2 - (-1.0_f64).exp()).abs() < 1e-9);
    }

    #[test]
    fn decay_tracker_basic() {
        let mut dt = MemoryDecayTracker::new();
        dt.add("rust", 5.0);
        dt.add("python", 2.0);
        assert_eq!(dt.len(), 2);
        let recall = dt.recall_at("rust", 0.0).unwrap();
        assert!((recall - 1.0).abs() < 1e-9);
    }

    #[test]
    fn decay_tracker_below_threshold() {
        let mut dt = MemoryDecayTracker::new();
        dt.add("weak", 0.1);
        dt.add("strong", 100.0);
        let below = dt.below_threshold(5.0, 0.5);
        assert!(below.contains(&"weak".to_string()));
        assert!(!below.contains(&"strong".to_string()));
    }

    #[test]
    fn decay_tracker_review_boost() {
        let mut dt = MemoryDecayTracker::new();
        dt.add("item", 1.0);
        dt.review("item", 5.0, 3.0);
        let recall = dt.recall_at("item", 5.0).unwrap();
        assert!((recall - 1.0).abs() < 1e-9);
    }
}
