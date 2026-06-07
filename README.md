# forgetting-curve

> **Memories decay. Spaced repetition fights back.**

[![crates.io](https://img.shields.io/crates/v/forgetting-curve.svg)](https://crates.io/crates/forgetting-curve)
[![docs.rs](https://docs.rs/forgetting-curve/badge.svg)](https://docs.rs/forgetting-curve)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A Rust library implementing the Ebbinghaus forgetting curve with SM-2 spaced repetition scheduling. Models memory decay as R = e^(−t/S), schedules optimal review times, and tracks memory stability over multiple rehearsals. Gives agents a biologically-grounded memory management system.

---

## Table of Contents

- [What is the Forgetting Curve?](#what-is-the-forgetting-curve)
- [Why Does This Matter?](#why-does-this-matter)
- [Architecture](#architecture)
- [Quick Start](#quick-start)
- [API Reference](#api-reference)
- [Mathematical Background](#mathematical-background)
- [Installation](#installation)
- [Related Crates](#related-crates)
- [License](#license)

---

## What is the Forgetting Curve?

In 1885, Hermann Ebbinghaus conducted the first scientific study of memory. He discovered that **retention follows an exponential decay curve** — without reinforcement, memories fade predictably:

```
Retention %
100 │╲
    │ ╲
    │  ╲
 80 │   ╲
    │    ╲
    │     ╲
 60 │      ╲
    │       ╲
    │        ╲
 40 │         ╲───────
    │                 ────────
 20 │                         ────────
    │                                 ────────
  0 └────────────────────────────────────────── Time
    0   1   2   3   4   5   6   7   8   9  10  days
```

The equation: **R(t) = e^(−t/S)** where R is retrievability (recall probability), t is time since last review, and S is memory stability (strength).

**Spaced repetition** exploits a key finding: each review increases stability more than the last, creating expanding intervals:

```
Review 1:  Day 0    →  S = 1.0
Review 2:  Day 1    →  S = 2.5
Review 3:  Day 3    →  S = 6.25
Review 4:  Day 8    →  S = 15.6
Review 5:  Day 21   →  S = 39.1
Review 6:  Day 55   →  S = 97.7
```

The intervals grow geometrically — hence "spaced" repetition. This library implements this math for AI agents.

## Why Does This Matter?

**For agent memory management**: Agents that never forget waste resources on stale information. Agents that forget too quickly lose important knowledge. The forgetting curve provides an optimal trade-off.

**For knowledge prioritization**: Not all memories are equally important. SM-2 assigns higher stability to well-remembered items and lower stability to poorly-remembered ones — automatically prioritizing review time.

**For cognitive modeling**: If you're building agents that model human-like cognition, the forgetting curve is fundamental. Human memory doesn't work like a database — it decays, and spaced repetition is how we fight back.

**For resource-constrained systems**: Memory is finite. Forgetting curves let agents allocate memory to what matters most, gracefully degrading older, less-practiced knowledge.

## Architecture

```
forgetting-curve
│
├── ForgettingCurve            ← Core decay function
│   ├── retrievability(t, S)       R = e^(−t/S)
│   └── time_to_decay(S, target)   When R drops below threshold
│
├── MemoryStrength             ← Memory stability tracker
│   ├── new(initial)               Start with initial stability
│   ├── rehearse(factor)           Boost: S *= factor
│   ├── decay(factor)              Reduce: S *= factor
│   └── default_initial()          1.0
│
├── Quality (enum)             ← Recall quality rating (SM-2)
│   ├── CompleteBlackout(0)        Complete failure
│   ├── Incorrect(1)               Wrong, but recognized
│   ├── IncorrectWithEffort(2)     Wrong, remembered with difficulty
│   ├── CorrectWithDifficulty(3)   Correct, but hard
│   ├── CorrectWithHesitation(4)   Correct, minor hesitation
│   └── Perfect(5)                 Perfect recall
│
├── SpacedRepetition           ← SM-2 algorithm
│   ├── new()                      Start SM-2 with defaults
│   ├── update_easiness(q)         Adjust difficulty rating
│   ├── review(q)                  Process review, return next interval
│   └── next_interval()           Days until next review
│
├── RetentionPredictor         ← Forecast memory retention
│   ├── new(stability)             Initialize with current S
│   ├── predict(t)                 R(t) = e^(−t/S)
│   └── time_below_threshold(th)   When retention drops below threshold
│
└── MemoryDecayTracker         ← Multi-item memory tracker
    ├── new()                      Empty tracker
    ├── add(name, stability)       Track a new memory item
    ├── review(name, now, factor)  Review an item at time `now`
    ├── recall_at(name, t)         Predicted retention at time t
    ├── below_threshold(t, th)     Items below retention threshold
    ├── len()                      Number of tracked items
    └── is_empty()                 No items tracked
```

## Quick Start

```rust
use forgetting_curve::{
    ForgettingCurve, MemoryStrength, Quality,
    SpacedRepetition, RetentionPredictor, MemoryDecayTracker,
};

// How well will you remember after 7 days with stability 10?
let r = ForgettingCurve::retrievability(7.0, 10.0);
println!("7-day retention: {:.1}%", r * 100.0); // ~49.7%

// When does retention drop below 50%?
let t_half = ForgettingCurve::time_to_decay(10.0, 0.5);
println!("Half-life: {:.1} days", t_half); // ~6.9 days

// Memory strength: boost through rehearsal
let mut memory = MemoryStrength::new(1.0);
memory.rehearse(2.5);  // First review: S = 2.5
memory.rehearse(2.5);  // Second review: S = 6.25
println!("Stability after 2 reviews: {:.2}", memory.stability);

// SM-2 spaced repetition
let mut sm2 = SpacedRepetition::new();
let interval = sm2.review(Quality::CorrectWithHesitation(4));
println!("Next review in {:.1} days", interval);

sm2.update_easiness(Quality::Perfect(5));
let next = sm2.next_interval();
println!("After perfect recall: {:.1} days", next);

// Retention prediction
let predictor = RetentionPredictor::new(10.0);
println!("Retention at day 1: {:.1}%", predictor.predict(1.0) * 100.0);
println!("Retention at day 5: {:.1}%", predictor.predict(5.0) * 100.0);
let critical = predictor.time_below_threshold(0.7);
println!("Drops below 70% at: {:.1} days", critical);

// Track multiple memories
let mut tracker = MemoryDecayTracker::new();
tracker.add("rust_syntax", 10.0);
tracker.add("phone_number", 2.0);
tracker.add("api_endpoint", 5.0);

// Check which memories are fading after 3 days
let fading = tracker.below_threshold(3.0, 0.5);
println!("Fading memories: {:?}", fading); // ["phone_number"]

// Review an item
tracker.review("api_endpoint", 3.0, 2.0); // Boost stability ×2
```

## API Reference

### ForgettingCurve

| Function | Signature | Description |
|----------|-----------|-------------|
| `retrievability(t, S)` | `(f64, f64) → f64` | R = e^(−t/S) — recall probability |
| `time_to_decay(S, target)` | `(f64, f64) → f64` | Time for R to reach target |

### MemoryStrength

| Method | Returns | Description |
|--------|---------|-------------|
| `new(initial)` | `Self` | Start with initial stability |
| `rehearse(factor)` | `()` | S *= factor (boost) |
| `decay(factor)` | `()` | S *= factor (reduce) |
| `default_initial()` | `f64` | 1.0 |

### Quality (SM-2 Rating)

| Variant | Value | Description |
|---------|-------|-------------|
| `CompleteBlackout` | 0 | Complete failure |
| `Incorrect` | 1 | Wrong, recognized later |
| `IncorrectWithEffort` | 2 | Wrong, remembered with difficulty |
| `CorrectWithDifficulty` | 3 | Correct, hard |
| `CorrectWithHesitation` | 4 | Correct, slight hesitation |
| `Perfect` | 5 | Perfect recall |

### SpacedRepetition

| Method | Returns | Description |
|--------|---------|-------------|
| `new()` | `Self` | Default SM-2 parameters |
| `update_easiness(q)` | `()` | Adjust E-factor |
| `review(q)` | `f64` | Process review → next interval |
| `next_interval()` | `f64` | Days until next review |

### RetentionPredictor

| Method | Returns | Description |
|--------|---------|-------------|
| `new(stability)` | `Self` | Initialize with S |
| `predict(t)` | `f64` | R(t) = e^(−t/S) |
| `time_below_threshold(th)` | `f64` | When R drops below threshold |

### MemoryDecayTracker

| Method | Returns | Description |
|--------|---------|-------------|
| `new()` | `Self` | Empty tracker |
| `add(name, stability)` | `()` | Track new item |
| `review(name, now, factor)` | `()` | Boost item at time |
| `recall_at(name, t)` | `Option<f64>` | Predicted retention |
| `below_threshold(t, th)` | `Vec<String>` | Items below threshold |
| `len()` | `usize` | Tracked items |
| `is_empty()` | `bool` | No items |

## Mathematical Background

### Ebbinghaus Forgetting Curve

The basic forgetting model:

```
R(t) = e^(−t/S)
```

Where:
- **R** = retrievability (probability of successful recall), range [0, 1]
- **t** = time elapsed since last review
- **S** = memory stability (the half-life of the memory)

The **half-life** (time for R to drop to 50%) is:

```
t_{1/2} = S × ln(2) ≈ 0.693 × S
```

### SM-2 Algorithm (Wozniak, 1985)

SM-2 schedules reviews based on recall quality q ∈ {0, 1, 2, 3, 4, 5}:

**Easiness factor** (EF): encodes item difficulty

```
EF' = EF + (0.1 − (5 − q) × (0.08 + (5 − q) × 0.02))
EF = max(1.3, EF')    // minimum difficulty
```

**Interval scheduling**:

```
I(1) = 1 day
I(2) = 6 days
I(n) = I(n−1) × EF    for n ≥ 3
```

If q < 3 (failed recall), restart from I(1) = 1.

### SuperMemo Family

SM-2 is the simplest of the SuperMemo algorithms. Later versions (SM-5, SM-8, SM-15, SM-17, SM-18) add:
- Matrix-based difficulty estimation
- Optimal factor tracking
- Retrievability-based scheduling
- Memory model integration

This library implements SM-2 as a clear, well-understood baseline.

### Memory Stability Growth

After each successful review, stability grows:

```
S_{n+1} = S_n × f(q)
```

Where f(q) is an increasing function of recall quality. In SM-2, f(q) = EF (easiness factor). In more recent models (SM-17/18):

```
S_{n+1} = S_n × (1 + e^{α × (1 − R)} × f(D, q))
```

Where R is the expected retrievability at review time, D is item difficulty, and α controls the spacing effect.

## Installation

```bash
cargo add forgetting-curve
```

Or add to your `Cargo.toml`:

```toml
[dependencies]
forgetting-curve = "0.1"
```

## Related Crates

Part of the **SuperInstance Exocortex** ecosystem:

- **[dream-cycle](https://github.com/SuperInstance/dream-cycle)** — Sleep consolidation for agent memory
- **[shadow-cathedral](https://github.com/SuperInstance/shadow-cathedral)** — 3-layer shadow rendering pipeline
- **[free-energy](https://github.com/SuperInstance/free-energy)** — Variational free energy computation
- **[cortex-bus-protocol](https://github.com/SuperInstance/cortex-bus-protocol)** — CQRS event bus for agents
- **[markov-blanket](https://github.com/SuperInstance/markov-blanket)** — Statistical boundary detection

## References

- Ebbinghaus, H. *Über das Gedächtnis* (1885)
- Wozniak, P. *Optimization of repetition spacing in the practice of learning* (1990)
- SuperMemo algorithm descriptions: supermemo.guru

## License

MIT © [SuperInstance](https://github.com/SuperInstance)

Part of the [Exocortex](https://github.com/SuperInstance/exocortex) project.
