use zero2prod::configuration::get_configurations;
use zero2prod::startup::Application;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configurations().expect("Failed to read configuration");
    let server = Application::build(configuration).await?;
    server.run_until_stopped().await?;
    Ok(())
}
