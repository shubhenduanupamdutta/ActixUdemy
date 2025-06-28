use std::io;
use tracing::info;
use tracing_appender::{
    non_blocking,
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

pub fn init_tracing() -> Vec<WorkerGuard> {
    let mut guards = Vec::new();

    // Determine if we are in development or production
    let is_development =
        std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()) == "development";

    // Create daily rolling file appender
    let file_appender = RollingFileAppender::new(Rotation::DAILY, "logs", "app.jsonl");
    let (file_writer, file_guard) = non_blocking(file_appender);
    guards.push(file_guard);

    // Create stdout writer for info and below
    let (stdout_writer, stdout_guard) = non_blocking(io::stdout());
    guards.push(stdout_guard);

    // Create stderr writer for warn and error
    let (stderr_writer, stderr_guard) = non_blocking(io::stderr());
    guards.push(stderr_guard);

    // Get stdout log level from environment, default to "info"
    let stdout_level = std::env::var("RUST_LOG_STDOUT").unwrap_or_else(|_| {
        if is_development {
            "trace".to_string()
        } else {
            "info".to_string()
        }
    });

    // Get file log level from environment, default to 'trace' in 'dev' and 'info' in production
    let file_level = std::env::var("RUST_LOG_FILE").unwrap_or_else(|_| {
        if is_development {
            "trace".to_string()
        } else {
            "info".to_string()
        }
    });

    // Configure the subscriber with multiple layers
    tracing_subscriber::registry()
        .with(
            // File layer - ALL levels including trace and debug
            fmt::layer()
                .json()
                .with_writer(file_writer)
                .with_ansi(false) // No ANSI colors in files
                .with_filter(EnvFilter::new(&file_level)),
        )
        .with(
            // Stdout layer - configurable via environment
            fmt::layer()
                .with_writer(stdout_writer)
                .with_filter(EnvFilter::new(&stdout_level)),
        )
        .with(
            // Stderr layer - warn and error only
            fmt::layer()
                .with_writer(stderr_writer)
                .with_filter(EnvFilter::new("warn")),
        )
        .init();

    info!(file_log_level = %file_level, stdout_log_level = %stdout_level, development = %is_development);

    guards
}
