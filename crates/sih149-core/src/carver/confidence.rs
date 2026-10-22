/// Represents the confidence level of a carved file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    /// Low confidence (e.g., only header matched, no footer, no structure validation)
    Low,
    /// Medium confidence (e.g., header and entropy align, partial structure)
    Medium,
    /// High confidence (e.g., header, footer, and full structure validation passed)
    High,
    /// Absolute confidence (e.g., exact hash match or flawless strict structure)
    Absolute,
}

impl Default for Confidence {
    fn default() -> Self {
        Confidence::Low
    }
}

/// A structure to hold scoring factors that compute a final confidence.
#[derive(Debug, Default)]
pub struct ConfidenceScorer {
    pub has_header: bool,
    pub has_footer: bool,
    pub structure_valid: bool,
    pub expected_entropy_range: Option<(f64, f64)>,
    pub actual_entropy: Option<f64>,
}

impl ConfidenceScorer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate the final confidence based on current factors.
    pub fn calculate(&self) -> Confidence {
        if self.has_header && self.has_footer && self.structure_valid {
            return Confidence::High;
        }

        if self.has_header && self.structure_valid {
            return Confidence::Medium;
        }

        if self.has_header && self.has_footer {
            return Confidence::Medium;
        }

        if self.has_header {
            if let (Some(range), Some(entropy)) = (self.expected_entropy_range, self.actual_entropy) {
                if entropy >= range.0 && entropy <= range.1 {
                    return Confidence::Medium;
                }
            }
        }

        Confidence::Low
    }
}
