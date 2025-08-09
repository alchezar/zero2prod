use tracing::Subscriber;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Registry, fmt::MakeWriter, layer::SubscriberExt};

/// We are using `impl Subscriber` as return type to avoid having to spell out
/// the actual type of the returned subscriber, which is indeed quite complex
/// We need to explicitly call out that the returned subscriber is `Send` and
/// `Sync` to make it possible to pass it to `init_subscriber` later on.
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Sync + Send
where
    Sink: for<'a> MakeWriter<'a> + Sync + Send + 'static,
{
    // We are falling back to printing all spans at info-level or above
    // if the RUST_LOG environment variable hasn't beet set.
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name, sink);

    // The `with` method is provided by `SubscriberExt` and extension
    // that for `Subscriber` exposed by `tracing_subscriber`.
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    subscriber
}

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_subscriber(subscriber: impl Subscriber + Sync + Send) {
    // Redirect all `log`'s events to our subscriber.
    LogTracer::init().expect("Failed to set logger.");

    // This method can be used by applications to specify whet subscriber should
    // be used to process spans.
    set_global_default(subscriber).expect("Failed to set subscriber");
}
