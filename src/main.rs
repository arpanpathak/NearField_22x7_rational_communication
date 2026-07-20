/// Binary entry point for the NearField NFC reader.
///
/// # CLI flags
/// - `--scan`                    Minimal mode: polls NFC and prints to terminal (no DB/display).
/// - `--scan --port /dev/...`    Set serial port in scan mode.
/// - `--scan --baud 115200`      Set baud rate in scan mode.
///
/// Without flags, runs the full pipeline (config, DB, display).

fn main() -> nearfield_22x7_rational_communication::AppResult<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--scan") {
        // Scan mode: minimal NFC polling, no DB, no display
        let port = args
            .windows(2)
            .find(|w| w[0] == "--port")
            .map(|w| w[1].clone())
            .or_else(|| std::env::var("NEARFIELD_SERIAL_PORT").ok())
            .unwrap_or_else(|| "/dev/ttyAMA0".into());

        let baud = args
            .windows(2)
            .find(|w| w[0] == "--baud")
            .and_then(|w| w[1].parse::<u32>().ok())
            .or_else(|| std::env::var("NEARFIELD_SERIAL_BAUD").ok()?.parse().ok())
            .unwrap_or(115_200);

        log::info!(
            "NearField_22x7_rational_communication v{} — scan mode",
            env!("CARGO_PKG_VERSION")
        );

        return nearfield_22x7_rational_communication::run_scan(&port, baud);
    }

    log::info!(
        "NearField_22x7_rational_communication v{}",
        env!("CARGO_PKG_VERSION")
    );

    nearfield_22x7_rational_communication::run()
}
