use crate::constants::CHECK_INTERVAL_MINUTES;
use crate::constants::STATE_FILE;
use crate::modules::api::{self, DetailActivity};
use crate::modules::auth;
use chrono::Local;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::thread;
use std::time::Duration;

// --- STATE MANAGEMENT ---
#[derive(Serialize, Deserialize, Debug, Clone)]
struct CourseState {
    total: Option<f64>,
    note: Option<String>,
}

type GradesState = HashMap<String, CourseState>; // Key: Course Sigle (e.g., "INF3173")

pub async fn start_daemon() {
    // 1. Load Interval from Env (Default: 60 min)
    let interval_min: u64 = env::var("CHECK_INTERVAL")
        .unwrap_or_else(|_| "60".to_string())
        .parse()
        .unwrap_or(60);

    println!("🚀 Starting UQGRD Daemon...");
    println!("   Interval: Every {} minutes", interval_min);

    loop {
        println!(
            "Checking grades at {}",
            Local::now().format("%Y-%m-%d %H:%M:%S")
        );

        if let Err(e) = check_and_notify().await {
            eprintln!("❌ Error during check cycle: {}", e);
        }

        // Sleep using the dynamic variable
        thread::sleep(Duration::from_secs(interval_min * 60));
    }
}

async fn check_and_notify() -> Result<(), String> {
    // 1. Load Credentials (API) - This will fail if not configured on the server
    let (username, password) = auth::get_credentials()?;

    // 2. Load Saved State (Previous Grades)
    let mut state = load_state()?;
    let mut state_changed = false;

    // 3. Authenticate
    let token = api::get_token(&username, &password).await?;

    // 4. Fetch Current Semester
    let current_sem_code = api::get_current_semester_code();
    let transcript = api::fetch_transcript(&token).await?;

    // Find the current semester
    if let Some(sem) = transcript.iter().find(|s| s.trimestre == current_sem_code) {
        for prog in &sem.programmes {
            for activity in &prog.activites {
                // Fetch live details
                let details = api::fetch_course_details(
                    &token,
                    sem.trimestre,
                    &activity.sigle,
                    activity.groupe,
                )
                .await;

                match details {
                    Ok(new_data) => {
                        // Check if grade changed
                        if has_grade_changed(&state, &activity.sigle, &new_data) {
                            println!(
                                "🔔 CHANGE DETECTED: {} ({})",
                                activity.sigle, activity.titre
                            );

                            // Send Alert
                            if let Err(e) = send_email_alert(
                                &username,
                                &activity.sigle,
                                &activity.titre,
                                &new_data,
                            ) {
                                eprintln!("   Failed to send email: {}", e);
                            } else {
                                println!("   📧 Email sent successfully!");
                            }

                            // Update State
                            state.insert(
                                activity.sigle.clone(),
                                CourseState {
                                    total: new_data.total,
                                    note: new_data.note,
                                },
                            );
                            state_changed = true;
                        }
                    }
                    Err(e) => eprintln!("   Failed to fetch details for {}: {}", activity.sigle, e),
                }
            }
        }
    } else {
        println!(
            "   No active semester found for code {} (Are you registered?)",
            current_sem_code
        );
    }

    // 5. Save new state
    if state_changed {
        save_state(&state)?;
    }

    Ok(())
}

// --- LOGIC HELPERS ---

fn has_grade_changed(state: &GradesState, sigle: &str, new_data: &DetailActivity) -> bool {
    match state.get(sigle) {
        Some(old_data) => {
            // Compare Total (Float tolerance)
            let total_diff = match (old_data.total, new_data.total) {
                (Some(a), Some(b)) => (a - b).abs() > 0.01,
                (None, None) => false,
                _ => true, // One is null, one is not
            };

            // Compare Note
            let note_diff = old_data.note != new_data.note;

            total_diff || note_diff
        }
        None => {
            // New course found.
            // If it has data (not null), trigger initial alert.
            new_data.total.is_some() || new_data.note.is_some()
        }
    }
}

fn load_state() -> Result<GradesState, String> {
    let config_dir = auth::get_config_dir()?;
    let state_path = config_dir.join(STATE_FILE);

    if !state_path.exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(state_path).map_err(|e| e.to_string())?;
    let state: GradesState = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(state)
}

fn save_state(state: &GradesState) -> Result<(), String> {
    let config_dir = auth::get_config_dir()?;
    let state_path = config_dir.join(STATE_FILE);

    let json = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(state_path, json).map_err(|e| e.to_string())?;
    Ok(())
}

// --- EMAILER ---

fn send_email_alert(
    username: &str,
    sigle: &str,
    title: &str,
    data: &DetailActivity,
) -> Result<(), String> {
    // Load SMTP settings from Env Vars (Standard for Docker/Cloud)
    let smtp_user = std::env::var("SMTP_USERNAME").map_err(|_| "SMTP_USERNAME env missing")?;
    let smtp_pass = std::env::var("SMTP_PASSWORD").map_err(|_| "SMTP_PASSWORD env missing")?;
    let smtp_host = std::env::var("SMTP_SERVER").unwrap_or_else(|_| "smtp.gmail.com".to_string());

    let dest_email = format!("{}@uqam.ca", username);

    let grade_display = data.note.clone().unwrap_or("N/A".to_string());
    let total_display = match data.total {
        Some(v) => format!("{:.2}%", v),
        None => "N/A".to_string(),
    };

    let email = Message::builder()
        .from(smtp_user.parse().map_err(|_| "Invalid sender")?)
        .to(dest_email.parse().map_err(|_| "Invalid recipient")?)
        .subject(format!("UQAM Grade Update: {}", sigle))
        .body(format!(
            "New grade detected!\n\nCourse: {} - {}\nGrade: {}\nTotal: {}\n\nCheck here: https://monportail.uqam.ca",
            sigle, title, grade_display, total_display
        ))
        .map_err(|e| e.to_string())?;

    let creds = Credentials::new(smtp_user, smtp_pass);

    // Using relay() allows automatic upgrading to TLS (StartTLS) on port 587
    let mailer = SmtpTransport::relay(&smtp_host)
        .map_err(|e| e.to_string())?
        .credentials(creds)
        .build();

    mailer.send(&email).map_err(|e| e.to_string())?;

    Ok(())
}
