use crate::modules::api::{Programme, SemesterResult, format_semester_name};
use inquire::{Password, PasswordDisplayMode, Select, Text};
use std::fmt;

// A helper struct to make the Menu Option look pretty
#[derive(Clone)]
pub struct MenuOption {
    pub label: String,
    pub semester_code: u32,
    pub program_index: usize, // Points to which program inside the semester
}

// Tells Inquire how to print this struct in the list
impl fmt::Display for MenuOption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}

pub fn select_semester(
    history: &[SemesterResult],
) -> Result<Option<(&SemesterResult, &Programme)>, String> {
    let mut options = Vec::new();

    // Build the list of choices
    for semester in history {
        let sem_name = format_semester_name(semester.trimestre);

        for (i, prog) in semester.programmes.iter().enumerate() {
            let label = format!("{} - {}", sem_name, prog.titre_programme);
            options.push(MenuOption {
                label,
                semester_code: semester.trimestre,
                program_index: i,
            });
        }
    }

    if options.is_empty() {
        return Err("No semesters found.".to_string());
    }

    let selection = Select::new("Select a semester:", options)
        .prompt()
        .map_err(|e| e.to_string())?;

    // Find the original data based on selection
    // (We search by code to get the reference back)
    for sem in history {
        if sem.trimestre == selection.semester_code {
            return Ok(Some((sem, &sem.programmes[selection.program_index])));
        }
    }

    Ok(None)
}

// We return a Result containing a tuple (username, password) or an error
pub fn prompt_credentials() -> Result<(String, String), String> {
    let username = Text::new("Enter UQAM Username (Code permanent)")
        .prompt()
        .map_err(|e| e.to_string())?;

    let password = Password::new("Enter Password:")
        .without_confirmation()
        .with_display_mode(PasswordDisplayMode::Masked)
        .prompt()
        .map_err(|e| e.to_string())?;

    Ok((username, password))
}
