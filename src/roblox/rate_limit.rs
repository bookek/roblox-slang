use super::types::CloudSyncError;
use anyhow::Result;
use log::{info, warn};
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;
#[allow(dead_code)]
pub struct RateLimiter {
    max_retries: u32,
    base_delay: u64,
}

#[allow(dead_code)]
impl RateLimiter {
    pub fn new(max_retries: u32, base_delay: u64) -> Self {
        Self {
            max_retries,
            base_delay,
        }
    }
    pub async fn execute<F, Fut, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let mut attempt = 0;

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(err) => {
                    let is_retryable = self.is_retryable_error(&err);

                    if !is_retryable || attempt >= self.max_retries {
                        return Err(err);
                    }
                    let delay = self.calculate_delay(attempt, &err);

                    warn!(
                        "Request failed (attempt {}/{}): {}. Retrying in {}s...",
                        attempt + 1,
                        self.max_retries,
                        err,
                        delay.as_secs()
                    );
                    sleep(delay).await;

                    attempt += 1;
                }
            }
        }
    }
    fn is_retryable_error(&self, err: &anyhow::Error) -> bool {
        if let Some(cloud_err) = err.downcast_ref::<CloudSyncError>() {
            matches!(
                cloud_err,
                CloudSyncError::RateLimitError { .. } | CloudSyncError::ServerError { .. }
            )
        } else {
            err.to_string().contains("network")
                || err.to_string().contains("timeout")
                || err.to_string().contains("connection")
        }
    }
    fn calculate_delay(&self, attempt: u32, err: &anyhow::Error) -> Duration {
        if let Some(CloudSyncError::RateLimitError { retry_after, .. }) =
            err.downcast_ref::<CloudSyncError>()
        {
            info!("Using Retry-After header: {}s", retry_after);
            return Duration::from_secs(*retry_after);
        }
        let delay_secs = self.base_delay * 2u64.pow(attempt);
        Duration::from_secs(delay_secs)
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(3, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_rate_limiter_success_first_try() {
        let limiter = RateLimiter::new(3, 1);

        let result = limiter
            .execute(|| async { Ok::<_, anyhow::Error>(42) })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_rate_limiter_retry_then_success() {
        let limiter = RateLimiter::new(3, 1);
        let attempts = Arc::new(Mutex::new(0));

        let attempts_clone = Arc::clone(&attempts);
        let result = limiter
            .execute(move || {
                let attempts = Arc::clone(&attempts_clone);
                async move {
                    let mut count = attempts.lock().unwrap();
                    *count += 1;
                    let current = *count;
                    drop(count);

                    if current < 2 {
                        Err(CloudSyncError::RateLimitError {
                            retry_after: 1,
                            attempt: 1,
                        }
                        .into())
                    } else {
                        Ok::<_, anyhow::Error>(42)
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(*attempts.lock().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_rate_limiter_max_retries_exceeded() {
        let limiter = RateLimiter::new(2, 1);
        let attempts = Arc::new(Mutex::new(0));

        let attempts_clone = Arc::clone(&attempts);
        let result = limiter
            .execute(move || {
                let attempts = Arc::clone(&attempts_clone);
                async move {
                    let mut count = attempts.lock().unwrap();
                    *count += 1;
                    let current = *count;
                    drop(count);

                    Err::<(), _>(
                        CloudSyncError::RateLimitError {
                            retry_after: 1,
                            attempt: current as u32,
                        }
                        .into(),
                    )
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(*attempts.lock().unwrap(), 3); // Initial + 2 retries
    }

    #[tokio::test]
    async fn test_rate_limiter_non_retryable_error() {
        let limiter = RateLimiter::new(3, 1);
        let attempts = Arc::new(Mutex::new(0));

        let attempts_clone = Arc::clone(&attempts);
        let result = limiter
            .execute(move || {
                let attempts = Arc::clone(&attempts_clone);
                async move {
                    let mut count = attempts.lock().unwrap();
                    *count += 1;
                    drop(count);

                    Err::<(), _>(
                        CloudSyncError::AuthenticationError("Invalid API key".to_string()).into(),
                    )
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(*attempts.lock().unwrap(), 1); // No retries for auth errors
    }

    #[tokio::test]
    async fn test_rate_limiter_respects_retry_after() {
        let limiter = RateLimiter::new(3, 1);

        let delay = limiter.calculate_delay(
            0,
            &CloudSyncError::RateLimitError {
                retry_after: 5,
                attempt: 1,
            }
            .into(),
        );

        assert_eq!(delay, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_rate_limiter_exponential_backoff() {
        let limiter = RateLimiter::new(3, 1);
        let delay0 = limiter.calculate_delay(
            0,
            &CloudSyncError::ServerError {
                status: 500,
                message: "Server error".to_string(),
            }
            .into(),
        );
        assert_eq!(delay0, Duration::from_secs(1));
        let delay1 = limiter.calculate_delay(
            1,
            &CloudSyncError::ServerError {
                status: 500,
                message: "Server error".to_string(),
            }
            .into(),
        );
        assert_eq!(delay1, Duration::from_secs(2));
        let delay2 = limiter.calculate_delay(
            2,
            &CloudSyncError::ServerError {
                status: 500,
                message: "Server error".to_string(),
            }
            .into(),
        );
        assert_eq!(delay2, Duration::from_secs(4));
    }

    #[test]
    fn test_rate_limiter_default() {
        let limiter = RateLimiter::default();
        assert_eq!(limiter.max_retries, 3);
        assert_eq!(limiter.base_delay, 1);
    }
}
