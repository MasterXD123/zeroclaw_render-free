//! Resource Router - Cost control and caching
//!
//! Provides cost estimation, budget management, and response caching.

use std::collections::HashMap;
use std::sync::RwLock;

/// Model cost information (input, output per 1K tokens)
#[derive(Debug, Clone)]
pub struct ModelCosts {
    pub input_per_1k: f64,
    pub output_per_1k: f64,
}

/// Resource usage record
#[derive(Debug, Clone)]
pub struct UsageRecord {
    pub user: String,
    pub model: String,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub cost: f64,
}

/// Resource Router for cost and cache management
pub struct ResourceRouter {
    /// Model costs (model name -> (input, output) per 1K tokens)
    model_costs: HashMap<String, ModelCosts>,
    /// User budgets (user -> max cost)
    budgets: HashMap<String, f64>,
    /// Usage records
    usage: RwLock<Vec<UsageRecord>>,
    /// Response cache (key -> value)
    cache: RwLock<HashMap<String, String>>,
}

impl Default for ResourceRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceRouter {
    /// Create new resource router with default model costs
    pub fn new() -> Self {
        let mut costs = HashMap::new();
        costs.insert(
            "gpt-4".to_string(),
            ModelCosts {
                input_per_1k: 0.03,
                output_per_1k: 0.06,
            },
        );
        costs.insert(
            "gpt-3.5-turbo".to_string(),
            ModelCosts {
                input_per_1k: 0.0015,
                output_per_1k: 0.002,
            },
        );
        costs.insert(
            "claude-3-opus".to_string(),
            ModelCosts {
                input_per_1k: 0.015,
                output_per_1k: 0.075,
            },
        );
        costs.insert(
            "claude-3-sonnet".to_string(),
            ModelCosts {
                input_per_1k: 0.003,
                output_per_1k: 0.015,
            },
        );

        Self {
            model_costs: costs,
            budgets: HashMap::new(),
            usage: RwLock::new(Vec::new()),
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Add or update model costs
    pub fn set_model_costs(&mut self, model: &str, input: f64, output: f64) {
        self.model_costs.insert(
            model.to_string(),
            ModelCosts {
                input_per_1k: input,
                output_per_1k: output,
            },
        );
    }

    /// Set user budget
    pub fn set_budget(&mut self, user: &str, max_cost: f64) {
        self.budgets.insert(user.to_string(), max_cost);
    }

    /// Estimate cost for a request
    pub fn estimate_cost(&self, model: &str, input_tokens: usize, output_tokens: usize) -> f64 {
        let (input_cost, output_cost) = self
            .model_costs
            .get(model)
            .map(|c| (c.input_per_1k, c.output_per_1k))
            .unwrap_or((0.01, 0.03));

        (input_tokens as f64 / 1000.0 * input_cost)
            + (output_tokens as f64 / 1000.0 * output_cost)
    }

    /// Check if user has budget for a cost
    pub fn check_budget(&self, user: &str, cost: f64) -> bool {
        let budget = self.budgets.get(user).unwrap_or(&100.0); // Default 100
        cost <= *budget
    }

    /// Get remaining budget for user
    pub fn remaining_budget(&self, user: &str) -> f64 {
        let budget = self.budgets.get(user).unwrap_or(&100.0);
        let used = self.total_cost(user);
        budget - used
    }

    /// Record usage
    pub fn record_usage(&self, user: &str, model: &str, input_tokens: usize, output_tokens: usize) {
        let cost = self.estimate_cost(model, input_tokens, output_tokens);
        let record = UsageRecord {
            user: user.to_string(),
            model: model.to_string(),
            input_tokens,
            output_tokens,
            cost,
        };
        if let Ok(mut usage) = self.usage.write() {
            usage.push(record);
        }
    }

    /// Get total cost for user
    pub fn total_cost(&self, user: &str) -> f64 {
        if let Ok(usage) = self.usage.read() {
            usage.iter().filter(|u| u.user == user).map(|u| u.cost).sum()
        } else {
            0.0
        }
    }

    /// Get cache value
    pub fn cache_get(&self, key: &str) -> Option<String> {
        self.cache.read().ok()?.get(key).cloned()
    }

    /// Set cache value
    pub fn cache_set(&self, key: String, value: String) {
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(key, value);
        }
    }

    /// Clear cache
    pub fn cache_clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.read().map(|c| c.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_cost() {
        let router = ResourceRouter::new();
        let cost = router.estimate_cost("gpt-4", 1000, 500);
        assert!(cost > 0.0);
        assert_eq!(cost, 0.03 + 0.03); // 1K input + 0.5K output
    }

    #[test]
    fn test_check_budget() {
        let mut router = ResourceRouter::new();
        router.set_budget("user1", 10.0);
        assert!(router.check_budget("user1", 5.0));
        assert!(!router.check_budget("user1", 15.0));
    }

    #[test]
    fn test_cache() {
        let router = ResourceRouter::new();
        router.cache_set("key1".to_string(), "value1".to_string());
        assert_eq!(router.cache_get("key1"), Some("value1".to_string()));
        assert_eq!(router.cache_get("key2"), None);
    }

    #[test]
    fn test_record_usage() {
        let router = ResourceRouter::new();
        router.record_usage("user1", "gpt-4", 1000, 500);
        let total = router.total_cost("user1");
        assert!(total > 0.0);
    }
}