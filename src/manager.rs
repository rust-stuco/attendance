use crate::Student;
use crate::cli::Command;
use anyhow::{Result, bail};
use config::Config;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const COLOR_RESET: &str = "\x1b[0m";
const COLOR_CURRENT_WEEK: &str = "\x1b[1;32m"; // bright green

type AndrewId = String;
type Roster = HashMap<AndrewId, Student>;

pub struct AttendanceManager {
    roster_path: PathBuf,
    weekly_data_path: PathBuf,
    roster: Roster,
    weekly_data: WeeklyDataFile,
}

#[derive(Debug, Serialize, Deserialize)]
struct WeeklyDataFile {
    current_week: u8,
    weekly_data: HashMap<u8, WeeklyData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WeeklyData {
    week: u8,
    excused: HashSet<AndrewId>,
    attended: HashSet<AndrewId>,
}

impl WeeklyData {
    fn new(week: u8) -> Self {
        Self {
            week,
            excused: HashSet::new(),
            attended: HashSet::new(),
        }
    }

    fn get_unexcused_absences<'a>(&'a self, roster: &'a Roster) -> Vec<&'a AndrewId> {
        roster
            .keys()
            .filter(|id| !self.excused.contains(*id) && !self.attended.contains(*id))
            .collect()
    }
}

impl AttendanceManager {
    pub fn new(
        roster_path: &impl AsRef<Path>,
        weekly_data_path: &impl AsRef<Path>,
    ) -> Result<Self> {
        if !roster_path.as_ref().exists() {
            let empty_roster: Roster = HashMap::new();
            let json = serde_json::to_string_pretty(&empty_roster)?;
            let mut file = File::create(roster_path)?;
            file.write_all(json.as_bytes())?;
        }

        if !weekly_data_path.as_ref().exists() {
            let empty_data = WeeklyDataFile {
                current_week: 1,
                weekly_data: HashMap::new(),
            };
            let json = serde_json::to_string_pretty(&empty_data)?;
            let mut file = File::create(weekly_data_path)?;
            file.write_all(json.as_bytes())?;
        }

        // Load data
        let mut file = File::open(roster_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let roster: Roster = serde_json::from_str(&contents)?;

        let mut file = File::open(weekly_data_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let weekly_data: WeeklyDataFile = serde_json::from_str(&contents)?;

        Ok(Self {
            roster_path: roster_path.as_ref().to_owned(),
            weekly_data_path: weekly_data_path.as_ref().to_owned(),
            roster,
            weekly_data,
        })
    }

    /// Create a new `AttendanceManager` with default configuration from `config.toml`.
    pub fn default_config_manager() -> Result<Self> {
        // Load configuration from `config.toml`.
        let settings = Config::builder()
            .add_source(config::File::with_name("config"))
            .build()?;

        let roster_path = settings.get_string("attendance_manager.roster_path")?;
        let weekly_data_path = settings.get_string("attendance_manager.weekly_data_path")?;

        AttendanceManager::new(&roster_path, &weekly_data_path)
    }

    pub fn run_command(&mut self, command: &Command) -> Result<()> {
        match command {
            Command::AddStudent(student) => {
                self.add_student(student)?;
                println!("Student {} added successfully.", student.andrew_id);
            }
            Command::RemoveStudent(student) => {
                self.remove_student(&student.andrew_id)?;
                println!("Student {} removed successfully.", student.andrew_id);
            }
            Command::MarkExcused(student) => {
                self.mark_excused(&student.andrew_id)?;
                println!("Student {} marked as excused.", student.andrew_id);
            }
            Command::MarkAttended(student) => {
                self.mark_attended(&student.andrew_id)?;
                println!("Student {} marked as attended.", student.andrew_id);
            }
            Command::BulkMarkAttended { file_path } => {
                self.bulk_mark_attended(&file_path)?;
                println!("Bulk attendance marked successfully.");
            }
            Command::ListUnexcused => {
                let unexcused = self.get_unexcused_absentees();
                println!("Unexcused absentees:");
                for (id, student) in unexcused {
                    println!("{}: {:?}", id, student.name);
                }
            }
            Command::EmailUnexcused => {
                self.email_unexcused_absentees()?;
            }
            Command::ShowWeek => {}
            Command::BumpWeek => {
                self.bump_week()?;
                println!("Week bumped successfully.");
            }
            Command::ResetWeek => {
                self.reset_weekly_data()?;
                println!("Weekly data reset successfully.");
            }
            Command::SetWeek { week } => {
                match self.set_current_week(*week) {
                    Ok(_) => println!("Set current week to {}", week),
                    Err(e) => eprintln!("Error: {}", e),
                };
            }
            Command::AggregateStats => {
                let counts = self.aggregate_unexcused();
                println!("Unexcused absences:");
                let mut sorted: Vec<_> = counts.iter().collect();
                // Sort by highest count first, then alphabetically by `andrew_id`.
                sorted.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));

                for (andrew_id, count) in sorted {
                    println!("- {}: {} absences", andrew_id, count);
                }
            }
            Command::FlagAtRisk => {
                let (_, warnings) = self.aggregate_unexcused_with_warning(2);
                if !warnings.is_empty() {
                    println!("Students at risk:");
                    for (andrew_id, count) in warnings {
                        println!("! {} has {} unexcused absences", andrew_id, count);
                    }
                }
            }
        }

        self.print_weekly_summary();

        Ok(())
    }

