//! Additive live-testnet records, separate from every simulator table.
use super::{Store, params};
use anyhow::{Result, ensure};
use rusqlite::OptionalExtension;

pub(crate) enum History {
    Attempts,
    Observations,
    MerchantObservations,
}

impl Store {
    pub(super) fn init_live(&self) -> Result<()> {
        self.connection.execute_batch("CREATE TABLE IF NOT EXISTS testnet_dispatch_v1 (
          invoice_hash TEXT PRIMARY KEY REFERENCES testnet_candidates_v1(invoice_hash), digest TEXT NOT NULL UNIQUE, value TEXT NOT NULL CHECK(length(value)<=1048576));
          CREATE TABLE IF NOT EXISTS testnet_attempts_v1 (id INTEGER PRIMARY KEY, invoice_hash TEXT NOT NULL REFERENCES testnet_dispatch_v1(invoice_hash), value TEXT NOT NULL CHECK(length(value)<=1048576));
          CREATE TABLE IF NOT EXISTS testnet_wallet_observations_v1 (id INTEGER PRIMARY KEY, invoice_hash TEXT NOT NULL REFERENCES testnet_candidates_v1(invoice_hash), value TEXT NOT NULL CHECK(length(value)<=1048576));
          CREATE TABLE IF NOT EXISTS testnet_merchant_config_v1 (id INTEGER PRIMARY KEY CHECK(id=1), value TEXT NOT NULL CHECK(length(value)<=65536));
          CREATE TABLE IF NOT EXISTS testnet_registrations_v1 (invoice_hash TEXT PRIMARY KEY, chain TEXT NOT NULL, receiver TEXT NOT NULL, value TEXT NOT NULL CHECK(length(value)<=65536), UNIQUE(chain,receiver));
          CREATE TABLE IF NOT EXISTS testnet_merchant_observations_v1 (id INTEGER PRIMARY KEY, invoice_hash TEXT NOT NULL REFERENCES testnet_registrations_v1(invoice_hash), value TEXT NOT NULL CHECK(length(value)<=1048576));
          CREATE TABLE IF NOT EXISTS testnet_credits_v1 (invoice_hash TEXT PRIMARY KEY REFERENCES testnet_registrations_v1(invoice_hash), chain TEXT NOT NULL, digest TEXT NOT NULL, value TEXT NOT NULL CHECK(length(value)<=1048576), UNIQUE(chain,digest));
          CREATE TABLE IF NOT EXISTS testnet_confirmations_v1 (invoice_hash TEXT PRIMARY KEY REFERENCES testnet_candidates_v1(invoice_hash), value TEXT NOT NULL CHECK(length(value)<=1048576));")?;
        Ok(())
    }
    fn live_value(&self, sql: &str, hash: &str) -> Result<Option<String>> {
        let value: Option<String> = self
            .connection
            .query_row(sql, [hash], |r| r.get(0))
            .optional()?;
        if let Some(v) = &value {
            ensure!(v.len() <= 1_048_576, "live record exceeds limit");
        }
        Ok(value)
    }
    pub fn live_dispatch(&self, hash: &str, digest: &str) -> Result<Option<String>> {
        let row: Option<(String, String)> = self
            .connection
            .query_row(
                "SELECT digest,value FROM testnet_dispatch_v1 WHERE invoice_hash=?1",
                [hash],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        row.map(|(saved, value)| {
            ensure!(saved == digest, "dispatch SQL digest mismatch");
            Ok(value)
        })
        .transpose()
    }
    pub fn merchant_config(&self) -> Result<Option<String>> {
        self.live_value(
            "SELECT value FROM testnet_merchant_config_v1 WHERE id=1 AND ?1=''",
            "",
        )
    }
    pub fn merchant_registration(
        &self,
        hash: &str,
        chain: &str,
        receiver: &str,
    ) -> Result<Option<String>> {
        let row: Option<(String, String, String)> = self
            .connection
            .query_row(
                "SELECT chain,receiver,value FROM testnet_registrations_v1 WHERE invoice_hash=?1",
                [hash],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        row.map(|(saved_chain, saved_receiver, value)| {
            ensure!(
                saved_chain == chain && saved_receiver == receiver,
                "registration SQL chain/receiver mismatch"
            );
            Ok(value)
        })
        .transpose()
    }
    pub fn merchant_credit(&self, hash: &str, chain: &str, digest: &str) -> Result<Option<String>> {
        let row: Option<(String, String, String)> = self
            .connection
            .query_row(
                "SELECT chain,digest,value FROM testnet_credits_v1 WHERE invoice_hash=?1",
                [hash],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        row.map(|(saved_chain, saved_digest, value)| {
            ensure!(
                saved_chain == chain && saved_digest == digest,
                "credit SQL chain/digest mismatch"
            );
            Ok(value)
        })
        .transpose()
    }
    pub fn wallet_confirmation(&self, hash: &str) -> Result<Option<String>> {
        self.live_value(
            "SELECT value FROM testnet_confirmations_v1 WHERE invoice_hash=?1",
            hash,
        )
    }
    pub fn live_history(
        &self,
        hash: &str,
        history: History,
        mut check: impl FnMut(i64, &str) -> Result<()>,
    ) -> Result<()> {
        let sql = match history {
            History::Attempts => {
                "SELECT id,value FROM testnet_attempts_v1 WHERE invoice_hash=?1 ORDER BY id"
            }
            History::Observations => {
                "SELECT id,value FROM testnet_wallet_observations_v1 WHERE invoice_hash=?1 ORDER BY id"
            }
            History::MerchantObservations => {
                "SELECT id,value FROM testnet_merchant_observations_v1 WHERE invoice_hash=?1 ORDER BY id"
            }
        };
        let mut stmt = self.connection.prepare(sql)?;
        let mut rows = stmt.query([hash])?;
        while let Some(row) = rows.next()? {
            let value = row.get_ref(1)?.as_str()?;
            ensure!(value.len() <= 1_048_576, "live record exceeds limit");
            check(row.get(0)?, value)?;
        }
        Ok(())
    }
    pub fn live_attempt_exists(&self, hash: &str, id: i64) -> Result<bool> {
        Ok(self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM testnet_attempts_v1 WHERE invoice_hash=?1 AND id=?2)",
            params![hash, id],
            |r| r.get(0),
        )?)
    }
    pub fn begin_dispatch(
        &self,
        hash: &str,
        digest: &str,
        first_intent: Option<&str>,
        attempt: &str,
    ) -> Result<i64> {
        let tx = self.connection.unchecked_transaction()?;
        if let Some(intent) = first_intent {
            tx.execute(
                "INSERT INTO testnet_dispatch_v1 VALUES (?1,?2,?3)",
                params![hash, digest, intent],
            )?;
        }
        tx.execute(
            "INSERT INTO testnet_attempts_v1 (invoice_hash,value) VALUES (?1,?2)",
            params![hash, attempt],
        )?;
        let id = tx.last_insert_rowid();
        // Commit is the conservative possibly-sent barrier, BEFORE the execution helper is polled.
        tx.commit()?;
        Ok(id)
    }
    pub fn save_wallet_observation(&self, hash: &str, value: &str) -> Result<()> {
        self.connection.execute(
            "INSERT INTO testnet_wallet_observations_v1 (invoice_hash,value) VALUES (?1,?2)",
            params![hash, value],
        )?;
        Ok(())
    }
    pub fn register_merchant(
        &self,
        hash: &str,
        chain: &str,
        receiver: &str,
        config: &str,
        registration: &str,
    ) -> Result<()> {
        let tx = self.connection.unchecked_transaction()?;
        let prior: Option<String> = tx
            .query_row(
                "SELECT value FROM testnet_merchant_config_v1 WHERE id=1",
                [],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(prior) = prior {
            ensure!(prior == config, "merchant trust namespace is immutable");
        } else {
            tx.execute(
                "INSERT INTO testnet_merchant_config_v1 VALUES (1,?1)",
                [config],
            )?;
        }
        tx.execute(
            "INSERT INTO testnet_registrations_v1 VALUES (?1,?2,?3,?4)",
            params![hash, chain, receiver, registration],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn save_merchant_observation(&self, hash: &str, observation: &str) -> Result<()> {
        self.connection.execute(
            "INSERT INTO testnet_merchant_observations_v1 (invoice_hash,value) VALUES (?1,?2)",
            params![hash, observation],
        )?;
        Ok(())
    }
    pub fn merchant_observation_exists(&self, hash: &str, observation: &str) -> Result<bool> {
        Ok(self.connection.query_row("SELECT EXISTS(SELECT 1 FROM testnet_merchant_observations_v1 WHERE invoice_hash=?1 AND value=?2)", params![hash,observation], |r| r.get(0))?)
    }
    pub fn credit_merchant(
        &self,
        hash: &str,
        chain: &str,
        digest: &str,
        observation: &str,
        receipt: &str,
    ) -> Result<()> {
        let tx = self.connection.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO testnet_merchant_observations_v1 (invoice_hash,value) VALUES (?1,?2)",
            params![hash, observation],
        )?;
        tx.execute(
            "INSERT INTO testnet_credits_v1 VALUES (?1,?2,?3,?4)",
            params![hash, chain, digest, receipt],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn confirm_wallet(&self, hash: &str, observation: &str, receipt: &str) -> Result<()> {
        let tx = self.connection.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO testnet_wallet_observations_v1 (invoice_hash,value) VALUES (?1,?2)",
            params![hash, observation],
        )?;
        tx.execute(
            "INSERT INTO testnet_confirmations_v1 VALUES (?1,?2)",
            params![hash, receipt],
        )?;
        tx.commit()?;
        Ok(())
    }
}
