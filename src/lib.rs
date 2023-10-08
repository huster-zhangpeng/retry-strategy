use futures::Future;
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::time::{sleep, Sleep};

pub mod error;
pub mod strategy;

pub trait ToDuration {
    fn s(self) -> Duration;
    fn ms(self) -> Duration;
    fn ns(self) -> Duration;
}

impl ToDuration for u64 {
    fn s(self) -> Duration {
        Duration::from_secs(self)
    }

    fn ms(self) -> Duration {
        Duration::from_millis(self)
    }

    fn ns(self) -> Duration {
        Duration::from_nanos(self)
    }
}

impl ToDuration for f64 {
    fn s(self) -> Duration {
        Duration::from_secs_f64(self)
    }

    fn ms(self) -> Duration {
        Duration::from_secs_f64(self / 1000.0)
    }

    fn ns(self) -> Duration {
        Duration::from_secs_f64(self / 1_000_000_000.0)
    }
}

impl ToDuration for f32 {
    fn s(self) -> Duration {
        Duration::from_secs_f32(self)
    }

    fn ms(self) -> Duration {
        Duration::from_secs_f32(self / 1000.0)
    }

    fn ns(self) -> Duration {
        Duration::from_secs_f32(self / 1_000_000_000.0)
    }
}

///
/// # Example
/// ```
/// let res = retry(
///     vec![100.ms(), 200.ms(), 300.ms()],
///     |n| async move {
///         sleep(Duration::from_millis(250)).await;
///         n
///     }
/// ).await;
/// ```
pub fn retry<I, A, F>(iter: I, action: A) -> Retry<I::IntoIter, A, F>
where
    I: IntoIterator<Item = Duration>,
    A: FnMut(i32) -> F,
    F: Future,
{
    Retry {
        strategy: iter.into_iter(),
        action,
        state: RetryState::default(),
    }
}

#[pin_project(project = RetryStateProj)]
#[derive(Default)]
enum RetryState<F> {
    #[default]
    Initing,
    Running(i32, #[pin] Sleep, #[pin] F),
}

enum RetryPoll<F>
where
    F: Future,
{
    Initing(i32),
    Pending,
    Ready(<F as Future>::Output),
}

impl<F> RetryState<F>
where
    F: Future,
{
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> RetryPoll<F> {
        match self.project() {
            RetryStateProj::Initing => RetryPoll::Initing(0),
            RetryStateProj::Running(n, sleep, fut) => match fut.poll(cx) {
                Poll::Ready(result) => RetryPoll::Ready(result),
                Poll::Pending => match sleep.poll(cx) {
                    Poll::Pending => RetryPoll::Pending,
                    Poll::Ready(()) => RetryPoll::Initing(*n + 1),
                },
            },
        }
    }
}

#[pin_project]
pub struct Retry<I, A, F>
where
    I: Iterator<Item = Duration>,
    A: FnMut(i32) -> F,
    F: Future,
{
    strategy: I,
    action: A,
    #[pin]
    state: RetryState<F>,
}

impl<I, A, F> Future for Retry<I, A, F>
where
    I: Iterator<Item = Duration>,
    A: FnMut(i32) -> F,
    F: Future,
{
    type Output = Result<<F as Future>::Output, error::Exhausted>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let me = self.as_mut().project();
            match me.state.poll(cx) {
                RetryPoll::Initing(n) => {
                    let period = me.strategy.next();
                    match period {
                        None => return Poll::Ready(Err(error::Exhausted::new())),
                        Some(tv) => {
                            let fut = (me.action)(n);
                            let sleep = sleep(tv);
                            self.as_mut()
                                .project()
                                .state
                                .set(RetryState::Running(n, sleep, fut));
                        }
                    };
                }
                RetryPoll::Pending => return Poll::Pending,
                RetryPoll::Ready(result) => return Poll::Ready(Ok(result)),
            };
        }
    }
}

pub mod prelude {
    pub use super::error::Exhausted;
    pub use super::strategy::{Exponential, Fibonacci, Fixed, NoDelay};
    pub use super::{retry, Retry, ToDuration};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_retry() {
        let counter = Arc::new(Mutex::new(0));
        let m = &counter;

        let res = retry(vec![100.ms(), 200.ms(), 300.ms()], |n| async move {
            {
                let mut lock = m.lock().unwrap();
                assert_eq!(*lock, n);
                *lock += 1;
            }
            sleep(Duration::from_millis(250)).await;
            n
        })
        .await;

        assert_eq!(3, *counter.lock().unwrap());
        assert_eq!(res, Ok(2));
    }
}
