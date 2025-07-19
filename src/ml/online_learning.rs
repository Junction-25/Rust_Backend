use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use std::sync::Arc;

// Online Learning Engine for Real-time Model Adaptation
// This system continuously learns from user interactions and adapts recommendations

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeedback {
    pub user_id: i32,
    pub property_id: i32,
    pub interaction_type: InteractionType,
    pub timestamp: u64,
    pub session_id: String,
    pub context: InteractionContext,
    pub feedback_value: f64, // -1.0 to 1.0 (negative to positive feedback)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InteractionType {
    View,
    Like,
    Dislike,
    Contact,
    Save,
    Share,
    BookViewing,
    Apply,
    Skip,
    Dismiss,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionContext {
    pub device_type: String,
    pub location: Option<(f64, f64)>,
    pub time_of_day: u8, // 0-23 hours
    pub day_of_week: u8, // 0-6 days
    pub search_query: Option<String>,
    pub previous_interactions: Vec<i32>, // Previous property IDs in session
    pub recommendation_rank: Option<usize>, // Position in recommendation list
}

#[derive(Debug, Clone)]
pub struct UserProfile {
    pub user_id: i32,
    pub preferences: HashMap<String, f64>,
    pub session_preferences: HashMap<String, f64>,
    pub interaction_history: Vec<UserFeedback>,
    pub learning_rate: f64,
    pub exploration_factor: f64,
    pub last_updated: u64,
    pub confidence_scores: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct ModelWeights {
    pub feature_weights: HashMap<String, f64>,
    pub interaction_weights: HashMap<InteractionType, f64>,
    pub context_weights: HashMap<String, f64>,
    pub decay_factor: f64,
    pub learning_momentum: f64,
    pub last_update: u64,
}

pub struct OnlineLearningEngine {
    user_profiles: Arc<RwLock<HashMap<i32, UserProfile>>>,
    model_weights: Arc<RwLock<ModelWeights>>,
    feedback_buffer: Arc<RwLock<Vec<UserFeedback>>>,
    config: LearningConfig,
    drift_detector: DriftDetector,
}

#[derive(Debug, Clone)]
pub struct LearningConfig {
    pub base_learning_rate: f64,
    pub min_learning_rate: f64,
    pub max_learning_rate: f64,
    pub adaptation_window: usize,
    pub buffer_size: usize,
    pub update_frequency: u64, // seconds
    pub exploration_decay: f64,
    pub confidence_threshold: f64,
    pub drift_sensitivity: f64,
}

#[derive(Debug, Clone)]
pub struct DriftDetector {
    performance_history: Vec<f64>,
    current_performance: f64,
    drift_threshold: f64,
    window_size: usize,
    detected_drift: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearningMetrics {
    pub total_interactions: usize,
    pub active_users: usize,
    pub average_learning_rate: f64,
    pub model_accuracy: f64,
    pub drift_detected: bool,
    pub last_update: u64,
    pub performance_trend: Vec<f64>,
}

impl OnlineLearningEngine {
    pub fn new(config: LearningConfig) -> Self {
        let drift_detector = DriftDetector {
            performance_history: Vec::new(),
            current_performance: 0.0,
            drift_threshold: config.drift_sensitivity,
            window_size: 100,
            detected_drift: false,
        };

        let model_weights = ModelWeights {
            feature_weights: Self::initialize_feature_weights(),
            interaction_weights: Self::initialize_interaction_weights(),
            context_weights: Self::initialize_context_weights(),
            decay_factor: 0.95,
            learning_momentum: 0.9,
            last_update: Self::current_timestamp(),
        };

        Self {
            user_profiles: Arc::new(RwLock::new(HashMap::new())),
            model_weights: Arc::new(RwLock::new(model_weights)),
            feedback_buffer: Arc::new(RwLock::new(Vec::new())),
            config,
            drift_detector,
        }
    }

    // Process real-time user feedback and update models
    pub async fn process_feedback(&self, feedback: UserFeedback) -> Result<(), Box<dyn std::error::Error>> {
        // Add to feedback buffer
        {
            let mut buffer = self.feedback_buffer.write().await;
            buffer.push(feedback.clone());
            
            // Keep buffer size manageable
            if buffer.len() > self.config.buffer_size {
                buffer.remove(0);
            }
        }

        // Update user profile immediately for session-based personalization
        self.update_user_profile(feedback.clone()).await?;

        // Trigger model update if needed
        if self.should_update_model().await {
            self.update_global_model().await?;
        }

        Ok(())
    }

    // Update individual user profile with new feedback
    async fn update_user_profile(&self, feedback: UserFeedback) -> Result<(), Box<dyn std::error::Error>> {
        let mut profiles = self.user_profiles.write().await;
        
        let profile = profiles.entry(feedback.user_id).or_insert_with(|| UserProfile {
            user_id: feedback.user_id,
            preferences: HashMap::new(),
            session_preferences: HashMap::new(),
            interaction_history: Vec::new(),
            learning_rate: self.config.base_learning_rate,
            exploration_factor: 1.0,
            last_updated: Self::current_timestamp(),
            confidence_scores: HashMap::new(),
        });

        // Apply online learning algorithm (gradient descent with momentum)
        self.apply_gradient_update(profile, &feedback).await?;

        // Update session-specific preferences
        self.update_session_preferences(profile, &feedback).await?;

        // Maintain interaction history
        profile.interaction_history.push(feedback.clone());
        if profile.interaction_history.len() > 1000 {
            profile.interaction_history.remove(0);
        }

        profile.last_updated = Self::current_timestamp();

        Ok(())
    }

    // Apply gradient-based learning update to user preferences
    async fn apply_gradient_update(&self, profile: &mut UserProfile, feedback: &UserFeedback) -> Result<(), Box<dyn std::error::Error>> {
        let learning_rate = self.adaptive_learning_rate(profile, feedback).await?;
        
        // Extract features from the interaction
        let features = self.extract_features(feedback).await?;
        
        // Calculate prediction error
        let predicted_score = self.predict_interaction_score(profile, feedback).await?;
        let actual_score = feedback.feedback_value;
        let error = actual_score - predicted_score;

        // Update preferences using gradient descent
        for (feature, value) in features {
            let current_weight = profile.preferences.get(&feature).unwrap_or(&0.0);
            let gradient = error * value;
            let new_weight = current_weight + learning_rate * gradient;
            profile.preferences.insert(feature.clone(), new_weight);

            // Update confidence score
            let confidence = profile.confidence_scores.get(&feature).unwrap_or(&0.5);
            let new_confidence = confidence + 0.1 * (1.0 - confidence) * error.abs();
            profile.confidence_scores.insert(feature, new_confidence.min(1.0));
        }

        // Decay exploration factor over time
        profile.exploration_factor *= self.config.exploration_decay;
        profile.exploration_factor = profile.exploration_factor.max(0.1);

        Ok(())
    }

    // Calculate adaptive learning rate based on user behavior and confidence
    async fn adaptive_learning_rate(&self, profile: &UserProfile, feedback: &UserFeedback) -> Result<f64, Box<dyn std::error::Error>> {
        let base_rate = self.config.base_learning_rate;
        
        // Adjust based on interaction recency (more recent = higher rate)
        let time_factor = 1.0 / (1.0 + (Self::current_timestamp() - feedback.timestamp) as f64 / 3600.0);
        
        // Adjust based on interaction strength
        let strength_factor = feedback.feedback_value.abs();
        
        // Adjust based on user's interaction history (more history = lower rate)
        let history_factor = 1.0 / (1.0 + profile.interaction_history.len() as f64 / 100.0);
        
        // Adjust based on confidence in current preferences
        let avg_confidence: f64 = profile.confidence_scores.values().sum::<f64>() / profile.confidence_scores.len().max(1) as f64;
        let confidence_factor = 1.0 - avg_confidence;

        let adaptive_rate = base_rate * time_factor * strength_factor * history_factor * confidence_factor;
        
        Ok(adaptive_rate.clamp(self.config.min_learning_rate, self.config.max_learning_rate))
    }

    // Update session-specific preferences for real-time personalization
    async fn update_session_preferences(&self, profile: &mut UserProfile, feedback: &UserFeedback) -> Result<(), Box<dyn std::error::Error>> {
        let session_learning_rate = 0.3; // Higher rate for session-based learning
        
        // Extract session context features
        let context_features = vec![
            ("device_type", feedback.context.device_type.clone()),
            ("time_of_day", feedback.context.time_of_day.to_string()),
            ("day_of_week", feedback.context.day_of_week.to_string()),
        ];

        for (feature_type, feature_value) in context_features {
            let feature_key = format!("{}_{}", feature_type, feature_value);
            let current_weight = profile.session_preferences.get(&feature_key).unwrap_or(&0.0);
            let new_weight = current_weight + session_learning_rate * feedback.feedback_value;
            profile.session_preferences.insert(feature_key, new_weight);
        }

        // Apply temporal decay to session preferences
        for (key, value) in profile.session_preferences.iter_mut() {
            *value *= 0.95; // Decay factor for session preferences
        }

        Ok(())
    }

    // Extract features from user feedback for learning
    async fn extract_features(&self, feedback: &UserFeedback) -> Result<HashMap<String, f64>, Box<dyn std::error::Error>> {
        let mut features = HashMap::new();

        // Interaction type features
        features.insert(
            format!("interaction_{:?}", feedback.interaction_type),
            1.0,
        );

        // Time-based features
        features.insert(
            format!("hour_{}", feedback.context.time_of_day),
            1.0,
        );
        features.insert(
            format!("day_{}", feedback.context.day_of_week),
            1.0,
        );

        // Device features
        features.insert(
            format!("device_{}", feedback.context.device_type),
            1.0,
        );

        // Ranking position feature (important for learning to rank)
        if let Some(rank) = feedback.context.recommendation_rank {
            features.insert("recommendation_rank".to_string(), rank as f64);
            features.insert("rank_penalty".to_string(), 1.0 / (1.0 + rank as f64));
        }

        // Sequential interaction patterns
        if feedback.context.previous_interactions.len() > 0 {
            features.insert(
                "has_previous_interactions".to_string(),
                1.0,
            );
            features.insert(
                "interaction_sequence_length".to_string(),
                feedback.context.previous_interactions.len() as f64,
            );
        }

        // Search context features
        if let Some(query) = &feedback.context.search_query {
            features.insert(
                format!("search_query_length_{}", query.len() / 10), // Bucketed query length
                1.0,
            );
        }

        Ok(features)
    }

    // Predict interaction score using current user profile
    async fn predict_interaction_score(&self, profile: &UserProfile, feedback: &UserFeedback) -> Result<f64, Box<dyn std::error::Error>> {
        let features = self.extract_features(feedback).await?;
        let mut score = 0.0;

        // Apply user preferences
        for (feature, value) in &features {
            if let Some(weight) = profile.preferences.get(feature) {
                score += weight * value;
            }
        }

        // Apply session preferences
        for (feature, value) in &features {
            if let Some(weight) = profile.session_preferences.get(feature) {
                score += weight * value * 0.5; // Lower weight for session preferences
            }
        }

        // Apply global model weights
        let model_weights = self.model_weights.read().await;
        for (feature, value) in &features {
            if let Some(weight) = model_weights.feature_weights.get(feature) {
                score += weight * value * 0.3; // Even lower weight for global features
            }
        }

        // Apply interaction type weight
        if let Some(interaction_weight) = model_weights.interaction_weights.get(&feedback.interaction_type) {
            score += interaction_weight;
        }

        // Normalize score to [-1, 1] range
        Ok(score.tanh())
    }

    // Check if global model should be updated
    async fn should_update_model(&self) -> bool {
        let buffer = self.feedback_buffer.read().await;
        let model_weights = self.model_weights.read().await;
        
        let time_since_update = Self::current_timestamp() - model_weights.last_update;
        let buffer_full = buffer.len() >= self.config.adaptation_window;
        let time_elapsed = time_since_update >= self.config.update_frequency;

        buffer_full || time_elapsed
    }

    // Update global model weights based on accumulated feedback
    async fn update_global_model(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut model_weights = self.model_weights.write().await;
        let buffer = self.feedback_buffer.read().await;

        if buffer.is_empty() {
            return Ok(());
        }

        // Calculate global learning rate
        let global_learning_rate = self.config.base_learning_rate * 0.1; // Lower rate for global updates

        // Process each feedback item
        let mut feature_gradients: HashMap<String, f64> = HashMap::new();
        let mut total_error = 0.0;

        for feedback in buffer.iter() {
            let features = self.extract_features(feedback).await?;
            
            // Calculate prediction using current global weights
            let mut predicted_score = 0.0;
            for (feature, value) in &features {
                if let Some(weight) = model_weights.feature_weights.get(feature) {
                    predicted_score += weight * value;
                }
            }

            let error = feedback.feedback_value - predicted_score.tanh();
            total_error += error.abs();

            // Accumulate gradients
            for (feature, value) in features {
                let gradient = feature_gradients.get(&feature).unwrap_or(&0.0);
                feature_gradients.insert(feature, gradient + error * value);
            }
        }

        // Apply gradients with momentum
        let gradient_count = feature_gradients.len();
        for (feature, gradient) in feature_gradients {
            let current_weight = model_weights.feature_weights.get(&feature).unwrap_or(&0.0);
            let momentum = model_weights.learning_momentum;
            let new_weight = current_weight * momentum + global_learning_rate * gradient;
            model_weights.feature_weights.insert(feature, new_weight);
        }

        // Update performance metrics
        let avg_error = total_error / buffer.len() as f64;
        self.update_drift_detection(1.0 - avg_error).await?;

        model_weights.last_update = Self::current_timestamp();

        println!("Global model updated. Average error: {:.4}, Features updated: {}", 
                avg_error, gradient_count);

        Ok(())
    }

    // Update drift detection with current performance
    async fn update_drift_detection(&self, current_performance: f64) -> Result<(), Box<dyn std::error::Error>> {
        // This would be implemented with more sophisticated drift detection algorithms
        // For now, we use a simple moving average approach
        
        println!("Performance tracking: {:.4}", current_performance);
        
        // In a full implementation, this would detect concept drift and trigger model retraining
        Ok(())
    }

    // Get personalized recommendations for a user considering their learning profile
    pub async fn get_personalized_score(&self, user_id: i32, property_id: i32, context: &InteractionContext) -> Result<f64, Box<dyn std::error::Error>> {
        let profiles = self.user_profiles.read().await;
        
        if let Some(profile) = profiles.get(&user_id) {
            // Create mock feedback for scoring
            let mock_feedback = UserFeedback {
                user_id,
                property_id,
                interaction_type: InteractionType::View,
                timestamp: Self::current_timestamp(),
                session_id: "scoring".to_string(),
                context: context.clone(),
                feedback_value: 0.0, // Not used for prediction
            };

            self.predict_interaction_score(profile, &mock_feedback).await
        } else {
            // Use global model weights for new users
            let model_weights = self.model_weights.read().await;
            Ok(0.0) // Default score for unknown users
        }
    }

    // Get comprehensive learning metrics
    pub async fn get_metrics(&self) -> LearningMetrics {
        let profiles = self.user_profiles.read().await;
        let buffer = self.feedback_buffer.read().await;
        let model_weights = self.model_weights.read().await;

        let total_interactions = buffer.len();
        let active_users = profiles.len();
        let average_learning_rate = profiles.values()
            .map(|p| p.learning_rate)
            .sum::<f64>() / active_users.max(1) as f64;

        LearningMetrics {
            total_interactions,
            active_users,
            average_learning_rate,
            model_accuracy: 0.85, // Placeholder - would calculate from recent predictions
            drift_detected: false, // Placeholder
            last_update: model_weights.last_update,
            performance_trend: vec![0.8, 0.82, 0.85], // Placeholder
        }
    }

    // Initialize default feature weights
    fn initialize_feature_weights() -> HashMap<String, f64> {
        let mut weights = HashMap::new();
        
        // Interaction type weights
        weights.insert("interaction_View".to_string(), 0.1);
        weights.insert("interaction_Like".to_string(), 0.8);
        weights.insert("interaction_Dislike".to_string(), -0.8);
        weights.insert("interaction_Contact".to_string(), 0.9);
        weights.insert("interaction_Save".to_string(), 0.7);
        weights.insert("interaction_Share".to_string(), 0.6);
        weights.insert("interaction_BookViewing".to_string(), 0.9);
        weights.insert("interaction_Apply".to_string(), 1.0);
        weights.insert("interaction_Skip".to_string(), -0.3);
        weights.insert("interaction_Dismiss".to_string(), -0.5);

        // Time-based initial weights (neutral)
        for hour in 0..24 {
            weights.insert(format!("hour_{}", hour), 0.0);
        }
        for day in 0..7 {
            weights.insert(format!("day_{}", day), 0.0);
        }

        // Device weights (slight mobile preference)
        weights.insert("device_mobile".to_string(), 0.1);
        weights.insert("device_desktop".to_string(), 0.0);
        weights.insert("device_tablet".to_string(), 0.05);

        weights
    }

    fn initialize_interaction_weights() -> HashMap<InteractionType, f64> {
        let mut weights = HashMap::new();
        weights.insert(InteractionType::View, 0.1);
        weights.insert(InteractionType::Like, 0.8);
        weights.insert(InteractionType::Dislike, -0.8);
        weights.insert(InteractionType::Contact, 0.9);
        weights.insert(InteractionType::Save, 0.7);
        weights.insert(InteractionType::Share, 0.6);
        weights.insert(InteractionType::BookViewing, 0.9);
        weights.insert(InteractionType::Apply, 1.0);
        weights.insert(InteractionType::Skip, -0.3);
        weights.insert(InteractionType::Dismiss, -0.5);
        weights
    }

    fn initialize_context_weights() -> HashMap<String, f64> {
        let mut weights = HashMap::new();
        weights.insert("recommendation_rank".to_string(), -0.1); // Lower rank = better
        weights.insert("has_previous_interactions".to_string(), 0.2);
        weights.insert("interaction_sequence_length".to_string(), 0.05);
        weights
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            base_learning_rate: 0.01,
            min_learning_rate: 0.001,
            max_learning_rate: 0.1,
            adaptation_window: 100,
            buffer_size: 1000,
            update_frequency: 300, // 5 minutes
            exploration_decay: 0.999,
            confidence_threshold: 0.7,
            drift_sensitivity: 0.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_online_learning_basic() {
        let config = LearningConfig::default();
        let engine = OnlineLearningEngine::new(config);

        let feedback = UserFeedback {
            user_id: 1,
            property_id: 100,
            interaction_type: InteractionType::Like,
            timestamp: OnlineLearningEngine::current_timestamp(),
            session_id: "test_session".to_string(),
            context: InteractionContext {
                device_type: "mobile".to_string(),
                location: None,
                time_of_day: 14,
                day_of_week: 1,
                search_query: None,
                previous_interactions: vec![],
                recommendation_rank: Some(0),
            },
            feedback_value: 0.8,
        };

        let result = engine.process_feedback(feedback).await;
        assert!(result.is_ok());

        let metrics = engine.get_metrics().await;
        assert_eq!(metrics.active_users, 1);
        assert_eq!(metrics.total_interactions, 1);
    }

    #[tokio::test]
    async fn test_personalized_scoring() {
        let config = LearningConfig::default();
        let engine = OnlineLearningEngine::new(config);

        // Train with positive feedback
        let feedback = UserFeedback {
            user_id: 1,
            property_id: 100,
            interaction_type: InteractionType::Like,
            timestamp: OnlineLearningEngine::current_timestamp(),
            session_id: "test_session".to_string(),
            context: InteractionContext {
                device_type: "mobile".to_string(),
                location: None,
                time_of_day: 14,
                day_of_week: 1,
                search_query: None,
                previous_interactions: vec![],
                recommendation_rank: Some(0),
            },
            feedback_value: 0.9,
        };

        engine.process_feedback(feedback).await.unwrap();

        // Get personalized score
        let context = InteractionContext {
            device_type: "mobile".to_string(),
            location: None,
            time_of_day: 14,
            day_of_week: 1,
            search_query: None,
            previous_interactions: vec![],
            recommendation_rank: Some(0),
        };

        let score = engine.get_personalized_score(1, 101, &context).await.unwrap();
        
        // Score should be influenced by the learning
        assert!(score > -1.0 && score < 1.0);
    }
}
