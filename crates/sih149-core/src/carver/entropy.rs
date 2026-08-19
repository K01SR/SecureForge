//! Shannon entropy calculator and heatmap data generator.
//!
//! Calculates per-sector entropy values (0.0 to 8.0) for:
//! - Disk surface visualization (entropy heatmap in GUI)
//! - Post-wipe verification (confirming erasure success)
//! - Block classification (distinguishing encrypted vs text vs empty)
//!
//! Entropy values are exported as a compact array for frontend rendering.
