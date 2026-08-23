//! Portable shutdown signals for the continuous watch runtime.

#[cfg(not(windows))]
pub(super) async fn shutdown_signal() -> std::io::Result<()> {
    tokio::signal::ctrl_c().await
}

#[cfg(windows)]
pub(super) async fn shutdown_signal() -> std::io::Result<()> {
    let mut ctrl_break = tokio::signal::windows::ctrl_break()?;
    tokio::select! {
        result = tokio::signal::ctrl_c() => result,
        signal = ctrl_break.recv() => signal.ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Ctrl-Break stream closed")
        }),
    }
}
