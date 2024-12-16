use std::{fmt::Debug, sync::Arc};

use reqwest::Client;
use serde_json::{json, Map, Value};
use tokio::runtime::Handle;
use tracing::{
    field::{Field, Visit},
    Event, Metadata, Subscriber,
};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

pub struct AuditLayer {
    client: Arc<Client>,
    username: String,
    password: String,
    log_endpoint: String,
    runtime_handle: Handle,
}

impl AuditLayer {
    /// Create an audit layer that works with the tracing system to capture
    /// and push audit logs to the appropriate logger over HTTP
    pub fn new(
        log_endpoint: String,
        username: String,
        password: String,
        runtime_handle: Handle,
    ) -> Self {
        let client = Arc::new(reqwest::Client::new());

        Self {
            client,
            log_endpoint,
            username,
            password,
            runtime_handle,
        }
    }
}

impl<S> Layer<S> for AuditLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn enabled(&self, _: &Metadata<'_>, _: Context<'_, S>) -> bool {
        true // log everything if it is auditable
    }

    fn on_event(&self, event: &Event<'_>, _: Context<'_, S>) {
        let mut visitor = AuditVisitor::default();
        event.record(&mut visitor);

        if visitor.audit {
            visitor
                .json
                .insert("message".to_owned(), json!(visitor.message));

            let req = self
                .client
                .post(&self.log_endpoint)
                .basic_auth(&self.username, Some(&self.password))
                .json(&visitor.json);

            self.runtime_handle.spawn(async move {
                match req.send().await {
                    Ok(r) => {
                        if let Err(e) = r.error_for_status() {
                            println!("{e}")
                        }
                    }
                    Err(e) => eprintln!("Failed to send audit event: {}", e),
                }
            });
        }
    }
}

#[derive(Debug, Default)]
struct AuditVisitor {
    message: String,
    json: Map<String, Value>,
    audit: bool,
}

impl Visit for AuditVisitor {
    fn record_bool(&mut self, field: &Field, value: bool) {
        if field.name() == "audit" {
            self.audit = value;
        } else {
            self.json.insert(field.name().to_owned(), json!(value));
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_owned();
        } else {
            self.json.insert(field.name().to_owned(), json!(value));
        }
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.json.insert(field.name().to_owned(), json!(value));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.json.insert(field.name().to_owned(), json!(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.json.insert(field.name().to_owned(), json!(value));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        } else {
            self.json
                .insert(field.name().to_owned(), json!(format!("{value:?}")));
        }
    }
}
