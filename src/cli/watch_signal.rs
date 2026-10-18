//! Portable shutdown signals for the continuous watch runtime.

use std::io;

pub(super) trait ShutdownSignal {
    async fn recv(&mut self) -> io::Result<()>;
}

#[cfg(unix)]
pub(super) struct PlatformShutdownSignal(tokio::signal::unix::Signal);

#[cfg(windows)]
pub(super) struct PlatformShutdownSignal {
    ctrl_c: tokio::signal::windows::CtrlC,
    ctrl_break: tokio::signal::windows::CtrlBreak,
}

#[cfg(unix)]
pub(super) fn listen() -> io::Result<PlatformShutdownSignal> {
    let signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
    Ok(PlatformShutdownSignal(signal))
}

#[cfg(windows)]
pub(super) fn listen() -> io::Result<PlatformShutdownSignal> {
    Ok(PlatformShutdownSignal {
        ctrl_c: tokio::signal::windows::ctrl_c()?,
        ctrl_break: tokio::signal::windows::ctrl_break()?,
    })
}

#[cfg(unix)]
impl ShutdownSignal for PlatformShutdownSignal {
    async fn recv(&mut self) -> io::Result<()> {
        self.0.recv().await;
        Ok(())
    }
}

#[cfg(windows)]
impl ShutdownSignal for PlatformShutdownSignal {
    async fn recv(&mut self) -> io::Result<()> {
        tokio::select! {
            _ = self.ctrl_c.recv() => Ok(()),
            signal = self.ctrl_break.recv() => signal.ok_or_else(|| {
                io::Error::new(io::ErrorKind::BrokenPipe, "Ctrl-Break stream closed")
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ShutdownSignal;
    use std::io;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tokio::sync::broadcast;

    struct ModelSignal {
        receiver: broadcast::Receiver<()>,
    }

    impl ModelSignal {
        fn subscribe(sender: &broadcast::Sender<()>, constructions: &Arc<AtomicUsize>) -> Self {
            constructions.fetch_add(1, Ordering::SeqCst);
            Self {
                receiver: sender.subscribe(),
            }
        }
    }

    impl ShutdownSignal for ModelSignal {
        async fn recv(&mut self) -> io::Result<()> {
            self.receiver
                .recv()
                .await
                .map(|_| ())
                .map_err(|error| io::Error::other(error.to_string()))
        }
    }

    #[tokio::test]
    async fn retained_listener_observes_signal_after_select_cancellation() {
        let (sender, receiver) = broadcast::channel(1);
        drop(receiver);
        let constructions = Arc::new(AtomicUsize::new(0));
        let mut listener = ModelSignal::subscribe(&sender, &constructions);
        assert_eq!(sender.receiver_count(), 1);

        tokio::select! {
            biased;
            _ = listener.recv() => panic!("no signal was published"),
            _ = std::future::ready(()) => {},
        }

        sender
            .send(())
            .expect("retained listener remains subscribed");
        listener.recv().await.unwrap();

        assert_eq!(constructions.load(Ordering::SeqCst), 1);
        assert_eq!(sender.receiver_count(), 1);
    }

    #[tokio::test]
    async fn recreated_listener_does_not_receive_signal_published_without_subscriber() {
        let (sender, receiver) = broadcast::channel(1);
        drop(receiver);
        let constructions = Arc::new(AtomicUsize::new(0));
        let listener = ModelSignal::subscribe(&sender, &constructions);
        assert_eq!(sender.receiver_count(), 1);
        drop(listener);
        assert_eq!(sender.receiver_count(), 0);

        assert!(sender.send(()).is_err());

        let mut recreated = ModelSignal::subscribe(&sender, &constructions);
        assert!(matches!(
            recreated.receiver.try_recv(),
            Err(broadcast::error::TryRecvError::Empty)
        ));
        assert_eq!(constructions.load(Ordering::SeqCst), 2);
        assert_eq!(sender.receiver_count(), 1);
    }
}
