use audit_layer::AuditLayer;
use std::{env::var, time::Duration};
use tokio::{runtime::Handle, time::sleep};
use tracing::{error, info, trace, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Layer, Registry};

#[tokio::main]
async fn main() {
    let log_endpoint =
        var("AUDIT_LOG_ENDPOINT").expect("Env Variable for audit logger's HTTP endpoint");
    let username =
        var("AUDIT_LOG_USERNAME").expect("Env Variable for audit logger instance username");
    let password =
        var("AUDIT_LOG_PASSWORD").expect("Env Variable for audit logger instance username");

    let audit_layer = AuditLayer::new(log_endpoint, username, password, Handle::current());
    let stdout_layer = fmt::layer()
        .with_target(false)
        .with_level(true)
        .compact()
        .with_filter(EnvFilter::from_default_env());

    // All logs go through audit_layer before reaching the stdout_layer
    let subscriber = Registry::default().with(stdout_layer).with(audit_layer);
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    // Example usage
    info!(audit = true, "This is an audit event");
    info!(audit = false, "This is not an audit event");

    for i in 0..100 {
        sleep(Duration::from_secs(1)).await;
        if i % 25 == 0 && i != 0 {
            error!(audit = true, "This is an audittable error log");
        } else if i % 5 == 0 {
            error!(audit = false, "This is a non auditable error log");
        } else if i % 3 == 0 {
            warn!("This is a regular log without audit field");
        } else {
            trace!(audit = true, "This probably won't get audited or logged");
        }
    }
}
