pub mod ai;
pub mod database;
pub mod desktop;
pub mod launcher;
pub mod media;
pub mod mcp;
pub mod paste;
pub mod session;
pub mod square;
pub mod square_page;
pub mod updates;

pub mod skills;

pub(super) fn ensure_authorized(status: reqwest::StatusCode) -> Result<(), String> {
    if status == reqwest::StatusCode::UNAUTHORIZED { Err("SESSION_EXPIRED".into()) } else { Ok(()) }
}
