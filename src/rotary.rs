use std::process::Command;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use openaction::{Instance, InstanceId, OpenActionResult, async_trait};
use serde::{Deserialize, Serialize};

const LONG_PRESS_THRESHOLD: Duration = Duration::from_millis(500);

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct RotarySettings {
	/// Shell command to run on clockwise rotation.
	pub cw_command: String,
	/// Shell command to run on counter-clockwise rotation.
	pub ccw_command: String,
	/// Shell command to run on short press.
	pub press_command: String,
	/// Shell command to run on long press.
	pub long_press_command: String,
}

impl Default for RotarySettings {
	fn default() -> Self {
		Self {
			cw_command: String::new(),
			ccw_command: String::new(),
			press_command: String::new(),
			long_press_command: String::new(),
		}
	}
}

pub struct RotaryAction {
	press_starts: DashMap<InstanceId, Instant>,
}

impl RotaryAction {
	pub fn new() -> Self {
		Self {
			press_starts: DashMap::new(),
		}
	}
}

fn run_command(label: &str, cmd: &str) {
	if cmd.is_empty() {
		return;
	}
	log::debug!("{label}: running command: {cmd}");
	let cmd = cmd.to_string();
	let label = label.to_string();
	tokio::spawn(async move {
		let label2 = label.clone();
		let result = tokio::task::spawn_blocking(move || {
			Command::new("/bin/sh").args(["-c", &cmd]).output()
		})
		.await;
		match result {
			Ok(Ok(output)) => {
				if !output.status.success() {
					let stderr = String::from_utf8_lossy(&output.stderr);
					log::error!("{label2} command failed: {stderr}");
				}
			}
			Ok(Err(e)) => log::error!("{label2} command error: {e}"),
			Err(e) => log::error!("{label2} task join error: {e}"),
		}
	});
}

#[async_trait]
impl openaction::Action for RotaryAction {
	const UUID: &'static str = "info.degois.damien.opendeck.plugins.rotary-command";
	type Settings = RotarySettings;

	async fn will_appear(
		&self,
		_instance: &Instance,
		_settings: &Self::Settings,
	) -> OpenActionResult<()> {
		Ok(())
	}

	async fn dial_rotate(
		&self,
		_instance: &Instance,
		settings: &Self::Settings,
		ticks: i16,
		_pressed: bool,
	) -> OpenActionResult<()> {
		// Run the appropriate command once per event (ticks sign gives direction)
		if ticks > 0 {
			run_command("cw", &settings.cw_command);
		} else if ticks < 0 {
			run_command("ccw", &settings.ccw_command);
		}
		Ok(())
	}

	async fn dial_down(
		&self,
		instance: &Instance,
		_settings: &Self::Settings,
	) -> OpenActionResult<()> {
		self.press_starts
			.insert(instance.instance_id.clone(), Instant::now());
		Ok(())
	}

	async fn dial_up(
		&self,
		instance: &Instance,
		settings: &Self::Settings,
	) -> OpenActionResult<()> {
		let elapsed = self
			.press_starts
			.remove(&instance.instance_id)
			.map(|(_, start)| start.elapsed())
			.unwrap_or(Duration::ZERO);

		if elapsed >= LONG_PRESS_THRESHOLD {
			run_command("long_press", &settings.long_press_command);
		} else {
			run_command("press", &settings.press_command);
		}
		Ok(())
	}

	async fn did_receive_settings(
		&self,
		_instance: &Instance,
		_settings: &Self::Settings,
	) -> OpenActionResult<()> {
		Ok(())
	}
}
