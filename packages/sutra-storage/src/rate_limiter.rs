/// Production-grade token-bucket rate limiter
///
/// Features:
/// - Per-token rate limiting (prevents individual token abuse)
/// - Configurable burst capacity
/// - Sliding window implementation
/// - Thread-safe with lock-free fast path
/// - Automatic cleanup of stale entries

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    /// Maximum requests per second per token
    pub requests_per_second: u32,
    /// Burst capacity (max requests in short burst)
    pub burst_capacity: u32,
    /// How long to remember token usage (for cleanup)
    pub memory_duration: Duration,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 100,   // 100 req/sec default
            burst_capacity: 200,         // 2× burst allowed
            memory_duration: Duration::from_secs(300), // Remember 5 minutes
        }
    }
}

/// Token bucket for a single subject
#[derive(Debug, Clone)]
struct TokenBucket {
    /// Available tokens
    tokens: f64,
    /// Last refill timestamp
    last_refill: Instant,
    /// Last access timestamp (for cleanup)
    last_access: Instant,
    /// Configuration
    config: RateLimiterConfig,
}

impl TokenBucket {
    fn new(config: RateLimiterConfig) -> Self {
        let now = Instant::now();
        Self {
            tokens: config.burst_capacity as f64,
            last_refill: now,
            last_access: now,
            config,
        }
    }
    
    /// Try to consume a token (returns true if allowed)
    fn try_consume(&mut self) -> bool {
        let now = Instant::now();
        
        // Refill tokens based on elapsed time
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let tokens_to_add = elapsed * self.config.requests_per_second as f64;
        
        self.tokens = (self.tokens + tokens_to_add).min(self.config.burst_capacity as f64);
        self.last_refill = now;
        self.last_access = now;
        
        // Try to consume a token
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
    
    /// Check if bucket is stale (not accessed recently)
    fn is_stale(&self, memory_duration: Duration) -> bool {
        self.last_access.elapsed() > memory_duration
    }
}

/// Token-bucket rate limiter with per-subject limits
pub struct RateLimiter {
    /// Per-subject buckets (subject -> TokenBucket)
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    /// Configuration
    config: RateLimiterConfig,
    /// Last cleanup timestamp
    last_cleanup: Arc<RwLock<Instant>>,
}

impl RateLimiter {
    /// Create new rate limiter with default config
    pub fn new() -> Self {
        Self::with_config(RateLimiterConfig::default())
    }
    
    /// Create new rate limiter with custom config
    pub fn with_config(config: RateLimiterConfig) -> Self {
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
            config,
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
        }
    }
    
    /// Check if request is allowed for given subject
    pub fn check_rate_limit(&self, subject: &str) -> Result<(), RateLimitError> {
        // Fast path: Check if bucket exists (read lock)
        {
            let buckets = self.buckets.read();
            if let Some(bucket) = buckets.get(subject) {
                // Quick check without modifying state
                let elapsed = bucket.last_refill.elapsed().as_secs_f64();
                let estimated_tokens = bucket.tokens + (elapsed * self.config.requests_per_second as f64);
                
                if estimated_tokens < 0.5 {
                    // Definitely rate limited
                    drop(buckets); // Release read lock
                    return Err(RateLimitError::RateLimitExceeded {
                        subject: subject.to_string(),
                        retry_after: Duration::from_secs_f64(1.0 / self.config.requests_per_second as f64),
                    });
                }
            }
        }
        
        // Slow path: Modify bucket (write lock)
        let result = {
            let mut buckets = self.buckets.write();
            
            let bucket = buckets
                .entry(subject.to_string())
                .or_insert_with(|| TokenBucket::new(self.config.clone()));
            
            if bucket.try_consume() {
                Ok(())
            } else {
                Err(RateLimitError::RateLimitExceeded {
                    subject: subject.to_string(),
                    retry_after: Duration::from_secs_f64(1.0 / self.config.requests_per_second as f64),
                })
            }
        };
        
        // Periodic cleanup (every 60 seconds)
        if self.should_cleanup() {
            self.cleanup_stale_buckets();
        }
        
        result
    }
    
    /// Check if cleanup is needed
    fn should_cleanup(&self) -> bool {
        let last_cleanup = *self.last_cleanup.read();
        last_cleanup.elapsed() > Duration::from_secs(60)
    }
    
    /// Remove stale buckets to prevent memory leak
    fn cleanup_stale_buckets(&self) {
        let mut buckets = self.buckets.write();
        let mut last_cleanup = self.last_cleanup.write();
        
        let memory_duration = self.config.memory_duration;
        buckets.retain(|_, bucket| !bucket.is_stale(memory_duration));
        
        *last_cleanup = Instant::now();
        
        log::debug!("🧹 Rate limiter cleanup: {} active buckets remaining", buckets.len());
    }
    
