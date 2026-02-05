use chrono::{Datelike, Local};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

const AUTH_ENDPOINT: &str = "https://monportail.uqam.ca/authentification";
const RESUME_ENDPOINT: &str = "https://monportail.uqam.ca/apis/resumeResultat/identifiant";
const DETAIL_ENDPOINT: &str = "https://monportail.uqam.ca/apis/resultatActivite/identifiant";

// --- AUTH STRUCTS ---

#[derive(Deserialize)]
struct AuthResponse {
    token: String,
}

// --- TRANSCRIPT (RESUME) STRUCTS ---

#[derive(Deserialize, Debug, Clone)]
pub struct ResumeResponse {
    pub data: ResumeData,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ResumeData {
    pub resultats: Vec<SemesterResult>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SemesterResult {
    pub trimestre: u32,
    pub programmes: Vec<Programme>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Programme {
    #[serde(rename = "codeProg")]
    #[allow(dead_code)] // Suppress unused warning
    pub code_prog: String,
    #[serde(rename = "titreProgramme")]
    pub titre_programme: String,
    pub activites: Vec<Activity>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Activity {
    #[serde(rename = "sigle")]
    pub sigle: String,
    #[serde(rename = "titreActivite")]
    pub titre: String,
    #[allow(dead_code)] // Suppress unused warning (we use DetailActivity's note)
    pub note: Option<String>,
    #[serde(rename = "groupe")]
    pub groupe: u32,
}

// --- COURSE DETAIL STRUCTS ---

#[derive(Deserialize, Debug)]
pub struct CourseDetailResponse {
    pub data: CourseDetailData,
}

#[derive(Deserialize, Debug)]
pub struct CourseDetailData {
    pub resultats: Vec<DetailResult>,
}

#[derive(Deserialize, Debug)]
pub struct DetailResult {
    pub programmes: Vec<DetailProgramme>,
}

#[derive(Deserialize, Debug)]
pub struct DetailProgramme {
    pub activites: Vec<DetailActivity>,
}

#[derive(Deserialize, Debug)]
pub struct DetailActivity {
    pub total: Option<f64>,
    pub note: Option<String>,
}

// --- API FUNCTIONS ---

pub async fn get_token(username: &str, password: &str) -> Result<String, String> {
    let client = Client::new();
    let payload = json!({ "identifiant": username, "motDePasse": password });

    let response = client
        .post(AUTH_ENDPOINT)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let auth_data = response
        .json::<AuthResponse>()
        .await
        .map_err(|e| format!("Failed to parse auth response: {}", e))?;

    if auth_data.token.is_empty() {
        return Err("Received empty token from server".to_string());
    }

    Ok(auth_data.token)
}

pub async fn fetch_transcript(token: &str) -> Result<Vec<SemesterResult>, String> {
    let client = Client::new();

    let response = client
        .get(RESUME_ENDPOINT)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch transcript: {}", e))?;

    let mut resume: ResumeResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse transcript: {}", e))?;

    // Sort descending (Newest first)
    resume
        .data
        .resultats
        .sort_by(|a, b| b.trimestre.cmp(&a.trimestre));

    Ok(resume.data.resultats)
}

pub async fn fetch_course_details(
    token: &str,
    semester: u32,
    sigle: &str,
    group: u32,
) -> Result<DetailActivity, String> {
    let client = Client::new();
    let url = format!("{}/{}/{}/{}", DETAIL_ENDPOINT, semester, sigle, group);

    let response = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch course {}: {}", sigle, e))?;

    let details: CourseDetailResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse course {}: {}", sigle, e))?;

    if let Some(res) = details.data.resultats.first() {
        if let Some(prog) = res.programmes.first() {
            if let Some(act) = prog.activites.first() {
                return Ok(DetailActivity {
                    total: act.total,
                    note: act.note.clone(),
                });
            }
        }
    }

    Err(format!("Details not found in response for {}", sigle))
}

// --- DATE & SEMESTER LOGIC ---

pub fn get_current_semester_code() -> u32 {
    let now = Local::now();
    let year = now.year();
    let month = now.month();

    match month {
        1..=4 => (year * 10 + 1) as u32, // Hiver (Jan-Apr)
        5..=8 => (year * 10 + 2) as u32, // Été (May-Aug)
        _ => (year * 10 + 3) as u32,     // Automne (Sept-Dec)
    }
}

pub fn format_semester_name(code: u32) -> String {
    let s = code.to_string();
    if s.len() != 5 {
        return s;
    }

    let year_part: i32 = s[0..4].parse().unwrap_or(0);
    let term_part: u32 = s[4..5].parse().unwrap_or(0);

    match term_part {
        1 => format!("Hiver {}", year_part),
        2 => format!("Été {}", year_part),
        3 => format!("Automne {}", year_part),
        _ => s,
    }
}
