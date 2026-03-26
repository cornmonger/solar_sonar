#[tokio::main]
pub async fn main() -> std::process::ExitCode {
    solar_sonar::run_cli().await
}