    fn save_roster(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.roster)?;
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true) // To overwrite the file content
            .create(true)
            .open(&self.roster_path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    fn save_weekly_data(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.weekly_data)?;
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true) // To overwrite the file content
            .create(true)
            .open(&self.weekly_data_path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn add_student(&mut self, student: &Student) -> Result<()> {
        self.roster
            .insert(student.andrew_id.clone(), student.clone());
        self.save_roster()?;
        Ok(())
    }

    pub fn remove_student(&mut self, andrew_id: &str) -> Result<()> {
        self.roster.remove(andrew_id);
        self.save_roster()?;
        Ok(())
    }

    pub fn mark_excused(&mut self, andrew_id: &str) -> Result<()> {
        if self.roster.contains_key(andrew_id) {
            let current_week = self.weekly_data.current_week;
            let weekly_data = self
                .weekly_data
                .weekly_data
                .entry(current_week)
                .or_insert_with(|| WeeklyData::new(current_week));
            weekly_data.excused.insert(andrew_id.to_string());
            self.save_weekly_data()?;
            Ok(())
        } else {
            bail!("Student not found in roster")
        }
    }

    pub fn remove_excused(&mut self, andrew_id: &str) -> Result<()> {
        let current_week = self.weekly_data.current_week;
        if let Some(weekly_data) = self.weekly_data.weekly_data.get_mut(&current_week) {
            weekly_data.excused.remove(andrew_id);
            self.save_weekly_data()?;
        }
        Ok(())
    }

    pub fn mark_attended(&mut self, andrew_id: &str) -> Result<()> {
        if self.roster.contains_key(andrew_id) {
            let current_week = self.weekly_data.current_week;
            let weekly_data = self
                .weekly_data
                .weekly_data
                .entry(current_week)
                .or_insert_with(|| WeeklyData::new(current_week));
            weekly_data.attended.insert(andrew_id.to_string());
            self.save_weekly_data()?;
            Ok(())
        } else {
            bail!("Student not found in roster")
        }
    }

    pub fn remove_attended(&mut self, andrew_id: &str) -> Result<()> {
        let current_week = self.weekly_data.current_week;
        if let Some(weekly_data) = self.weekly_data.weekly_data.get_mut(&current_week) {
            weekly_data.attended.remove(andrew_id);
            self.save_weekly_data()?;
        }
        Ok(())
    }

    pub fn bulk_mark_attended(&mut self, path: &impl AsRef<Path>) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let mut errors = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let andrew_id = if line.contains('@') {
                line.split('@').next().unwrap_or(line).to_string()
            } else {
                line.to_string()
            };

            if let Err(e) = self.mark_attended(&andrew_id) {
                errors.push(format!("Line {}: {} - {}", line_num + 1, andrew_id, e));
            }
        }
        if !errors.is_empty() {
            bail!(
                "Errors occurred during bulk marking:\n{}",
                errors.join("\n")
            );
        }

