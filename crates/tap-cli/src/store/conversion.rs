//! One immutable conversion operation per journal; separate additive tables from payments.
use super::{Store, params};
use anyhow::{Result, ensure};
use rusqlite::OptionalExtension;

pub(crate) enum ConversionRecord {
    Candidate,
    Reservation,
    Artifact,
    Dispatch,
}
impl ConversionRecord {
    fn name(&self) -> &'static str {
        match self {
            Self::Candidate => "candidate",
            Self::Reservation => "reservation",
            Self::Artifact => "artifact",
            Self::Dispatch => "dispatch",
        }
    }
}
pub(crate) enum ConversionHistory {
    Recovery,
    Attempts,
    Observations,
}
impl ConversionHistory {
    fn name(&self) -> &'static str {
        match self {
            Self::Recovery => "recovery",
            Self::Attempts => "attempt",
            Self::Observations => "observation",
        }
    }
}
impl Store {
    pub(super) fn init_conversion(&self) -> Result<()> {
        self.connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS conversion_records_v1 (
            kind TEXT PRIMARY KEY CHECK(kind IN ('candidate','reservation','artifact','dispatch')),
            value TEXT NOT NULL CHECK(length(value)<=1048576));
            CREATE TABLE IF NOT EXISTS conversion_history_v1 (id INTEGER PRIMARY KEY,
            kind TEXT NOT NULL CHECK(kind IN ('recovery','attempt','observation')),
            value TEXT NOT NULL CHECK(length(value)<=1048576));",
        )?;
        Ok(())
    }
    pub fn conversion_record(&self, kind: ConversionRecord) -> Result<Option<String>> {
        let value: Option<String> = self
            .connection
            .query_row(
                "SELECT value FROM conversion_records_v1 WHERE kind=?1",
                [kind.name()],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(value) = &value {
            ensure!(value.len() <= 1_048_576, "conversion record exceeds limit");
        }
        Ok(value)
    }
    pub fn save_conversion_record(&self, kind: ConversionRecord, value: &str) -> Result<()> {
        ensure!(value.len() <= 1_048_576, "conversion record exceeds limit");
        self.connection.execute(
            "INSERT INTO conversion_records_v1 VALUES (?1,?2)",
            params![kind.name(), value],
        )?;
        Ok(())
    }
    pub fn conversion_history(
        &self,
        kind: ConversionHistory,
        mut check: impl FnMut(i64, &str) -> Result<()>,
    ) -> Result<()> {
        let mut stmt = self
            .connection
            .prepare("SELECT id,value FROM conversion_history_v1 WHERE kind=?1 ORDER BY id")?;
        let mut rows = stmt.query([kind.name()])?;
        while let Some(row) = rows.next()? {
            let value = row.get_ref(1)?.as_str()?;
            ensure!(value.len() <= 1_048_576, "conversion history exceeds limit");
            check(row.get(0)?, value)?;
        }
        Ok(())
    }
    pub fn append_conversion(&self, kind: ConversionHistory, value: &str) -> Result<i64> {
        ensure!(value.len() <= 1_048_576, "conversion history exceeds limit");
        self.connection.execute(
            "INSERT INTO conversion_history_v1(kind,value) VALUES (?1,?2)",
            params![kind.name(), value],
        )?;
        Ok(self.connection.last_insert_rowid())
    }
    pub fn begin_conversion_dispatch(
        &self,
        first_intent: Option<&str>,
        approval: &str,
    ) -> Result<i64> {
        let tx = self.connection.unchecked_transaction()?;
        if let Some(intent) = first_intent {
            tx.execute(
                "INSERT INTO conversion_records_v1 VALUES ('dispatch',?1)",
                [intent],
            )?;
        }
        tx.execute(
            "INSERT INTO conversion_history_v1(kind,value) VALUES ('attempt',?1)",
            [approval],
        )?;
        let id = tx.last_insert_rowid();
        // FULL synchronous commit BEFORE the execution future can be polled.
        tx.commit()?;
        Ok(id)
    }
}
