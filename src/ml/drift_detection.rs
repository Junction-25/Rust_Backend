use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Timelike, Datelike};

// Concept Drift Detection System for Real-time Model Adaptation
// Detects when the underlying data distribution changes and triggers model updates

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftSignal {
    pub drift_type: DriftType,
    pub severity: DriftSeverity,
    pub detection_time: u64,
    pub affected_segments: Vec<String>,
    pub confidence: f64,
    pub recommended_action: RecommendedAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriftType {
    ConceptDrift,      // Changes in user preferences
    DataDrift,         // Changes in feature distributions
    PerformanceDrift,  // Model performance degradation
    SeasonalDrift,     // Seasonal/temporal changes
    PopulationDrift,   // Changes in user demographics
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DriftSeverity {
    Low,      // Minor changes, continue monitoring
    Medium,   // Noticeable changes, adapt learning rates
    High,     // Significant changes, retrain components
    Critical, // Major changes, full model update needed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendedAction {
    Monitor,
    IncreaseLearningRate,
    AdaptFeatures,
    RetrainModel,
    RollbackModel,
}

pub struct ConceptDriftDetector {
    // Performance tracking
    performance_window: VecDeque<PerformanceSnapshot>,
    baseline_performance: f64,
    window_size: usize,
    
    // Statistical tests
    ks_test_threshold: f64,
    chi_square_threshold: f64,
    
    // Drift detection parameters
    drift_threshold: f64,
    warning_threshold: f64,
    min_samples: usize,
    
    // Feature distribution tracking
    feature_distributions: std::collections::HashMap<String, FeatureDistribution>,
    
    // DDM (Drift Detection Method) parameters
    ddm_warning_level: f64,
    ddm_drift_level: f64,
    
    // ADWIN (Adaptive Windowing) for performance monitoring
    adwin_window: VecDeque<f64>,
    adwin_delta: f64,
}

#[derive(Debug, Clone)]
struct PerformanceSnapshot {
    timestamp: u64,
    accuracy: f64,
    precision: f64,
    recall: f64,
    f1_score: f64,
    user_satisfaction: f64,
    response_time: f64,
    recommendation_diversity: f64,
}

#[derive(Debug, Clone)]
struct FeatureDistribution {
    mean: f64,
    variance: f64,
    min_value: f64,
    max_value: f64,
    histogram: Vec<f64>,
    sample_count: usize,
    last_update: u64,
}

impl ConceptDriftDetector {
    pub fn new() -> Self {
        Self {
            performance_window: VecDeque::new(),
            baseline_performance: 0.0,
            window_size: 100,
            ks_test_threshold: 0.05,
            chi_square_threshold: 0.05,
            drift_threshold: 0.1,
            warning_threshold: 0.05,
            min_samples: 30,
            feature_distributions: std::collections::HashMap::new(),
            ddm_warning_level: 2.0,
            ddm_drift_level: 3.0,
            adwin_window: VecDeque::new(),
            adwin_delta: 0.002,
        }
    }

    // Main drift detection method
    pub async fn detect_drift(&mut self, performance_data: PerformanceSnapshot, feature_data: std::collections::HashMap<String, f64>) -> Option<DriftSignal> {
        // Update performance tracking
        self.update_performance_tracking(performance_data.clone());
        
        // Update feature distributions
        for (feature_name, value) in &feature_data {
            self.update_feature_distribution(feature_name.clone(), *value);
        }

        // Check for different types of drift
        let mut detected_drifts = Vec::new();

        // 1. Performance-based drift detection (DDM)
        if let Some(drift) = self.detect_performance_drift(&performance_data) {
            detected_drifts.push(drift);
        }

        // 2. Feature distribution drift detection (KS test)
        if let Some(drift) = self.detect_feature_drift() {
            detected_drifts.push(drift);
        }

        // 3. Concept drift detection (statistical tests)
        if let Some(drift) = self.detect_concept_drift(&performance_data) {
            detected_drifts.push(drift);
        }

        // 4. Seasonal drift detection
        if let Some(drift) = self.detect_seasonal_drift(&performance_data) {
            detected_drifts.push(drift);
        }

        // Return the most severe drift detected
        detected_drifts.into_iter().max_by_key(|d| match d.severity {
            DriftSeverity::Critical => 4,
            DriftSeverity::High => 3,
            DriftSeverity::Medium => 2,
            DriftSeverity::Low => 1,
        })
    }

    // DDM (Drift Detection Method) for performance monitoring
    fn detect_performance_drift(&mut self, performance: &PerformanceSnapshot) -> Option<DriftSignal> {
        if self.performance_window.len() < self.min_samples {
            return None;
        }

        let recent_performance: Vec<f64> = self.performance_window
            .iter()
            .rev()
            .take(30)
            .map(|p| p.accuracy)
            .collect();

        let mean = recent_performance.iter().sum::<f64>() / recent_performance.len() as f64;
        let variance = recent_performance.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / recent_performance.len() as f64;
        let std_dev = variance.sqrt();

        // DDM statistical test
        let error_rate = 1.0 - mean;
        let p_i = error_rate;
        let s_i = (p_i * (1.0 - p_i) / recent_performance.len() as f64).sqrt();

        let warning_threshold = p_i + self.ddm_warning_level * s_i;
        let drift_threshold = p_i + self.ddm_drift_level * s_i;

        let current_error = 1.0 - performance.accuracy;

        if current_error > drift_threshold {
            Some(DriftSignal {
                drift_type: DriftType::PerformanceDrift,
                severity: DriftSeverity::High,
                detection_time: Self::current_timestamp(),
                affected_segments: vec!["global_performance".to_string()],
                confidence: 0.9,
                recommended_action: RecommendedAction::RetrainModel,
            })
        } else if current_error > warning_threshold {
            Some(DriftSignal {
                drift_type: DriftType::PerformanceDrift,
                severity: DriftSeverity::Medium,
                detection_time: Self::current_timestamp(),
                affected_segments: vec!["global_performance".to_string()],
                confidence: 0.7,
                recommended_action: RecommendedAction::IncreaseLearningRate,
            })
        } else {
            None
        }
    }

    // Kolmogorov-Smirnov test for feature distribution changes
    fn detect_feature_drift(&self) -> Option<DriftSignal> {
        let mut drift_features = Vec::new();

        for (feature_name, distribution) in &self.feature_distributions {
            if distribution.sample_count < self.min_samples {
                continue;
            }

            // Simplified KS test - in production, use proper statistical library
            let ks_statistic = self.calculate_ks_statistic(distribution);
            
            if ks_statistic > self.ks_test_threshold {
                drift_features.push(feature_name.clone());
            }
        }

        if !drift_features.is_empty() {
            let severity = if drift_features.len() > 5 {
                DriftSeverity::High
            } else if drift_features.len() > 2 {
                DriftSeverity::Medium
            } else {
                DriftSeverity::Low
            };

            Some(DriftSignal {
                drift_type: DriftType::DataDrift,
                severity: severity.clone(),
                detection_time: Self::current_timestamp(),
                affected_segments: drift_features,
                confidence: 0.8,
                recommended_action: match severity {
                    DriftSeverity::High => RecommendedAction::AdaptFeatures,
                    DriftSeverity::Medium => RecommendedAction::IncreaseLearningRate,
                    _ => RecommendedAction::Monitor,
                },
            })
        } else {
            None
        }
    }

    // Concept drift detection using performance degradation patterns
    fn detect_concept_drift(&self, performance: &PerformanceSnapshot) -> Option<DriftSignal> {
        if self.performance_window.len() < self.window_size {
            return None;
        }

        // Calculate trend in user satisfaction and recommendation quality
        let recent_satisfaction: Vec<f64> = self.performance_window
            .iter()
            .rev()
            .take(self.window_size / 2)
            .map(|p| p.user_satisfaction)
            .collect();

        let older_satisfaction: Vec<f64> = self.performance_window
            .iter()
            .rev()
            .skip(self.window_size / 2)
            .take(self.window_size / 2)
            .map(|p| p.user_satisfaction)
            .collect();

        if recent_satisfaction.is_empty() || older_satisfaction.is_empty() {
            return None;
        }

        let recent_avg = recent_satisfaction.iter().sum::<f64>() / recent_satisfaction.len() as f64;
        let older_avg = older_satisfaction.iter().sum::<f64>() / older_satisfaction.len() as f64;

        let satisfaction_decline = older_avg - recent_avg;

        if satisfaction_decline > self.drift_threshold {
            Some(DriftSignal {
                drift_type: DriftType::ConceptDrift,
                severity: if satisfaction_decline > 0.2 {
                    DriftSeverity::Critical
                } else if satisfaction_decline > 0.1 {
                    DriftSeverity::High
                } else {
                    DriftSeverity::Medium
                },
                detection_time: Self::current_timestamp(),
                affected_segments: vec!["user_preferences".to_string()],
                confidence: 0.85,
                recommended_action: RecommendedAction::AdaptFeatures,
            })
        } else {
            None
        }
    }

    // Seasonal drift detection based on temporal patterns
    fn detect_seasonal_drift(&self, performance: &PerformanceSnapshot) -> Option<DriftSignal> {
        // Simple implementation - in production, use more sophisticated time series analysis
        let current_hour = chrono::Utc::now().hour();
        let current_day = chrono::Utc::now().weekday().num_days_from_monday();

        // Check if performance varies significantly by time patterns
        let time_based_performance = self.get_performance_by_time_pattern(current_hour, current_day);
        
        if let Some(expected_performance) = time_based_performance {
            let performance_gap = (expected_performance - performance.accuracy).abs();
            
            if performance_gap > 0.15 {
                Some(DriftSignal {
                    drift_type: DriftType::SeasonalDrift,
                    severity: DriftSeverity::Medium,
                    detection_time: Self::current_timestamp(),
                    affected_segments: vec![format!("hour_{}_day_{}", current_hour, current_day)],
                    confidence: 0.6,
                    recommended_action: RecommendedAction::AdaptFeatures,
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    // Update performance tracking window
    fn update_performance_tracking(&mut self, performance: PerformanceSnapshot) {
        self.performance_window.push_back(performance);
        
        if self.performance_window.len() > self.window_size {
            self.performance_window.pop_front();
        }

        // Update baseline performance
        if self.performance_window.len() >= self.min_samples {
            self.baseline_performance = self.performance_window
                .iter()
                .map(|p| p.accuracy)
                .sum::<f64>() / self.performance_window.len() as f64;
        }
    }

    // Update feature distribution statistics
    fn update_feature_distribution(&mut self, feature_name: String, value: f64) {
        let distribution = self.feature_distributions.entry(feature_name).or_insert_with(|| {
            FeatureDistribution {
                mean: 0.0,
                variance: 0.0,
                min_value: f64::INFINITY,
                max_value: f64::NEG_INFINITY,
                histogram: vec![0.0; 10], // 10 bins
                sample_count: 0,
                last_update: Self::current_timestamp(),
            }
        });

        // Update statistics using online algorithms
        distribution.sample_count += 1;
        let n = distribution.sample_count as f64;
        
        // Online mean calculation
        let delta = value - distribution.mean;
        distribution.mean += delta / n;
        
        // Online variance calculation (Welford's algorithm)
        let delta2 = value - distribution.mean;
        distribution.variance += delta * delta2;

        // Update min/max
        distribution.min_value = distribution.min_value.min(value);
        distribution.max_value = distribution.max_value.max(value);

        // Update histogram
        let min_val = distribution.min_value;
        let max_val = distribution.max_value;
        if max_val > min_val {
            let bin_size = (max_val - min_val) / distribution.histogram.len() as f64;
            let bin_index = ((value - min_val) / bin_size).floor() as usize;
            let bin_index = bin_index.min(distribution.histogram.len() - 1);
            distribution.histogram[bin_index] += 1.0;
        }
        
        distribution.last_update = Self::current_timestamp();
    }

    // Calculate KS statistic (simplified version)
    fn calculate_ks_statistic(&self, distribution: &FeatureDistribution) -> f64 {
        // Simplified KS test - compares current distribution with expected uniform distribution
        // In production, use proper statistical libraries
        
        let total_samples = distribution.histogram.iter().sum::<f64>();
        if total_samples == 0.0 {
            return 0.0;
        }

        let expected_per_bin = total_samples / distribution.histogram.len() as f64;
        
        let mut max_diff: f64 = 0.0;
        let mut cumulative_observed = 0.0;
        let mut cumulative_expected = 0.0;

        for &observed in &distribution.histogram {
            cumulative_observed += observed / total_samples;
            cumulative_expected += expected_per_bin / total_samples;
            
            let diff = (cumulative_observed - cumulative_expected).abs();
            max_diff = max_diff.max(diff);
        }

        max_diff
    }

    // Get expected performance based on time patterns
    fn get_performance_by_time_pattern(&self, hour: u32, day: u32) -> Option<f64> {
        // Simplified implementation - in production, use time series analysis
        let time_filtered_performance: Vec<f64> = self.performance_window
            .iter()
            .filter(|p| {
                let p_hour = chrono::DateTime::from_timestamp(p.timestamp as i64, 0)
                    .map(|dt| dt.hour())
                    .unwrap_or(0);
                let p_day = chrono::DateTime::from_timestamp(p.timestamp as i64, 0)
                    .map(|dt| dt.weekday().num_days_from_monday())
                    .unwrap_or(0);
                
                (p_hour as i32 - hour as i32).abs() <= 1 && p_day == day
            })
            .map(|p| p.accuracy)
            .collect();

        if time_filtered_performance.len() >= 5 {
            Some(time_filtered_performance.iter().sum::<f64>() / time_filtered_performance.len() as f64)
        } else {
            None
        }
    }

    // Get drift detection statistics
    pub fn get_drift_statistics(&self) -> DriftStatistics {
        let current_performance = self.performance_window.back()
            .map(|p| p.accuracy)
            .unwrap_or(0.0);

        DriftStatistics {
            baseline_performance: self.baseline_performance,
            current_performance,
            performance_trend: self.calculate_performance_trend(),
            monitored_features: self.feature_distributions.len(),
            samples_processed: self.performance_window.len(),
            last_update: Self::current_timestamp(),
        }
    }

    fn calculate_performance_trend(&self) -> f64 {
        if self.performance_window.len() < 10 {
            return 0.0;
        }

        let recent: f64 = self.performance_window
            .iter()
            .rev()
            .take(5)
            .map(|p| p.accuracy)
            .sum::<f64>() / 5.0;

        let older: f64 = self.performance_window
            .iter()
            .rev()
            .skip(5)
            .take(5)
            .map(|p| p.accuracy)
            .sum::<f64>() / 5.0;

        recent - older
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

#[derive(Debug, Serialize)]
pub struct DriftStatistics {
    pub baseline_performance: f64,
    pub current_performance: f64,
    pub performance_trend: f64,
    pub monitored_features: usize,
    pub samples_processed: usize,
    pub last_update: u64,
}

// Adaptive windowing for performance monitoring
pub struct AdaptiveWindow {
    window: VecDeque<f64>,
    delta: f64,
    min_window_size: usize,
    total_sum: f64,
    total_sum_squares: f64,
}

impl AdaptiveWindow {
    pub fn new(delta: f64, min_window_size: usize) -> Self {
        Self {
            window: VecDeque::new(),
            delta,
            min_window_size,
            total_sum: 0.0,
            total_sum_squares: 0.0,
        }
    }

    pub fn add_element(&mut self, value: f64) -> bool {
        self.window.push_back(value);
        self.total_sum += value;
        self.total_sum_squares += value * value;
        
        self.detect_change()
    }

    fn detect_change(&mut self) -> bool {
        if self.window.len() < self.min_window_size * 2 {
            return false;
        }

        // ADWIN algorithm for detecting change points
        let n = self.window.len();
        let mean = self.total_sum / n as f64;
        
        for i in self.min_window_size..(n - self.min_window_size) {
            let n1 = i as f64;
            let n2 = (n - i) as f64;
            
            let sum1: f64 = self.window.iter().take(i).sum();
            let sum2: f64 = self.window.iter().skip(i).sum();
            
            let mean1 = sum1 / n1;
            let mean2 = sum2 / n2;
            
            let harmonic_mean = 1.0 / ((1.0 / n1) + (1.0 / n2));
            let epsilon = (2.0 * (1.0 + mean * (1.0 - mean)) * (2.0 * self.delta.ln().abs() + (2.0 * n as f64).ln()) / harmonic_mean).sqrt();
            
            if (mean1 - mean2).abs() > epsilon {
                // Change detected, remove elements from the beginning
                for _ in 0..i {
                    if let Some(removed) = self.window.pop_front() {
                        self.total_sum -= removed;
                        self.total_sum_squares -= removed * removed;
                    }
                }
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_detector_creation() {
        let detector = ConceptDriftDetector::new();
        assert_eq!(detector.window_size, 100);
        assert_eq!(detector.performance_window.len(), 0);
    }

    #[tokio::test]
    async fn test_performance_drift_detection() {
        let mut detector = ConceptDriftDetector::new();
        
        // Add some baseline performance data
        for i in 0..50 {
            let performance = PerformanceSnapshot {
                timestamp: ConceptDriftDetector::current_timestamp(),
                accuracy: 0.8,
                precision: 0.75,
                recall: 0.8,
                f1_score: 0.77,
                user_satisfaction: 0.8,
                response_time: 10.0,
                recommendation_diversity: 0.7,
            };
            detector.update_performance_tracking(performance);
        }

        // Add performance drop
        let poor_performance = PerformanceSnapshot {
            timestamp: ConceptDriftDetector::current_timestamp(),
            accuracy: 0.5, // Significant drop
            precision: 0.4,
            recall: 0.5,
            f1_score: 0.45,
            user_satisfaction: 0.3,
            response_time: 15.0,
            recommendation_diversity: 0.4,
        };

        let result = detector.detect_drift(poor_performance, std::collections::HashMap::new()).await;
        assert!(result.is_some());
        
        let drift = result.unwrap();
        assert_eq!(drift.drift_type, DriftType::PerformanceDrift);
    }

    #[test]
    fn test_adaptive_window() {
        let mut window = AdaptiveWindow::new(0.002, 10);
        
        // Add stable data
        for _ in 0..30 {
            assert!(!window.add_element(0.8));
        }
        
        // Add changed data
        for _ in 0..20 {
            if window.add_element(0.3) {
                // Change detected
                break;
            }
        }
    }
}
