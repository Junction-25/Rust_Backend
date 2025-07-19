use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rand::{Rng, thread_rng};

// A/B Testing Framework for Recommendation System Optimization
// Enables controlled experiments to test different recommendation strategies

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: ExperimentStatus,
    pub variants: Vec<ExperimentVariant>,
    pub traffic_allocation: HashMap<String, f64>, // variant_id -> traffic percentage
    pub target_metrics: Vec<String>,
    pub success_criteria: SuccessCriteria,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub min_sample_size: usize,
    pub confidence_level: f64,
    pub created_by: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExperimentStatus {
    Draft,
    Running,
    Paused,
    Completed,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentVariant {
    pub id: String,
    pub name: String,
    pub description: String,
    pub config: VariantConfig,
    pub is_control: bool,
    pub allocation_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantConfig {
    pub recommendation_strategy: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub feature_flags: HashMap<String, bool>,
    pub model_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriteria {
    pub primary_metric: String,
    pub minimum_effect_size: f64,
    pub statistical_power: f64,
    pub p_value_threshold: f64,
    pub max_duration_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentAssignment {
    pub user_id: i32,
    pub experiment_id: String,
    pub variant_id: String,
    pub assignment_time: u64,
    pub session_id: String,
    pub user_segment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentEvent {
    pub user_id: i32,
    pub experiment_id: String,
    pub variant_id: String,
    pub event_type: String,
    pub event_data: HashMap<String, serde_json::Value>,
    pub timestamp: u64,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExperimentResults {
    pub experiment_id: String,
    pub variant_results: HashMap<String, VariantResults>,
    pub statistical_significance: HashMap<String, StatisticalTest>,
    pub overall_winner: Option<String>,
    pub confidence_intervals: HashMap<String, ConfidenceInterval>,
    pub sample_sizes: HashMap<String, usize>,
    pub experiment_duration: u64,
    pub generated_at: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantResults {
    pub variant_id: String,
    pub metrics: HashMap<String, MetricResult>,
    pub conversion_rate: f64,
    pub average_revenue: f64,
    pub user_engagement: f64,
    pub recommendation_ctr: f64,
    pub total_users: usize,
    pub total_events: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricResult {
    pub metric_name: String,
    pub value: f64,
    pub variance: f64,
    pub sample_size: usize,
    pub confidence_interval: ConfidenceInterval,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatisticalTest {
    pub test_type: String,
    pub p_value: f64,
    pub z_score: f64,
    pub is_significant: bool,
    pub effect_size: f64,
    pub power: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfidenceInterval {
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub confidence_level: f64,
}

pub struct ABTestingFramework {
    experiments: HashMap<String, Experiment>,
    assignments: HashMap<i32, Vec<ExperimentAssignment>>, // user_id -> assignments
    events: Vec<ExperimentEvent>,
    hash_salt: String,
}

impl ABTestingFramework {
    pub fn new() -> Self {
        Self {
            experiments: HashMap::new(),
            assignments: HashMap::new(),
            events: Vec::new(),
            hash_salt: Uuid::new_v4().to_string(),
        }
    }

    // Create a new experiment
    pub fn create_experiment(&mut self, mut experiment: Experiment) -> Result<String, String> {
        // Validate experiment configuration
        self.validate_experiment(&experiment)?;
        
        experiment.id = Uuid::new_v4().to_string();
        experiment.status = ExperimentStatus::Draft;
        experiment.start_time = Self::current_timestamp();
        
        let experiment_id = experiment.id.clone();
        self.experiments.insert(experiment_id.clone(), experiment);
        
        Ok(experiment_id)
    }

    // Start an experiment
    pub fn start_experiment(&mut self, experiment_id: &str) -> Result<(), String> {
        let experiment = self.experiments.get_mut(experiment_id)
            .ok_or("Experiment not found")?;

        match experiment.status {
            ExperimentStatus::Draft => {
                experiment.status = ExperimentStatus::Running;
                experiment.start_time = Self::current_timestamp();
                Ok(())
            }
            _ => Err("Experiment can only be started from Draft status".to_string())
        }
    }

    // Stop an experiment
    pub fn stop_experiment(&mut self, experiment_id: &str) -> Result<(), String> {
        let experiment = self.experiments.get_mut(experiment_id)
            .ok_or("Experiment not found")?;

        match experiment.status {
            ExperimentStatus::Running | ExperimentStatus::Paused => {
                experiment.status = ExperimentStatus::Stopped;
                experiment.end_time = Some(Self::current_timestamp());
                Ok(())
            }
            _ => Err("Experiment can only be stopped when running or paused".to_string())
        }
    }

    // Assign user to experiment variant
    pub fn assign_user(&mut self, user_id: i32, experiment_ids: &[String], session_id: String) -> HashMap<String, String> {
        let mut assignments = HashMap::new();

        for experiment_id in experiment_ids {
            if let Some(experiment) = self.experiments.get(experiment_id) {
                if experiment.status != ExperimentStatus::Running {
                    continue;
                }

                // Check if user is already assigned
                if let Some(existing_assignment) = self.get_existing_assignment(user_id, experiment_id) {
                    assignments.insert(experiment_id.clone(), existing_assignment);
                    continue;
                }

                // Perform assignment using consistent hashing
                if let Some(variant_id) = self.deterministic_assignment(user_id, experiment) {
                    let assignment = ExperimentAssignment {
                        user_id,
                        experiment_id: experiment_id.clone(),
                        variant_id: variant_id.clone(),
                        assignment_time: Self::current_timestamp(),
                        session_id: session_id.clone(),
                        user_segment: None, // Could be enhanced with user segmentation
                    };

                    self.assignments.entry(user_id).or_insert_with(Vec::new).push(assignment);
                    assignments.insert(experiment_id.clone(), variant_id);
                }
            }
        }

        assignments
    }

    // Get existing assignment for user
    fn get_existing_assignment(&self, user_id: i32, experiment_id: &str) -> Option<String> {
        self.assignments.get(&user_id)?
            .iter()
            .find(|a| a.experiment_id == experiment_id)
            .map(|a| a.variant_id.clone())
    }

    // Deterministic assignment using consistent hashing
    fn deterministic_assignment(&self, user_id: i32, experiment: &Experiment) -> Option<String> {
        // Create hash from user_id, experiment_id, and salt
        let hash_input = format!("{}-{}-{}", user_id, experiment.id, self.hash_salt);
        let hash = self.simple_hash(&hash_input);
        let assignment_value = (hash % 10000) as f64 / 100.0; // 0-100

        // Find which variant this assignment falls into
        let mut cumulative_percentage = 0.0;
        for variant in &experiment.variants {
            cumulative_percentage += variant.allocation_percentage;
            if assignment_value <= cumulative_percentage {
                return Some(variant.id.clone());
            }
        }

        // Fallback to control variant
        experiment.variants.iter()
            .find(|v| v.is_control)
            .map(|v| v.id.clone())
    }

    // Simple hash function (in production, use a proper hash function)
    fn simple_hash(&self, input: &str) -> u32 {
        let mut hash = 5381u32;
        for byte in input.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
        }
        hash
    }

    // Record experiment event
    pub fn record_event(&mut self, event: ExperimentEvent) {
        self.events.push(event);
        
        // Keep events manageable (in production, use a proper data store)
        if self.events.len() > 100000 {
            self.events.drain(0..10000);
        }
    }

    // Analyze experiment results
    pub fn analyze_experiment(&self, experiment_id: &str) -> Result<ExperimentResults, String> {
        let experiment = self.experiments.get(experiment_id)
            .ok_or("Experiment not found")?;

        let experiment_events: Vec<&ExperimentEvent> = self.events
            .iter()
            .filter(|e| e.experiment_id == experiment_id)
            .collect();

        if experiment_events.is_empty() {
            return Err("No events found for experiment".to_string());
        }

        let mut variant_results = HashMap::new();
        let mut statistical_significance = HashMap::new();
        let mut confidence_intervals = HashMap::new();
        let mut sample_sizes = HashMap::new();

        // Analyze each variant
        for variant in &experiment.variants {
            let variant_events: Vec<&ExperimentEvent> = experiment_events
                .iter()
                .filter(|e| e.variant_id == variant.id)
                .copied()
                .collect();

            if variant_events.is_empty() {
                continue;
            }

            // Calculate metrics for this variant
            let metrics = self.calculate_variant_metrics(&variant_events);
            let total_users = self.count_unique_users(&variant_events);
            
            let result = VariantResults {
                variant_id: variant.id.clone(),
                metrics: metrics.clone(),
                conversion_rate: self.calculate_conversion_rate(&variant_events),
                average_revenue: self.calculate_average_revenue(&variant_events),
                user_engagement: self.calculate_user_engagement(&variant_events),
                recommendation_ctr: self.calculate_recommendation_ctr(&variant_events),
                total_users,
                total_events: variant_events.len(),
            };

            variant_results.insert(variant.id.clone(), result);
            sample_sizes.insert(variant.id.clone(), total_users);
        }

        // Perform statistical significance tests
        if let Some(control_variant) = experiment.variants.iter().find(|v| v.is_control) {
            for variant in experiment.variants.iter().filter(|v| !v.is_control) {
                if let (Some(control_results), Some(test_results)) = 
                    (variant_results.get(&control_variant.id), variant_results.get(&variant.id)) {
                    
                    let stat_test = self.perform_statistical_test(control_results, test_results, &experiment.success_criteria);
                    statistical_significance.insert(variant.id.clone(), stat_test);
                }
            }
        }

        // Calculate confidence intervals
        for (variant_id, results) in &variant_results {
            for (metric_name, metric_result) in &results.metrics {
                confidence_intervals.insert(
                    format!("{}_{}", variant_id, metric_name),
                    metric_result.confidence_interval.clone()
                );
            }
        }

        // Determine overall winner
        let overall_winner = self.determine_winner(&variant_results, &statistical_significance, experiment);

        let experiment_duration = experiment.end_time.unwrap_or(Self::current_timestamp()) - experiment.start_time;

        Ok(ExperimentResults {
            experiment_id: experiment_id.to_string(),
            variant_results,
            statistical_significance,
            overall_winner,
            confidence_intervals,
            sample_sizes,
            experiment_duration,
            generated_at: Self::current_timestamp(),
        })
    }

    // Calculate metrics for a variant
    fn calculate_variant_metrics(&self, events: &[&ExperimentEvent]) -> HashMap<String, MetricResult> {
        let mut metrics = HashMap::new();

        // Conversion rate metric
        let total_users = self.count_unique_users(events);
        let conversions = events.iter()
            .filter(|e| e.event_type == "conversion")
            .count();
        
        let conversion_rate = if total_users > 0 {
            conversions as f64 / total_users as f64
        } else {
            0.0
        };

        let conversion_variance = if total_users > 1 {
            conversion_rate * (1.0 - conversion_rate) / total_users as f64
        } else {
            0.0
        };

        metrics.insert("conversion_rate".to_string(), MetricResult {
            metric_name: "conversion_rate".to_string(),
            value: conversion_rate,
            variance: conversion_variance,
            sample_size: total_users,
            confidence_interval: self.calculate_confidence_interval(conversion_rate, conversion_variance, total_users, 0.95),
        });

        // Click-through rate metric
        let impressions = events.iter()
            .filter(|e| e.event_type == "impression")
            .count();
        let clicks = events.iter()
            .filter(|e| e.event_type == "click")
            .count();

        let ctr = if impressions > 0 {
            clicks as f64 / impressions as f64
        } else {
            0.0
        };

        let ctr_variance = if impressions > 1 {
            ctr * (1.0 - ctr) / impressions as f64
        } else {
            0.0
        };

        metrics.insert("ctr".to_string(), MetricResult {
            metric_name: "ctr".to_string(),
            value: ctr,
            variance: ctr_variance,
            sample_size: impressions,
            confidence_interval: self.calculate_confidence_interval(ctr, ctr_variance, impressions, 0.95),
        });

        // Revenue per user metric
        let total_revenue: f64 = events.iter()
            .filter(|e| e.event_type == "revenue")
            .filter_map(|e| e.event_data.get("amount")?.as_f64())
            .sum();

        let revenue_per_user = if total_users > 0 {
            total_revenue / total_users as f64
        } else {
            0.0
        };

        // Simplified variance calculation for revenue
        let revenue_variance = if total_users > 1 {
            revenue_per_user / total_users as f64 // Simplified
        } else {
            0.0
        };

        metrics.insert("revenue_per_user".to_string(), MetricResult {
            metric_name: "revenue_per_user".to_string(),
            value: revenue_per_user,
            variance: revenue_variance,
            sample_size: total_users,
            confidence_interval: self.calculate_confidence_interval(revenue_per_user, revenue_variance, total_users, 0.95),
        });

        metrics
    }

    // Calculate confidence interval
    fn calculate_confidence_interval(&self, mean: f64, variance: f64, sample_size: usize, confidence_level: f64) -> ConfidenceInterval {
        if sample_size == 0 {
            return ConfidenceInterval {
                lower_bound: mean,
                upper_bound: mean,
                confidence_level,
            };
        }

        let standard_error = (variance / sample_size as f64).sqrt();
        let z_score = match confidence_level {
            0.90 => 1.645,
            0.95 => 1.96,
            0.99 => 2.576,
            _ => 1.96, // Default to 95%
        };

        let margin_of_error = z_score * standard_error;

        ConfidenceInterval {
            lower_bound: mean - margin_of_error,
            upper_bound: mean + margin_of_error,
            confidence_level,
        }
    }

    // Perform statistical significance test
    fn perform_statistical_test(&self, control: &VariantResults, test: &VariantResults, criteria: &SuccessCriteria) -> StatisticalTest {
        // Get the primary metric results
        let control_metric = control.metrics.get(&criteria.primary_metric);
        let test_metric = test.metrics.get(&criteria.primary_metric);

        if let (Some(control_result), Some(test_result)) = (control_metric, test_metric) {
            // Two-proportion z-test for conversion rates
            let p1 = control_result.value;
            let p2 = test_result.value;
            let n1 = control_result.sample_size as f64;
            let n2 = test_result.sample_size as f64;

            if n1 <= 0.0 || n2 <= 0.0 {
                return StatisticalTest {
                    test_type: "two_proportion_z_test".to_string(),
                    p_value: 1.0,
                    z_score: 0.0,
                    is_significant: false,
                    effect_size: 0.0,
                    power: 0.0,
                };
            }

            // Pooled proportion
            let p_pooled = ((p1 * n1) + (p2 * n2)) / (n1 + n2);
            
            // Standard error
            let se = (p_pooled * (1.0 - p_pooled) * ((1.0 / n1) + (1.0 / n2))).sqrt();
            
            // Z-score
            let z_score = if se > 0.0 {
                (p2 - p1) / se
            } else {
                0.0
            };

            // P-value (two-tailed)
            let p_value = 2.0 * (1.0 - self.standard_normal_cdf(z_score.abs()));

            // Effect size (Cohen's h for proportions)
            let effect_size = 2.0 * ((p2.sqrt()) - (p1.sqrt()));

            // Statistical power (simplified calculation)
            let power = if p_value < criteria.p_value_threshold {
                0.8 // Assume 80% power when significant
            } else {
                0.2 // Low power when not significant
            };

            StatisticalTest {
                test_type: "two_proportion_z_test".to_string(),
                p_value,
                z_score,
                is_significant: p_value < criteria.p_value_threshold && effect_size.abs() >= criteria.minimum_effect_size,
                effect_size,
                power,
            }
        } else {
            StatisticalTest {
                test_type: "unknown".to_string(),
                p_value: 1.0,
                z_score: 0.0,
                is_significant: false,
                effect_size: 0.0,
                power: 0.0,
            }
        }
    }

    // Approximate standard normal CDF
    fn standard_normal_cdf(&self, x: f64) -> f64 {
        0.5 * (1.0 + self.erf(x / 2_f64.sqrt()))
    }

    // Approximate error function
    fn erf(&self, x: f64) -> f64 {
        // Abramowitz and Stegun approximation
        let a1 = 0.254829592;
        let a2 = -0.284496736;
        let a3 = 1.421413741;
        let a4 = -1.453152027;
        let a5 = 1.061405429;
        let p = 0.3275911;

        let sign = if x >= 0.0 { 1.0 } else { -1.0 };
        let x = x.abs();

        let t = 1.0 / (1.0 + p * x);
        let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

        sign * y
    }

    // Determine the winner of the experiment
    fn determine_winner(&self, variant_results: &HashMap<String, VariantResults>, 
                       statistical_tests: &HashMap<String, StatisticalTest>, 
                       experiment: &Experiment) -> Option<String> {
        
        let primary_metric = &experiment.success_criteria.primary_metric;
        
        // Find the variant with the best performance that is statistically significant
        let mut best_variant: Option<String> = None;
        let mut best_value = f64::NEG_INFINITY;

        for (variant_id, results) in variant_results {
            if let Some(metric_result) = results.metrics.get(primary_metric) {
                // Check if this variant is significantly better than control
                let is_significant = statistical_tests.get(variant_id)
                    .map(|test| test.is_significant)
                    .unwrap_or(false);

                if is_significant && metric_result.value > best_value {
                    best_value = metric_result.value;
                    best_variant = Some(variant_id.clone());
                }
            }
        }

        best_variant
    }

    // Helper methods
    fn count_unique_users(&self, events: &[&ExperimentEvent]) -> usize {
        let mut unique_users = std::collections::HashSet::new();
        for event in events {
            unique_users.insert(event.user_id);
        }
        unique_users.len()
    }

    fn calculate_conversion_rate(&self, events: &[&ExperimentEvent]) -> f64 {
        let total_users = self.count_unique_users(events);
        let conversions = events.iter()
            .filter(|e| e.event_type == "conversion")
            .count();
        
        if total_users > 0 {
            conversions as f64 / total_users as f64
        } else {
            0.0
        }
    }

    fn calculate_average_revenue(&self, events: &[&ExperimentEvent]) -> f64 {
        let total_users = self.count_unique_users(events);
        let total_revenue: f64 = events.iter()
            .filter(|e| e.event_type == "revenue")
            .filter_map(|e| e.event_data.get("amount")?.as_f64())
            .sum();

        if total_users > 0 {
            total_revenue / total_users as f64
        } else {
            0.0
        }
    }

    fn calculate_user_engagement(&self, events: &[&ExperimentEvent]) -> f64 {
        let total_users = self.count_unique_users(events);
        let engagement_events = events.iter()
            .filter(|e| matches!(e.event_type.as_str(), "click" | "view" | "save" | "share"))
            .count();

        if total_users > 0 {
            engagement_events as f64 / total_users as f64
        } else {
            0.0
        }
    }

    fn calculate_recommendation_ctr(&self, events: &[&ExperimentEvent]) -> f64 {
        let impressions = events.iter()
            .filter(|e| e.event_type == "impression")
            .count();
        let clicks = events.iter()
            .filter(|e| e.event_type == "click")
            .count();

        if impressions > 0 {
            clicks as f64 / impressions as f64
        } else {
            0.0
        }
    }

    // Validate experiment configuration
    fn validate_experiment(&self, experiment: &Experiment) -> Result<(), String> {
        if experiment.variants.is_empty() {
            return Err("Experiment must have at least one variant".to_string());
        }

        let total_allocation: f64 = experiment.variants.iter()
            .map(|v| v.allocation_percentage)
            .sum();

        if (total_allocation - 100.0).abs() > 0.01 {
            return Err("Variant allocations must sum to 100%".to_string());
        }

        let control_count = experiment.variants.iter()
            .filter(|v| v.is_control)
            .count();

        if control_count != 1 {
            return Err("Experiment must have exactly one control variant".to_string());
        }

        Ok(())
    }

    // Get experiment statistics
    pub fn get_experiment_statistics(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        stats.insert("total_experiments".to_string(), 
                    serde_json::Value::Number(serde_json::Number::from(self.experiments.len())));
        
        let running_count = self.experiments.values()
            .filter(|e| matches!(e.status, ExperimentStatus::Running))
            .count();
        
        stats.insert("running_experiments".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(running_count)));
        
        stats.insert("total_events".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(self.events.len())));
        
        let total_assignments = self.assignments.values()
            .map(|assignments| assignments.len())
            .sum::<usize>();
        
        stats.insert("total_assignments".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(total_assignments)));

        stats
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_experiment() {
        let mut framework = ABTestingFramework::new();
        
        let experiment = Experiment {
            id: "".to_string(), // Will be set by create_experiment
            name: "Test Experiment".to_string(),
            description: "Testing recommendation algorithm".to_string(),
            status: ExperimentStatus::Draft,
            variants: vec![
                ExperimentVariant {
                    id: "control".to_string(),
                    name: "Control".to_string(),
                    description: "Current algorithm".to_string(),
                    config: VariantConfig {
                        recommendation_strategy: "collaborative_filtering".to_string(),
                        parameters: HashMap::new(),
                        feature_flags: HashMap::new(),
                        model_version: None,
                    },
                    is_control: true,
                    allocation_percentage: 50.0,
                },
                ExperimentVariant {
                    id: "test".to_string(),
                    name: "Test Variant".to_string(),
                    description: "New ML algorithm".to_string(),
                    config: VariantConfig {
                        recommendation_strategy: "deep_learning".to_string(),
                        parameters: HashMap::new(),
                        feature_flags: HashMap::new(),
                        model_version: Some("v2.0".to_string()),
                    },
                    is_control: false,
                    allocation_percentage: 50.0,
                },
            ],
            traffic_allocation: HashMap::new(),
            target_metrics: vec!["conversion_rate".to_string()],
            success_criteria: SuccessCriteria {
                primary_metric: "conversion_rate".to_string(),
                minimum_effect_size: 0.05,
                statistical_power: 0.8,
                p_value_threshold: 0.05,
                max_duration_days: 30,
            },
            start_time: 0,
            end_time: None,
            min_sample_size: 1000,
            confidence_level: 0.95,
            created_by: "test_user".to_string(),
            tags: vec!["recommendation".to_string()],
        };

        let result = framework.create_experiment(experiment);
        assert!(result.is_ok());
        assert_eq!(framework.experiments.len(), 1);
    }

    #[test]
    fn test_user_assignment() {
        let mut framework = ABTestingFramework::new();
        
        // Create experiment first
        let experiment = Experiment {
            id: "test-exp".to_string(),
            name: "Test Experiment".to_string(),
            description: "Testing".to_string(),
            status: ExperimentStatus::Running,
            variants: vec![
                ExperimentVariant {
                    id: "control".to_string(),
                    name: "Control".to_string(),
                    description: "Control".to_string(),
                    config: VariantConfig {
                        recommendation_strategy: "current".to_string(),
                        parameters: HashMap::new(),
                        feature_flags: HashMap::new(),
                        model_version: None,
                    },
                    is_control: true,
                    allocation_percentage: 50.0,
                },
                ExperimentVariant {
                    id: "test".to_string(),
                    name: "Test".to_string(),
                    description: "Test".to_string(),
                    config: VariantConfig {
                        recommendation_strategy: "new".to_string(),
                        parameters: HashMap::new(),
                        feature_flags: HashMap::new(),
                        model_version: None,
                    },
                    is_control: false,
                    allocation_percentage: 50.0,
                },
            ],
            traffic_allocation: HashMap::new(),
            target_metrics: vec!["conversion_rate".to_string()],
            success_criteria: SuccessCriteria {
                primary_metric: "conversion_rate".to_string(),
                minimum_effect_size: 0.05,
                statistical_power: 0.8,
                p_value_threshold: 0.05,
                max_duration_days: 30,
            },
            start_time: ABTestingFramework::current_timestamp(),
            end_time: None,
            min_sample_size: 100,
            confidence_level: 0.95,
            created_by: "test".to_string(),
            tags: vec![],
        };

        framework.experiments.insert("test-exp".to_string(), experiment);

        // Test user assignment
        let assignments = framework.assign_user(123, &["test-exp".to_string()], "session1".to_string());
        assert!(assignments.contains_key("test-exp"));
        
        // Test consistent assignment
        let assignments2 = framework.assign_user(123, &["test-exp".to_string()], "session2".to_string());
        assert_eq!(assignments.get("test-exp"), assignments2.get("test-exp"));
    }

    #[test]
    fn test_statistical_test() {
        let framework = ABTestingFramework::new();
        
        let control_results = VariantResults {
            variant_id: "control".to_string(),
            metrics: {
                let mut map = HashMap::new();
                map.insert("conversion_rate".to_string(), MetricResult {
                    metric_name: "conversion_rate".to_string(),
                    value: 0.10, // 10% conversion rate
                    variance: 0.09, // p(1-p)
                    sample_size: 1000,
                    confidence_interval: ConfidenceInterval {
                        lower_bound: 0.08,
                        upper_bound: 0.12,
                        confidence_level: 0.95,
                    },
                });
                map
            },
            conversion_rate: 0.10,
            average_revenue: 50.0,
            user_engagement: 2.5,
            recommendation_ctr: 0.15,
            total_users: 1000,
            total_events: 5000,
        };

        let test_results = VariantResults {
            variant_id: "test".to_string(),
            metrics: {
                let mut map = HashMap::new();
                map.insert("conversion_rate".to_string(), MetricResult {
                    metric_name: "conversion_rate".to_string(),
                    value: 0.12, // 12% conversion rate
                    variance: 0.1056, // p(1-p)
                    sample_size: 1000,
                    confidence_interval: ConfidenceInterval {
                        lower_bound: 0.10,
                        upper_bound: 0.14,
                        confidence_level: 0.95,
                    },
                });
                map
            },
            conversion_rate: 0.12,
            average_revenue: 60.0,
            user_engagement: 3.0,
            recommendation_ctr: 0.18,
            total_users: 1000,
            total_events: 6000,
        };

        let criteria = SuccessCriteria {
            primary_metric: "conversion_rate".to_string(),
            minimum_effect_size: 0.01,
            statistical_power: 0.8,
            p_value_threshold: 0.05,
            max_duration_days: 30,
        };

        let stat_test = framework.perform_statistical_test(&control_results, &test_results, &criteria);
        
        assert_eq!(stat_test.test_type, "two_proportion_z_test");
        assert!(stat_test.z_score > 0.0); // Test variant should have positive z-score
        assert!(stat_test.p_value < 1.0);
    }
}
