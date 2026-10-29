//! Opt-in observation of Execute's transport future, never fault injection or recovery policy.
use serde::Serialize;
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};
use sui_rpc::Client;
use tonic::body::Body;
use tower::{Service, util::BoxService};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Rpc = BoxService<http::Request<Body>, http::Response<Body>, Error>;

/// Distinguishes a received Execute transport result from cancellation by SDK recovery.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(tag = "state", content = "error")]
pub enum ExecutionTransportTrace {
    #[default]
    NotStarted,
    AwaitingResponseHeaders,
    HeadersReceived,
    TransportError(String),
    CancelledBeforeTransportResult,
}

#[derive(Clone, Default)]
pub(super) struct Trace(Arc<Mutex<ExecutionTransportTrace>>);
impl Trace {
    pub(super) fn result(&self) -> ExecutionTransportTrace {
        self.0
            .lock()
            .expect("execution trace lock poisoned")
            .clone()
    }
    pub(super) fn attach(self, client: Client) -> Client {
        client.request_layer(tower::layer::layer_fn(move |inner: Rpc| Observed {
            inner,
            trace: self.clone(),
        }))
    }
    fn set(&self, value: ExecutionTransportTrace) {
        *self.0.lock().expect("execution trace lock poisoned") = value;
    }
}

struct Pending {
    trace: Trace,
    completed: bool,
}
impl Pending {
    fn finish(mut self, result: ExecutionTransportTrace) {
        self.trace.set(result);
        self.completed = true;
    }
}
impl Drop for Pending {
    fn drop(&mut self) {
        if !self.completed {
            // Never panic during cancellation/unwinding. No IO or RPC in this guard.
            if let Ok(mut value) = self.trace.0.lock() {
                *value = ExecutionTransportTrace::CancelledBeforeTransportResult;
            }
        }
    }
}
struct Observed {
    inner: Rpc,
    trace: Trace,
}
impl Service<http::Request<Body>> for Observed {
    type Response = http::Response<Body>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        self.inner.poll_ready(cx)
    }
    fn call(&mut self, request: http::Request<Body>) -> Self::Future {
        let execute =
            request.uri().path() == "/sui.rpc.v2.TransactionExecutionService/ExecuteTransaction";
        let future = self.inner.call(request);
        if !execute {
            return future;
        }
        self.trace
            .set(ExecutionTransportTrace::AwaitingResponseHeaders);
        let pending = Pending {
            trace: self.trace.clone(),
            completed: false,
        };
        Box::pin(async move {
            let result = future.await;
            pending.finish(match &result {
                Ok(_) => ExecutionTransportTrace::HeadersReceived,
                Err(error) => ExecutionTransportTrace::TransportError(
                    format!("{error:?}").chars().take(2048).collect(),
                ),
            });
            result
        })
    }
}