        Ok(())
    }

    pub fn reset_weekly_data(&mut self) -> Result<()> {
        let current_week = self.weekly_data.current_week;
        self.weekly_data
            .weekly_data
            .insert(current_week, WeeklyData::new(current_week));
        self.save_weekly_data()?;
        Ok(())
    }

    pub fn bump_week(&mut self) -> Result<()> {
        let new_week = (self.weekly_data.weekly_data.len() + 1).try_into()?;
        self.weekly_data
            .weekly_data
            .insert(new_week, WeeklyData::new(new_week));

        self.weekly_data.current_week = new_week;
        self.save_weekly_data()?;

        Ok(())
    }

    pub fn set_current_week(&mut self, new_week: u8) -> Result<()> {
        if !self.weekly_data.weekly_data.contains_key(&new_week) {
            bail!("Nonexistent week");
        }
        if new_week <= self.weekly_data.current_week {
            println!(
                "Warning: You're jumping to an older week {} than our current week {}. Be careful not to override past data!",
                new_week, self.weekly_data.current_week
            );
            print!("Confirm? y/[N]: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() != "y" {
                println!("Week change canceled");
                return Ok(());
            }
        }

        self.weekly_data.current_week = new_week;
        self.save_weekly_data()?;
        Ok(())
    }

    pub fn get_current_week(&self) -> u8 {
        self.weekly_data.current_week
    }

    pub fn get_weekly_summary(&self) -> HashMap<u8, (usize, usize)> {
        self.weekly_data
            .weekly_data
            .iter()
            .map(|(week, data)| (*week, (data.excused.len(), data.attended.len())))
            .collect()
    }

    pub fn get_unexcused_absentees(&self) -> Vec<(&AndrewId, &Student)> {
        let current_week = self.weekly_data.current_week;
        match self.weekly_data.weekly_data.get(&current_week) {
            Some(weekly_data) => {
                let unexcused_ids = weekly_data.get_unexcused_absences(&self.roster);
                unexcused_ids
                    .into_iter()
                    .filter_map(|id| self.roster.get_key_value(id))
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    pub fn email_unexcused_absentees(&self) -> Result<()> {
        let unexcused = self.get_unexcused_absentees();

        let recipient_emails: Vec<String> = unexcused
            .iter()
            .map(|(_, student)| student.email())
            .collect();

        if recipient_emails.is_empty() {
            println!("No unexcused absentees to email.");
            return Ok(());
        }

        println!("Will email the following students: {:?}", recipient_emails);
        print!("Proceed? y/[N]: ");
        io::stdout().flush()?;

        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim();

        if user_input.to_lowercase() != "y" {
            println!("Emailing canceled!");
            return Ok(());
        }

        // mailer::send_mail(&recipient_emails)?;
        todo!();
        // Ok(())
    }

    pub fn aggregate_unexcused(&self) -> HashMap<&AndrewId, u32> {
        let mut counts = HashMap::new();

        // Include all weeks up to but not including current week
        for week in 1..self.weekly_data.current_week {
            if let Some(weekly_data) = self.weekly_data.weekly_data.get(&week) {
                for student_id in self.roster.keys() {
                    let absent = !weekly_data.excused.contains(student_id)
                        && !weekly_data.attended.contains(student_id);

                    if absent {
                        *counts.entry(student_id).or_insert(0) += 1;
                    }
                }
            }
        }

        counts
    }

    pub fn aggregate_unexcused_with_warning(
        &self,
        warning_threshold: u32,
    ) -> (HashMap<&AndrewId, u32>, Vec<(&AndrewId, u32)>) {
        let counts = self.aggregate_unexcused();
        let warnings: Vec<_> = counts
            .iter()
            .filter(|&(_, &count)| count >= warning_threshold)
            .map(|(s, c)| (*s, *c))
            .collect();

        (counts, warnings)
    }

    pub fn print_weekly_summary(&self) {
        let current_week = self.get_current_week();
        let weekly_summary = self.get_weekly_summary();
        println!("\nWeekly Data Summary:");

        let mut weeks: Vec<_> = weekly_summary.iter().collect();
        weeks.sort_by_key(|(k, _)| *k);

        for (week, (excused, attended)) in weeks {
            let is_current = *week == current_week;

            let status = if is_current { " (current)" } else { "" };
            let week_color = if is_current {
                COLOR_CURRENT_WEEK
            } else {
                COLOR_RESET
            };
            println!(
                "{}Week {}{}: {} excused, {} attended",
                week_color, week, status, excused, attended
            );
        }
    }
}