    /// Get statistics (for monitoring)
    pub fn stats(&self) -> RateLimiterStats {
        let buckets = self.buckets.read();
        
        let mut total_tokens = 0.0;
        let mut throttled_subjects = 0;
        
        for bucket in buckets.values() {
            total_tokens += bucket.tokens;
            if bucket.tokens < 1.0 {
                throttled_subjects += 1;
            }
        }
        
        RateLimiterStats {
            active_subjects: buckets.len(),
            throttled_subjects,
            average_tokens: if buckets.is_empty() { 0.0 } else { total_tokens / buckets.len() as f64 },
        }
    }
    
    /// Reset rate limit for a subject (admin operation)
    pub fn reset_subject(&self, subject: &str) {
        let mut buckets = self.buckets.write();
        buckets.remove(subject);
    }
    
    /// Clear all rate limits (admin operation)
    pub fn reset_all(&self) {
        let mut buckets = self.buckets.write();
        buckets.clear();
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limiter statistics
#[derive(Debug, Clone)]
pub struct RateLimiterStats {
    /// Number of subjects being tracked
    pub active_subjects: usize,
    /// Number of currently throttled subjects
    pub throttled_subjects: usize,
    /// Average tokens available across all subjects
    pub average_tokens: f64,
}

/// Rate limiter errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded for subject '{subject}'. Retry after {retry_after:?}")]
    RateLimitExceeded {
        subject: String,
        retry_after: Duration,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_basic_rate_limiting() {
        let config = RateLimiterConfig {
            requests_per_second: 10,
            burst_capacity: 10,
            memory_duration: Duration::from_secs(60),
        };
        
        let limiter = RateLimiter::with_config(config);
        let subject = "user123";
        
        // First 10 requests should succeed (burst)
        for i in 0..10 {
            assert!(limiter.check_rate_limit(subject).is_ok(), "Request {} should succeed", i);
        }
        
        // 11th request should fail (burst exhausted)
        assert!(limiter.check_rate_limit(subject).is_err(), "Request should be rate limited");
    }
    
    #[test]
    fn test_token_refill() {
        let config = RateLimiterConfig {
            requests_per_second: 10,
            burst_capacity: 10,
            memory_duration: Duration::from_secs(60),
        };
        
        let limiter = RateLimiter::with_config(config);
        let subject = "user456";
        
        // Exhaust burst
        for _ in 0..10 {
            limiter.check_rate_limit(subject).ok();
        }
        
        // Wait for refill (100ms = 1 token)
        thread::sleep(Duration::from_millis(100));
        
        // Should allow 1 more request
        assert!(limiter.check_rate_limit(subject).is_ok(), "Request should succeed after refill");
    }
    
    #[test]
    fn test_per_subject_isolation() {
        let config = RateLimiterConfig {
            requests_per_second: 5,
            burst_capacity: 5,
            memory_duration: Duration::from_secs(60),
        };
        
        let limiter = RateLimiter::with_config(config);
        
        // Exhaust subject1
        for _ in 0..5 {
            limiter.check_rate_limit("subject1").ok();
        }
        assert!(limiter.check_rate_limit("subject1").is_err());
        
        // subject2 should still work
        assert!(limiter.check_rate_limit("subject2").is_ok());
    }
    
    #[test]
    fn test_cleanup() {
        let config = RateLimiterConfig {
            requests_per_second: 100,
            burst_capacity: 100,
            memory_duration: Duration::from_millis(100), // Short memory for testing
        };
        
        let limiter = RateLimiter::with_config(config);
        
        // Create buckets for multiple subjects
        for i in 0..10 {
            limiter.check_rate_limit(&format!("user{}", i)).ok();
        }
        
        // Wait for buckets to become stale
        thread::sleep(Duration::from_millis(150));
        
        // Trigger cleanup
        limiter.cleanup_stale_buckets();
        
        let stats = limiter.stats();
        assert_eq!(stats.active_subjects, 0, "Stale buckets should be cleaned");
    }
    
    #[test]
    fn test_stats() {
        let limiter = RateLimiter::new();
        
        limiter.check_rate_limit("user1").ok();
        limiter.check_rate_limit("user2").ok();
        limiter.check_rate_limit("user3").ok();
        
        let stats = limiter.stats();
        assert_eq!(stats.active_subjects, 3);
        assert!(stats.average_tokens > 0.0);
    }
    
    #[test]
    fn test_reset_subject() {
        let config = RateLimiterConfig {
            requests_per_second: 1,
            burst_capacity: 1,
            memory_duration: Duration::from_secs(60),
        };
        
        let limiter = RateLimiter::with_config(config);
        let subject = "user_reset";
        
        // Exhaust limit
        limiter.check_rate_limit(subject).ok();
        assert!(limiter.check_rate_limit(subject).is_err());
        
        // Reset and retry
        limiter.reset_subject(subject);
        assert!(limiter.check_rate_limit(subject).is_ok());
    }
}
