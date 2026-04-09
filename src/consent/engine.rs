//! Consent Engine - Declarative authorization policies
//!
//! Provides consent levels and policy checking for agent operations.

use std::collections::HashMap;

/// Consent levels for operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsentLevel {
    /// Execute without approval
    Auto,
    /// Notify but execute
    Notify,
    /// Require explicit approval
    Require,
    /// Block the operation
    Block,
}

impl Default for ConsentLevel {
    fn default() -> Self {
        ConsentLevel::Notify
    }
}

impl std::fmt::Display for ConsentLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsentLevel::Auto => write!(f, "auto"),
            ConsentLevel::Notify => write!(f, "notify"),
            ConsentLevel::Require => write!(f, "require"),
            ConsentLevel::Block => write!(f, "block"),
        }
    }
}

impl std::str::FromStr for ConsentLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(ConsentLevel::Auto),
            "notify" => Ok(ConsentLevel::Notify),
            "require" => Ok(ConsentLevel::Require),
            "block" => Ok(ConsentLevel::Block),
            _ => Err(format!("Unknown consent level: {}", s)),
        }
    }
}

/// Consent check result
#[derive(Debug, Clone)]
pub struct ConsentCheck {
    pub allowed: bool,
    pub level: ConsentLevel,
    pub reason: Option<String>,
    pub requires_approval: bool,
}

impl ConsentCheck {
    pub fn allowed(level: ConsentLevel) -> Self {
        Self {
            allowed: true,
            level,
            reason: None,
            requires_approval: matches!(level, ConsentLevel::Require),
        }
    }

    pub fn blocked(reason: &str) -> Self {
        Self {
            allowed: false,
            level: ConsentLevel::Block,
            reason: Some(reason.to_string()),
            requires_approval: false,
        }
    }
}

/// Consent Engine for policy-based authorization
pub struct ConsentEngine {
    policies: HashMap<String, ConsentLevel>,
    default_level: ConsentLevel,
}

impl Default for ConsentEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsentEngine {
    /// Create new consent engine with default level
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
            default_level: ConsentLevel::Notify,
        }
    }

    /// Create with default policy
    pub fn with_default(level: ConsentLevel) -> Self {
        Self {
            policies: HashMap::new(),
            default_level: level,
        }
    }

    /// Set consent level for an action
    pub fn set_level(&mut self, action: &str, level: ConsentLevel) {
        self.policies.insert(action.to_string(), level);
    }

    /// Get consent level for an action
    pub fn get_level(&self, action: &str) -> ConsentLevel {
        self.policies.get(action).copied().unwrap_or(self.default_level)
    }

    /// Check if action is blocked
    pub fn is_blocked(&self, action: &str) -> bool {
        matches!(self.get_level(action), ConsentLevel::Block)
    }

    /// Check if action requires approval
    pub fn requires_approval(&self, action: &str) -> bool {
        matches!(self.get_level(action), ConsentLevel::Require)
    }

    /// Perform consent check for an action
    pub fn check(&self, action: &str) -> ConsentCheck {
        let level = self.get_level(action);

        match level {
            ConsentLevel::Auto => ConsentCheck::allowed(level),
            ConsentLevel::Notify => ConsentCheck::allowed(level),
            ConsentLevel::Require => ConsentCheck::allowed(level),
            ConsentLevel::Block => ConsentCheck::blocked(&format!("Action '{}' is blocked by policy", action)),
        }
    }

    /// Check multiple actions at once
    pub fn check_many(&self, actions: &[&str]) -> Vec<ConsentCheck> {
        actions.iter().map(|a| self.check(a)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_level() {
        let engine = ConsentEngine::new();
        assert_eq!(engine.get_level("unknown_action"), ConsentLevel::Notify);
    }

    #[test]
    fn test_set_level() {
        let mut engine = ConsentEngine::new();
        engine.set_level("delete", ConsentLevel::Require);
        assert_eq!(engine.get_level("delete"), ConsentLevel::Require);
    }

    #[test]
    fn test_is_blocked() {
        let mut engine = ConsentEngine::new();
        engine.set_level("dangerous", ConsentLevel::Block);
        assert!(engine.is_blocked("dangerous"));
        assert!(!engine.is_blocked("safe"));
    }

    #[test]
    fn test_consent_check() {
        let engine = ConsentEngine::with_default(ConsentLevel::Auto);
        let check = engine.check("any_action");
        assert!(check.allowed);
        assert!(!check.requires_approval);
    }

    #[test]
    fn test_require_approval() {
        let mut engine = ConsentEngine::new();
        engine.set_level("sensitive", ConsentLevel::Require);
        let check = engine.check("sensitive");
        assert!(check.requires_approval);
    }
}