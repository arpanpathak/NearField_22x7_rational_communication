/// Binary entry point for the NearField NFC reader.
///
/// This file is intentionally minimal. All logic lives in
/// [`nearfield_22x7_rational_communication::run()`].

fn main() -> nearfield_22x7_rational_communication::AppResult<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    log::info!(
        "NearField_22x7_rational_communication v{}",
        env!("CARGO_PKG_VERSION")
    );

    nearfield_22x7_rational_communication::run()
}
