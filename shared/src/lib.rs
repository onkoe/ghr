use libghr::Report;
use sqlx::types::Json;
use ts_rs::TS as TypeScript;

/// A type that wraps a `Report` with its time and identifier.
///
/// In other words, this is a row in the database.
#[derive(Clone, Debug, sqlx::FromRow)]
pub struct WrappedReport {
    pub id: uuid::Uuid,
    pub recv_time: chrono::DateTime<chrono::Utc>,
    pub report: Json<Report>,
}

/// A type that wraps a `Report` with its time and identifier.
///
/// Exported to the frontend.
#[derive(Clone, Debug, TypeScript, serde::Serialize)]
#[ts(export)]
pub struct WrappedReportTs {
    pub id: uuid::Uuid,
    pub recv_time: chrono::DateTime<chrono::Utc>,
    pub report: Report,
}

impl From<WrappedReport> for WrappedReportTs {
    fn from(value: WrappedReport) -> Self {
        Self {
            id: value.id,
            recv_time: value.recv_time,
            report: value.report.0,
        }
    }
}

// TODO(bray): `libghr` should have a feature flag to turn off all information
//             gathering to export types more efficiently
