use crate::manager::AttendanceManager;
use diesel::QueryResult;
use dotenvy::dotenv;
use lettre::{
    SmtpTransport, Transport,
    message::{Message, MultiPart, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use serde::Deserialize;
use std::{
    env,
    io::{self, Write},
};

#[derive(Debug, Deserialize)]
struct EmailConfig {
    smtp_server: String,
    smtp_port: u16,
    from_email: String,
    from_name: String,
}

impl EmailConfig {
    fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // Load .env file first
        dotenv().ok();

        // Load non-sensitive config from TOML
        let config = config::Config::builder()
            .add_source(config::File::with_name("config"))
            .build()?
            .try_deserialize::<EmailConfig>()?;

        Ok(config)
    }

    fn get_credentials() -> Result<Credentials, env::VarError> {
        let username = env::var("SMTP_USERNAME")?;
        let password = env::var("SMTP_PASSWORD")?;
        Ok(Credentials::new(username, password))
    }
}

fn create_mailer(config: &EmailConfig) -> Result<SmtpTransport, Box<dyn std::error::Error>> {
    let creds = EmailConfig::get_credentials()?;

    Ok(SmtpTransport::relay(&config.smtp_server)?
        .port(config.smtp_port)
        .credentials(creds)
        .build())
}

fn send_absence_email(
    mailer: &SmtpTransport,
    config: &EmailConfig,
    student_name: &str,
    student_email: &str,
    absences: usize,
    after_week: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let email = Message::builder()
        .from(format!("{} <{}>", config.from_name, config.from_email).parse()?)
        .to(format!("{} <{}>", student_name, student_email).parse()?)
        .subject("Course Attendance Notice")
        .multipart(
            MultiPart::alternative().singlepart(
                lettre::message::SinglePart::builder()
                    .header(ContentType::TEXT_PLAIN)
                    .body(format!(
                        "Hi {},\n\nBlah blah you have been absent for {} classes since week {}.\n\nBest regards,\n{}",
                        student_name, absences, after_week, config.from_name
                    )),
            ),
        )?;

    mailer.send(&email)?;
    Ok(())
}

/// Emails students who have more than the specified number of absences after a given week.
pub fn email_absentees(after_week: i32, min_absences: i32) -> QueryResult<()> {
    let mut manager = AttendanceManager::connect();
    let roster = manager.get_roster()?;

    // Get all attendance records in one batch using transaction
    let mut absentees = Vec::new();
    for student in roster {
        let attendance = manager.get_student_attendance(&student.id)?;
        let absences = attendance
            .absent
            .iter()
            .filter(|(week, _)| *week >= after_week)
            .count();

        if absences >= min_absences as usize {
            absentees.push((student, absences));
        }
    }

    // Display results
    println!(
        "\nStudents with {} or more absences after week {}:",
        min_absences, after_week
    );
    println!("{:<30} {:<30} {:<10}", "Name", "Email", "Absences");
    println!("{}", "-".repeat(70));

    for (student, absences) in &absentees {
        println!(
            "{:<30} {:<30} {:<10}",
            format!("{} {}", student.first_name, student.last_name),
            student.email,
            absences
        );
    }

    if absentees.is_empty() {
        println!(
            "\nNo students found with {} or more absences after week {}.",
            min_absences, after_week
        );
        return Ok(());
    }

    // Load email configuration
    let config = match EmailConfig::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load email configuration: {}", e);
            return Ok(());
        }
    };

    // Create mailer
    let mailer = match create_mailer(&config) {
        Ok(mailer) => mailer,
        Err(e) => {
            eprintln!("Failed to create mailer: {}", e);
            return Ok(());
        }
    };

    // Ask for confirmation
    print!("\nWould you like to email these students? [y/N] ");
    if let Err(e) = io::stdout().flush() {
        eprintln!("Error flushing stdout: {}", e);
        return Ok(());
    }

    let mut input = String::new();
    if let Err(e) = io::stdin().read_line(&mut input) {
        eprintln!("Error reading input: {}", e);
        return Ok(());
    }

    if input.trim().to_lowercase() != "y" {
        println!("Operation cancelled.");
        return Ok(());
    }

    // Send emails
    println!("\nSending emails...");
    for (student, absences) in absentees {
        let student_name = format!("{} {}", student.first_name, student.last_name);
        match send_absence_email(
            &mailer,
            &config,
            &student_name,
            &student.email,
            absences,
            after_week,
        ) {
            Ok(_) => println!("✓ Sent email to {}", student.email),
            Err(e) => eprintln!("✗ Failed to send email to {}: {}", student.email, e),
        }
    }

    Ok(())
}
