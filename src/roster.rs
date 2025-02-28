use crate::mailer;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::io;
use std::io::{Read, Write};

type AndrewId = String;
type Roster = HashMap<AndrewId, Student>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Student {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WeeklyData {
    week: u32,
    excused: HashSet<AndrewId>,
    attended: HashSet<AndrewId>,
}

impl WeeklyData {
    fn new(week: u32) -> Self {
        Self {
            week,
            excused: HashSet::new(),
            attended: HashSet::new(),
        }
    }
    
    fn get_unexcused_absences<'a>(&'a self, roster: &'a Roster) -> Vec<&'a AndrewId> {
        roster.keys()
            .filter(|id| !self.excused.contains(*id) && !self.attended.contains(*id))
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct WeeklyDataFile {
    current_week: u32,
    weekly_data: HashMap<u32, WeeklyData>,
}

pub struct AttendanceManager {
    roster_path: String,
    weekly_data_path: String,
    roster: Roster,
    weekly_data: WeeklyDataFile,
}

impl AttendanceManager {
    pub fn new(roster_path: &str, weekly_data_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if !Path::new(roster_path).exists() {
            let empty_roster: Roster = HashMap::new();
            let json = serde_json::to_string_pretty(&empty_roster)?;
            let mut file = File::create(roster_path)?;
            file.write_all(json.as_bytes())?;
        }
        
        if !Path::new(weekly_data_path).exists() {
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
            roster_path: roster_path.to_string(),
            weekly_data_path: weekly_data_path.to_string(),
            roster,
            weekly_data,
        })
    }
    
    fn save_roster(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.roster)?;
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true) // To overwrite the file content
            .create(true)
            .open(&self.roster_path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
    
    fn save_weekly_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.weekly_data)?;
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true) // To overwrite the file content
            .create(true)
            .open(&self.weekly_data_path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
    
    pub fn add_student(&mut self, andrew_id: AndrewId, name: String, email: String) -> Result<(), Box<dyn std::error::Error>> {
        self.roster.insert(andrew_id, Student { name, email });
        self.save_roster()?;
        Ok(())
    }
    
    pub fn remove_student(&mut self, andrew_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.roster.remove(andrew_id);
        self.save_roster()?;
        Ok(())
    }
    
    pub fn mark_excused(&mut self, andrew_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.roster.contains_key(andrew_id) {
            let current_week = self.weekly_data.current_week;
            let weekly_data = self.weekly_data.weekly_data.entry(current_week).or_insert_with(|| WeeklyData::new(current_week));
            weekly_data.excused.insert(andrew_id.to_string());
            self.save_weekly_data()?;
            Ok(())
        } else {
            Err("Student not found in roster".into())
        }
    }
    
    pub fn remove_excused(&mut self, andrew_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let current_week = self.weekly_data.current_week;
        if let Some(weekly_data) = self.weekly_data.weekly_data.get_mut(&current_week) {
            weekly_data.excused.remove(andrew_id);
            self.save_weekly_data()?;
        }
        Ok(())
    }
    
    pub fn mark_attended(&mut self, andrew_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.roster.contains_key(andrew_id) {
            let current_week = self.weekly_data.current_week;
            let weekly_data = self.weekly_data.weekly_data.entry(current_week).or_insert_with(|| WeeklyData::new(current_week));
            weekly_data.attended.insert(andrew_id.to_string());
            self.save_weekly_data()?;
            Ok(())
        } else {
            Err("Student not found in roster".into())
        }
    }
    
    pub fn remove_attended(&mut self, andrew_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let current_week = self.weekly_data.current_week;
        if let Some(weekly_data) = self.weekly_data.weekly_data.get_mut(&current_week) {
            weekly_data.attended.remove(andrew_id);
            self.save_weekly_data()?;
        }
        Ok(())
    }
    
    pub fn reset_weekly_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let current_week = self.weekly_data.current_week;
        self.weekly_data.weekly_data.insert(current_week, WeeklyData::new(current_week));
        self.save_weekly_data()?;
        Ok(())
    }
    
    pub fn bump_week(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.weekly_data.current_week += 1;
        self.save_weekly_data()?;
        Ok(())
    }
    
    pub fn get_unexcused_absentees(&self) -> Vec<(&AndrewId, &Student)> {
        let current_week = self.weekly_data.current_week;
        if let Some(weekly_data) = self.weekly_data.weekly_data.get(&current_week) {
            let unexcused_ids = weekly_data.get_unexcused_absences(&self.roster);
            unexcused_ids
                .into_iter()
                .filter_map(|id| {
                    self.roster.get_key_value(id)
                })
                .collect()
        } else {
            Vec::new()
        }
    }
    
    pub fn email_unexcused_absentees(&self) -> Result<(), Box<dyn std::error::Error>> {
        let unexcused = self.get_unexcused_absentees();
        
        let recipient_emails: Vec<String> = unexcused
            .iter()
            .map(|(_, student)| student.email.clone())
            .collect();
        
        if recipient_emails.is_empty() {
            println!("No unexcused absentees to email.");
            return Ok(());
        }
        
        // TODO add option to cancel before mailing
        println!("Will email the following students: {:?}", recipient_emails);
        print!("Proceed? y/[N]: ");
        io::stdout().flush()?; // Flush to ensure prompt is shown

        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim();

        if user_input.to_lowercase() != "y" {
            println!("Emailing canceled!");
            return Ok(());
        }

        mailer::send_mail(&recipient_emails)?;
        Ok(())
    }
}