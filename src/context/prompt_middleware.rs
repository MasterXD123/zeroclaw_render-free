//! Prompt Middleware - Token budgeting and context fragmentation.
//!
//! Provides token counting and budget management for LLM context windows.

use std::collections::HashMap;

/// Token estimation result
#[derive(Debug, Clone)]
pub struct TokenEstimate {
    pub tokens: usize,
    pub characters: usize,
}

/// Prompt fragment with budget allocation
#[derive(Debug, Clone)]
pub struct FragmentBudget {
    pub content: String,
    pub token_budget: usize,
    pub estimated_tokens: usize,
}

/// Prompt Middleware for token budgeting
pub struct PromptMiddleware {
    /// Default max tokens per fragment (4K for most models)
    max_context_tokens: usize,
    /// Budgets by fragment ID
    budgets: HashMap<String, usize>,
    /// Token counts cache (simple approximation)
    token_cache: HashMap<String, usize>,
}

impl PromptMiddleware {
    /// Create new middleware with max context tokens
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_context_tokens: max_tokens,
            budgets: HashMap::new(),
            token_cache: HashMap::new(),
        }
    }

    /// Estimate tokens using simple approximation (4 chars per token avg)
    pub fn estimate_tokens(&self, text: &str) -> usize {
        if let Some(cached) = self.token_cache.get(text) {
            return *cached;
        }
        // Simple approximation: ~4 characters per token for English
        let estimate = text.chars().count() / 4;
        self.token_cache.insert(text.to_string(), estimate);
        estimate
    }

    /// Budget for a fragment - returns truncated text if over budget
    pub fn budget_for_fragment(&self, fragment: &str) -> FragmentBudget {
        let tokens = self.estimate_tokens(fragment);

        let content = if tokens > self.max_context_tokens {
            // Truncate to fit budget
            let ratio = self.max_context_tokens as f64 / tokens as f64;
            let truncated_len = (fragment.len() as f64 * ratio) as usize;
            fragment[..truncated_len].to_string()
        } else {
            fragment.to_string()
        };

        let estimated_tokens = self.estimate_tokens(&content);

        FragmentBudget {
            content,
            token_budget: self.max_context_tokens,
            estimated_tokens,
        }
    }

    /// Set explicit budget for a fragment ID
    pub fn set_budget(&mut self, fragment_id: &str, tokens: usize) {
        self.budgets.insert(fragment_id.to_string(), tokens);
    }

    /// Get budget for fragment ID
    pub fn get_budget(&self, fragment_id: &str) -> usize {
        self.budgets
            .get(fragment_id)
            .copied()
            .unwrap_or(self.max_context_tokens)
    }

    /// Calculate total tokens across multiple fragments
    pub fn total_tokens(&self, fragments: &[&str]) -> usize {
        fragments.iter().map(|f| self.estimate_tokens(f)).sum()
    }

    /// Check if total tokens fit within budget
    pub fn fits_budget(&self, fragments: &[&str]) -> bool {
        self.total_tokens(fragments) <= self.max_context_tokens
    }

    /// Split text into fragments that fit token budget
    pub fn split_to_fit(&self, text: &str) -> Vec<String> {
        let mut fragments = Vec::new();
        let mut remaining = text;

        while self.estimate_tokens(remaining) > self.max_context_tokens {
            // Find break point
            let chars_per_token = 4;
            let max_chars = self.max_context_tokens * chars_per_token;

            // Find a good break point (end of sentence or paragraph)
            let break_idx = remaining[..max_chars.min(remaining.len())]
                .rfind(|c| c == '.' || c == '\n')
                .unwrap_or(max_chars.min(remaining.len()));

            if break_idx == 0 {
                // No break found, force break at max
                fragments.push(remaining[..max_chars.min(remaining.len())].to_string());
                remaining = &remaining[max_chars.min(remaining.len())..];
            } else {
                fragments.push(remaining[..break_idx].to_string());
                remaining = &remaining[break_idx + 1..];
            }
        }

        if !remaining.is_empty() {
            fragments.push(remaining.to_string());
        }

        fragments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_estimate() {
        let mw = PromptMiddleware::new(4000);
        let text = "Hello, this is a test message.";
        let tokens = mw.estimate_tokens(text);
        assert!(tokens > 0);
    }

    #[test]
    fn test_split_to_fit() {
        let mw = PromptMiddleware::new(10);
        let text = "This is a very long text that should be split into multiple fragments because it exceeds the token budget.";
        let fragments = mw.split_to_fit(text);
        assert!(fragments.len() > 1);
    }
}