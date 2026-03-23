mod rotary;

use openaction::{register_action, run};

#[tokio::main]
async fn main() {
	if let Err(e) = simplelog::TermLogger::init(
		log::LevelFilter::Info,
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
