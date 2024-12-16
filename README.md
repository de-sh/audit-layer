# AuditLayer

`AuditLayer` is a Rust library providing a structured logging layer that integrates with the `tracing` ecosystem. This layer captures auditable events and sends them to a specified HTTP endpoint, supporting centralized logging and monitoring systems.
Features
- Seamless integration with `tracing`: Automatically logs auditable events from your application's spans and events.
- HTTP Delivery: Pushes logs over HTTP using the `reqwest` client.
- Flexible Configuration: Easily specify log endpoint, credentials, and runtime configuration.
- Structured Data: Captures and sends event metadata as JSON.

## Usage
Creating a subscriber with `AuditLayer`

To use AuditLayer, initialize it with the following parameters:
- log_endpoint - The HTTP endpoint where logs will be sent.
- username - Username for basic authentication.
- password - Password for basic authentication.
- runtime_handle - A Tokio runtime handle for spawning the HTTP delivery task.

```rust
use audit_layer::AuditLayer;
use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt;
use tokio::runtime::Handle;

// Get handle to the current tokio runtime,
// NOTE: ensure this is done from within a tokio runtime else it will fail
let runtime_handle = Handle::current();
let audit_layer = AuditLayer::new(
    "https://logger-endpoint".to_string(),
    "logger-username".to_string(),
    "logger-password".to_string(),
    runtime_handle,
);

let subscriber = Registry::default().with(audit_layer);
tracing::subscriber::set_global_default(subscriber)
    .expect("Failed to set global subscriber");
```

After the subscriber has been setup, you will be able to witness its working as follows(logs should be received by the HTTP server):
```rust
use tracing::info;

info!(
    audit = true, // Marks this event as auditable
    message = "User login attempt",
    user_id = 1234,
    success = true,
    "Audit log example"
);
```
