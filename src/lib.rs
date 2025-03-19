use anyhow::Result;
use config::Config;

pub mod cli;
pub mod mailer;
pub mod manager;

use crate::manager::AttendanceManager;

pub fn create_default_manager() -> Result<AttendanceManager> {
    // Load configuration from `config.toml`.
    let settings = Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?;

    let roster_path = settings.get_string("attendance_manager.roster_path")?;
    let weekly_data_path = settings.get_string("attendance_manager.weekly_data_path")?;

    AttendanceManager::new(&roster_path, &weekly_data_path)
}
