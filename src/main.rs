mod cli;
mod constants;
mod modules;

use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Credentials { skip_encryption } => {
            match modules::interact::prompt_credentials() {
                Ok((username, password)) => {
                    // Save credentials with the optional encryption flag
                    if let Err(e) =
                        modules::auth::save_credentials(&username, &password, skip_encryption)
                    {
                        eprintln!("Error saving credentials: {}", e);
                    }
                }
                Err(e) => eprintln!("Error getting input: {}", e),
            }
        }
        Commands::Grades { current } => {
            // 1. Credentials
            let (username, password) = match modules::auth::get_credentials() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("❌ {}", e);
                    return;
                }
            };

            // 2. Auth
            let token = match modules::api::get_token(&username, &password).await {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("❌ {}", e);
                    return;
                }
            };

            // 3. Fetch Transcript
            let transcript = match modules::api::fetch_transcript(&token).await {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("❌ {}", e);
                    return;
                }
            };

            // 4. Select Semester (Auto vs Manual)
            let selected_pair = if current {
                // LOGIC: Calculate current code
                let current_code = modules::api::get_current_semester_code();
                let sem_name = modules::api::format_semester_name(current_code);

                println!("📅 Date detected: {}", sem_name);

                // Try to find exact match
                let match_found = transcript.iter().find(|s| s.trimestre == current_code);

                match match_found {
                    Some(sem) => {
                        println!("✅ Found current semester in transcript.");
                        // Default to the first program in that semester
                        sem.programmes.first().map(|p| (sem, p))
                    }
                    None => {
                        println!(
                            "⚠️  Current semester ({}) not found in transcript.",
                            sem_name
                        );
                        println!("👉 Falling back to latest available semester.");
                        // Fallback to the first one (Latest, assuming sorted)
                        transcript
                            .first()
                            .and_then(|sem| sem.programmes.first().map(|p| (sem, p)))
                    }
                }
            } else {
                // Manual selection
                match modules::interact::select_semester(&transcript) {
                    Ok(opt) => opt,
                    Err(e) => {
                        eprintln!("❌ {}", e);
                        return;
                    }
                }
            };

            // 5. Display Grades (Common logic)
            if let Some((sem_result, program)) = selected_pair {
                println!(
                    "\n📖 Grades for: {} - {}\n",
                    modules::api::format_semester_name(sem_result.trimestre),
                    program.titre_programme
                );

                println!(
                    "{:<10} | {:<40} | {:<10} | {:<5}",
                    "Sigle", "Title", "Total (%)", "Grade"
                );
                println!("{:-<10}-|-{:-<40}-|-{:-<10}-|-{:-<5}", "", "", "", "");

                for activity in &program.activites {
                    let details = modules::api::fetch_course_details(
                        &token,
                        sem_result.trimestre,
                        &activity.sigle,
                        activity.groupe,
                    )
                    .await;

                    match details {
                        Ok(det) => {
                            let total_str = match det.total {
                                Some(val) => format!("{:.2}%", val),
                                None => "N/A".to_string(),
                            };
                            let note_str = det.note.unwrap_or_else(|| "N/A".to_string());
                            println!(
                                "{:<10} | {:<40} | {:<10} | {:<5}",
                                activity.sigle,
                                activity.titre.chars().take(40).collect::<String>(),
                                total_str,
                                note_str
                            );
                        }
                        Err(_) => {
                            println!(
                                "{:<10} | {:<40} | {:<10} | {:<5}",
                                activity.sigle, activity.titre, "ERROR", "---"
                            );
                        }
                    }
                }
                println!("\n");
            } else {
                println!("No selection made.");
            }
        }
        Commands::Start => {
            modules::daemon::start_daemon().await;
        }
    }
}
