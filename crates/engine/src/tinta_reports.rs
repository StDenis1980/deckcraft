//! Tinta Office: automatic error reports, sent only with the person's consent.
//!
//! The desktop app asks once, at the first start of a build that can send reports, and the answer
//! is kept here ([`ReportsPrefs::consent`]); File › Options (Preferences) changes it. This module only
//! keeps the choice: the desktop app (`apps/deckcraft/src/tinta_report.rs`) writes, scrubs and
//! sends the reports to report.tintaoffice.com, makes the random installation id, and deletes the
//! server's copies when asked ([`ReportsPrefs::forget_pending`]). Builds without a report address
//! never ask and never send.

use serde::{Deserialize, Serialize};
use serde_json::{Value as Json, json};

use crate::cmd::{CommandSpec, always, cmd};
use crate::{EngineError, Result, Session};

/// The saved choice.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReportsPrefs {
    /// `None` until the person answered the first-start question.
    pub consent: Option<bool>,
    /// The random id reports are filed under on the server (36 characters, made by the desktop
    /// app when reports are first allowed). Not tied to the person.
    pub install_id: String,
    /// An id whose reports the server should delete; the desktop app clears it once the server
    /// confirmed, so a deletion asked for offline happens at the next start.
    pub forget_pending: Option<String>,
}

/// A valid installation id: the lowercase 8-4-4-4-12 hex form.
pub fn id_ok(id: &str) -> bool {
    let groups: Vec<&str> = id.split('-').collect();
    groups.len() == 5
        && groups.iter().zip([8, 4, 4, 4, 12]).all(|(g, n)| g.len() == n && g.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
}

/// A damaged saved value reads as "not asked yet"; malformed ids are dropped.
pub fn lenient<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<ReportsPrefs, D::Error> {
    let value = Json::deserialize(d)?;
    let mut p: ReportsPrefs = serde_json::from_value(value).unwrap_or_default();
    if !id_ok(&p.install_id) {
        p.install_id.clear();
    }
    p.forget_pending = p.forget_pending.filter(|id| id_ok(id));
    Ok(p)
}

fn bad(msg: &str) -> EngineError {
    EngineError::BadParams { cmd: "file.errorReports".into(), msg: msg.into() }
}

fn error_reports(s: &mut Session, p: &Json) -> Result<Json> {
    match p.get("value") {
        None | Some(Json::Null) => {}
        Some(Json::Bool(b)) => s.prefs.tinta_reports.consent = Some(*b),
        Some(_) => return Err(bad("`value` must be true or false")),
    }
    match p.get("forget") {
        None | Some(Json::Null) | Some(Json::Bool(false)) => {}
        // The server deletes the old id's reports; later reports go under a new id.
        Some(Json::Bool(true)) => {
            let r = &mut s.prefs.tinta_reports;
            if !r.install_id.is_empty() {
                r.forget_pending = Some(std::mem::take(&mut r.install_id));
            }
        }
        Some(_) => return Err(bad("`forget` must be true or false")),
    }
    let r = &s.prefs.tinta_reports;
    Ok(json!({"value": r.consent, "installId": r.install_id, "forgetPending": r.forget_pending.is_some()}))
}

pub fn specs() -> Vec<CommandSpec> {
    vec![cmd!(
        query "file.errorReports",
        "Error Reports",
        ["File", "Options"],
        None,
        "{value?: bool (send reports automatically), forget?: true (delete this installation's reports from the server)} → {value: bool|null, installId, forgetPending}",
        always,
        error_reports
    )]
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn not_asked_until_answered_and_the_answer_is_kept() {
        let mut s = Session::new();
        assert_eq!(s.prefs.tinta_reports.consent, None);
        let v = s.execute("file.errorReports", &json!({"value": true})).unwrap();
        assert_eq!(v["value"], json!(true));
        s.execute("file.errorReports", &json!({"value": false})).unwrap();
        assert_eq!(s.prefs.tinta_reports.consent, Some(false));
        assert!(s.execute("file.errorReports", &json!({"value": "yes"})).is_err());
        assert!(s.execute("file.errorReports", &json!({"forget": 1})).is_err());
    }

    #[test]
    fn forget_moves_the_id_to_the_pending_deletion() {
        let mut s = Session::new();
        let id = "0f8fad5b-d9cb-469f-a165-70867728950e";
        s.prefs.tinta_reports.install_id = id.into();
        let v = s.execute("file.errorReports", &json!({"forget": true})).unwrap();
        assert_eq!(v["forgetPending"], json!(true));
        assert_eq!(s.prefs.tinta_reports.forget_pending.as_deref(), Some(id));
        assert!(s.prefs.tinta_reports.install_id.is_empty());
        // Nothing to forget without an id.
        let mut s = Session::new();
        s.execute("file.errorReports", &json!({"forget": true})).unwrap();
        assert_eq!(s.prefs.tinta_reports.forget_pending, None);
    }

    #[test]
    fn damaged_saved_values_read_as_not_asked() {
        let p: crate::Prefs = serde_json::from_value(json!({"tintaReports": {"consent": "maybe"}})).unwrap();
        assert_eq!(p.tinta_reports, ReportsPrefs::default());
        let p: crate::Prefs =
            serde_json::from_value(json!({"tintaReports": {"consent": true, "install_id": "../../etc", "forget_pending": "x"}})).unwrap();
        assert_eq!(p.tinta_reports, ReportsPrefs { consent: Some(true), ..Default::default() });
        assert!(id_ok("0f8fad5b-d9cb-469f-a165-70867728950e"));
        assert!(!id_ok("0F8FAD5B-d9cb-469f-a165-70867728950e"));
    }
}
