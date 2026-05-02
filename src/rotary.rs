use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use dashmap::DashMap;
use openaction::{Instance, InstanceId, OpenActionResult, async_trait};
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tokio::task::JoinHandle;

const LONG_PRESS_THRESHOLD: Duration = Duration::from_millis(750);

#[derive(Serialize, Deserialize, Clone, Default)]
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

struct PressState {
	long_press_fired: Arc<AtomicBool>,
	long_press_task: JoinHandle<()>,
}

#[derive(Default)]
pub struct RotaryAction {
	press_states: DashMap<InstanceId, PressState>,
}

impl RotaryAction {
	pub fn new() -> Self {
		Self::default()
	}

	/// Cancel any in-flight long-press timer for `id` and drop its state.
	fn cancel_press(&self, id: &InstanceId) {
		if let Some((_, state)) = self.press_states.remove(id) {
			state.long_press_task.abort();
		}
	}
}

#[cfg(unix)]
fn is_flatpak() -> bool {
	use std::env::var;
	var("FLATPAK_ID").is_ok()
		|| var("container")
			.map(|x| x.to_lowercase().trim() == "flatpak")
			.unwrap_or(false)
}

#[cfg(unix)]
fn path_has(bin: &str) -> bool {
	use std::os::unix::fs::PermissionsExt;
	std::env::var_os("PATH")
		.map(|p| {
			std::env::split_paths(&p).any(|d| {
				std::fs::metadata(d.join(bin))
					.map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
					.unwrap_or(false)
			})
		})
		.unwrap_or(false)
}

#[cfg(unix)]
fn is_distrobox() -> bool {
	// Heuristic shared with OpenDeck's Run Command, but only useful when the
	// host-exec helper is actually reachable; toolbx and plain podman set the
	// same env var without shipping it.
	std::env::var("CONTAINER_ID").is_ok() && path_has("distrobox-host-exec")
}

#[cfg(unix)]
fn build_command(value: &str) -> Command {
	let trimmed = value.trim();
	let mut cmd = if is_flatpak() {
		let mut c = Command::new("flatpak-spawn");
		c.args(["--host", "sh", "-c", value]);
		c
	} else if is_distrobox() && !trimmed.starts_with("distrobox-host-exec") {
		let mut c = Command::new("distrobox-host-exec");
		c.args(["sh", "-c", value]);
		c
	} else {
		let mut c = Command::new("sh");
		c.args(["-c", value]);
		c
	};
	if let Some(home) = std::env::home_dir() {
		cmd.current_dir(home);
	}
	// Null stdin: a child shell reading stdin would steal bytes from our IPC.
	cmd.stdin(Stdio::null());
	cmd
}

#[cfg(windows)]
fn build_command(value: &str) -> Command {
	use std::os::windows::process::CommandExt;
	let mut cmd = Command::new("cmd");
	cmd.arg("/C");
	cmd.raw_arg(value);
	if let Some(home) = std::env::home_dir() {
		cmd.current_dir(home);
	}
	cmd.stdin(Stdio::null());
	cmd
}

fn run_command(label: &str, cmd_str: &str) {
	if cmd_str.trim().is_empty() {
		return;
	}
	log::debug!("{label}: running command: {cmd_str}");
	let cmd_str = cmd_str.to_string();
	let label = label.to_string();
	tokio::spawn(async move {
		match build_command(&cmd_str).output().await {
			Ok(output) => {
				if !output.status.success() {
					let stderr = String::from_utf8_lossy(&output.stderr);
					log::error!("{label} command failed: {stderr}");
				}
			}
			Err(e) => log::error!("{label} command error: {e}"),
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
		// `_pressed` (rotation while held) is not exposed as a separate trigger
		// for parity with OpenDeck's Run Command; combined gestures fire CW/CCW.
		_pressed: bool,
	) -> OpenActionResult<()> {
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
		settings: &Self::Settings,
	) -> OpenActionResult<()> {
		// dial_up races the timer's compare_exchange on long_press_fired —
		// exactly one path wins, and the timer fires on threshold for feedback.
		let id = instance.instance_id.clone();

		// Dropping a JoinHandle does NOT cancel the task; abort prior timer
		// before reinstall, otherwise a duplicate dial_down leaks it.
		self.cancel_press(&id);

		let long_press_cmd = settings.long_press_command.clone();
		let fired = Arc::new(AtomicBool::new(false));
		let fired_timer = fired.clone();
		let task = tokio::spawn(async move {
			tokio::time::sleep(LONG_PRESS_THRESHOLD).await;
			if fired_timer
				.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
				.is_ok()
			{
				run_command("long_press", &long_press_cmd);
			}
		});
		self.press_states.insert(
			id,
			PressState {
				long_press_fired: fired,
				long_press_task: task,
			},
		);
		Ok(())
	}

	async fn dial_up(
		&self,
		instance: &Instance,
		settings: &Self::Settings,
	) -> OpenActionResult<()> {
		if let Some((_, state)) = self.press_states.remove(&instance.instance_id)
			&& state
				.long_press_fired
				.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
				.is_ok()
		{
			state.long_press_task.abort();
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

	async fn will_disappear(
		&self,
		instance: &Instance,
		_settings: &Self::Settings,
	) -> OpenActionResult<()> {
		// Reclaim press state so an orphan timer can't fire against a removed instance.
		self.cancel_press(&instance.instance_id);
		Ok(())
	}
}
