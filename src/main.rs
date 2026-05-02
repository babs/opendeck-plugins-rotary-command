mod rotary;

use openaction::{register_action, run};

#[tokio::main]
async fn main() {
	// Honor RUST_LOG as a plain level (off|error|warn|info|debug|trace); the
	// README documents this knob for command-echo debug logging.
	let level = std::env::var("RUST_LOG")
		.ok()
		.and_then(|s| s.parse::<log::LevelFilter>().ok())
		.unwrap_or(log::LevelFilter::Info);
	if let Err(e) = simplelog::TermLogger::init(
		level,
		simplelog::Config::default(),
		simplelog::TerminalMode::Stdout,
		simplelog::ColorChoice::Never,
	) {
		eprintln!("logger init failed: {e}");
	}

	register_action(rotary::RotaryAction::new()).await;
	if let Err(e) = run(std::env::args().collect()).await {
		log::error!("plugin exited with error: {e}");
		std::process::exit(1);
	}
}
