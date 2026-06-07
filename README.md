# forgetting-curve

> **Memories decay. Spaced repetition fights back.**

[![crates.io](https://img.shields.io/crates/v/forgetting-curve.svg)](https://crates.io/crates/forgetting-curve)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Ebbinghaus forgetting curve implementation with spaced repetition scheduling. Models memory decay as R = e^(-t/S) and schedules optimal review times using the SM-2 algorithm.

## The Science

In 1885, Hermann Ebbinghaus discovered that memory retention follows an exponential decay curve. The **forgetting curve** describes how information is lost over time when there's no attempt to retain it.

`forgetting-curve` makes this computational:
- Model decay: R(t) = e^(-t/S) where S = memory strength
- Boost strength through rehearsal
- Schedule reviews via SM-2 algorithm
- Predict retention probability at any future time

## Quick Start

```rust
use forgetting_curve::{ForgettingCurve, SpacedRepetition};

// How well will you remember after 7 days with strength 10?
let curve = ForgettingCurve::new(10.0);
let retention = curve.retention(7.0);
println!("7-day retention: {:.1}%", retention * 100.0);
```

## Part of [Exocortex](https://github.com/SuperInstance/exocortex)

The **Forgetting Cortex** pattern: memories decay naturally, and the cortex uses spaced repetition to maintain important knowledge while letting stale information fade. Inspired by the Hermes 405B design competition entry.

## License

MIT © [SuperInstance](https://github.com/SuperInstance)
