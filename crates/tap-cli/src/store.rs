use anyhow::{Context, Result, ensure};
use rusqlite::{Connection, OptionalExtension, params};
use std::{os::unix::fs::OpenOptionsExt, path::Path};
use tap_core::SignedInvoice;
use tap_sui::Prepared;
mod conversion;
mod live;
pub(crate) use conversion::{ConversionHistory, ConversionRecord};
pub(crate) use live::History;

/// Identify the SQLite library used by this executable without opening any journal.
pub fn sqlite_info() -> Result<serde_json::Value> {
    let connection = Connection::open_in_memory()?;
    let (version, source_id): (String, String) =
        connection.query_row("SELECT sqlite_version(), sqlite_source_id()", [], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
    let mut statement = connection.prepare("PRAGMA compile_options")?;
    let mut options = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    options.sort();
    Ok(serde_json::json!({
        "sqlite_version": version,
        "sqlite_source_id": source_id,
        "compile_options": options,
        "database": "memory",
        "scope": "Library identity only; no file WAL, checkpoint or power-loss qualification"
    }))
}

pub struct Store {
    connection: Connection,
    // Cooperative single-owner exclusion; explicit testnet issuance also has durable reservations.
    _owner: Option<std::fs::File>,
}
impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        // Create private journals. No production encrypted keystore is implemented here.
        let owner = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(path)?;
        owner
            .try_lock()
            .context("state database is already owned or cannot be locked")?;
        let connection = Connection::open(path)?;
        connection.busy_timeout(std::time::Duration::from_secs(5))?;
        connection.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;")?;
        // journal_mode can return an unchanged mode without an SQL error (e.g. unsupported VFS).
        let journal_mode: String =
            connection.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
        let synchronous: i64 = connection.query_row("PRAGMA synchronous", [], |row| row.get(0))?;
        ensure!(
            journal_mode == "wal" && synchronous == 2,
            "state database requires effective WAL journal mode and FULL synchronous"
        );
        connection.execute_batch("CREATE TABLE IF NOT EXISTS demo_invoice (id INTEGER PRIMARY KEY CHECK(id=1), value TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS prepared (invoice_hash TEXT PRIMARY KEY, value TEXT NOT NULL, digest TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS simulated_payments (invoice_hash TEXT PRIMARY KEY, digest TEXT NOT NULL UNIQUE, value TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS simulated_receipts (invoice_hash TEXT PRIMARY KEY, digest TEXT NOT NULL);")?;
        connection.execute_batch("CREATE TABLE IF NOT EXISTS testnet_candidates_v1 (
            invoice_hash TEXT PRIMARY KEY, issuance TEXT NOT NULL UNIQUE, value TEXT NOT NULL CHECK(length(value)<=65536));
            CREATE TABLE IF NOT EXISTS testnet_reservations_v1 (
            invoice_hash TEXT PRIMARY KEY REFERENCES testnet_candidates_v1(invoice_hash), value TEXT NOT NULL CHECK(length(value)<=65536));
            CREATE TABLE IF NOT EXISTS testnet_artifacts_v1 (
            invoice_hash TEXT PRIMARY KEY REFERENCES testnet_reservations_v1(invoice_hash), value TEXT NOT NULL CHECK(length(value)<=65536));
            CREATE TABLE IF NOT EXISTS testnet_recovery_approvals_v1 (
            id INTEGER PRIMARY KEY, invoice_hash TEXT NOT NULL REFERENCES testnet_reservations_v1(invoice_hash), value TEXT NOT NULL CHECK(length(value)<=65536));
            PRAGMA foreign_keys=ON;")?;
        let store = Self {
            connection,
            _owner: Some(owner),
        };
        store.init_live()?;
        store.init_conversion()?;
        Ok(store)
    }
    /// A consistent read-only WAL view for the keyless gateway guard while submit owns the journal.
    /// Caller must validate the private existing file; no initialization or schema writes occur.
    pub(crate) fn open_read_only(path: &Path) -> Result<Self> {
        let connection =
            Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        connection.busy_timeout(std::time::Duration::from_secs(5))?;
        connection.execute_batch("PRAGMA query_only=ON; BEGIN;")?;
        Ok(Self {
            connection,
            _owner: None,
        })
    }
    pub fn load_invoice(&self) -> Result<Option<SignedInvoice>> {
        let s: Option<String> = self
            .connection
            .query_row("SELECT value FROM demo_invoice WHERE id=1", [], |r| {
                r.get(0)
            })
            .optional()?;
        s.map(|s| Ok(serde_json::from_str(&s)?)).transpose()
    }

    pub fn testnet_candidate(&self, hash: &str) -> Result<Option<String>> {
        Ok(self
            .connection
            .query_row(
                "SELECT value FROM testnet_candidates_v1 WHERE invoice_hash=?1",
                [hash],
                |r| r.get(0),
            )
            .optional()?)
    }
    pub fn save_testnet_candidate(&self, hash: &str, issuance: &str, value: &str) -> Result<()> {
        ensure!(value.len() <= 65_536, "candidate exceeds limit");
        self.connection.execute(
            "INSERT INTO testnet_candidates_v1 VALUES (?1,?2,?3)",
            params![hash, issuance, value],
        )?;
        Ok(())
    }
    pub fn testnet_reservation(&self, hash: &str) -> Result<Option<String>> {
        Ok(self
            .connection
            .query_row(
                "SELECT value FROM testnet_reservations_v1 WHERE invoice_hash=?1",
                [hash],
                |r| r.get(0),
            )
            .optional()?)
    }
    pub fn reserve_testnet(&self, hash: &str, approval: &str) -> Result<()> {
        ensure!(approval.len() <= 65_536, "approval exceeds limit");
        // Autocommit with synchronous=FULL completes BEFORE any key loading or signer invocation.
        self.connection.execute(
            "INSERT INTO testnet_reservations_v1 VALUES (?1,?2)",
            params![hash, approval],
        )?;
        Ok(())
    }
    pub fn testnet_artifact(&self, hash: &str) -> Result<Option<Prepared>> {
        let value: Option<String> = self
            .connection
            .query_row(
                "SELECT value FROM testnet_artifacts_v1 WHERE invoice_hash=?1",
                [hash],
                |r| r.get(0),
            )
            .optional()?;
        value
            .map(|s| {
                ensure!(s.len() <= 65_536, "artifact exceeds limit");
                Ok(serde_json::from_str(&s)?)
            })
            .transpose()
    }
    pub fn save_testnet_artifact(&self, hash: &str, prepared: &Prepared) -> Result<()> {
        self.connection.execute(
            "INSERT INTO testnet_artifacts_v1 VALUES (?1,?2)",
            params![hash, serde_json::to_string(prepared)?],
        )?;
        Ok(())
    }
    pub fn approve_testnet_recovery(&self, hash: &str, approval: &str) -> Result<()> {
        ensure!(approval.len() <= 65_536, "approval exceeds limit");
        self.connection.execute(
            "INSERT INTO testnet_recovery_approvals_v1 (invoice_hash,value) VALUES (?1,?2)",
            params![hash, approval],
        )?;
        Ok(())
    }
    pub fn check_testnet_recovery_approvals(
        &self,
        hash: &str,
        mut check: impl FnMut(&str) -> Result<()>,
    ) -> Result<()> {
        let mut statement = self.connection.prepare(
            "SELECT value FROM testnet_recovery_approvals_v1 WHERE invoice_hash=?1 ORDER BY id",
        )?;
        let mut rows = statement.query([hash])?;
        while let Some(row) = rows.next()? {
            // Stream history rather than collecting an unbounded list into memory.
            check(row.get_ref(0)?.as_str()?)?;
        }
        Ok(())
    }
    pub fn save_invoice(&self, i: &SignedInvoice) -> Result<()> {
        self.connection.execute(
            "INSERT INTO demo_invoice VALUES (1,?1)",
            [serde_json::to_string(i)?],
        )?;
        Ok(())
    }
    pub fn load_prepared(&self, h: &str) -> Result<Option<Prepared>> {
        let s: Option<String> = self
            .connection
            .query_row(
                "SELECT value FROM prepared WHERE invoice_hash=?1",
                [h],
                |r| r.get(0),
            )
            .optional()?;
        s.map(|s| Ok(serde_json::from_str(&s)?)).transpose()
    }
    pub fn save_prepared(&self, h: &str, p: &Prepared) -> Result<()> {
        // The first signed transaction wins. Do not replace it after an unknown outcome.
        self.connection.execute(
            "INSERT INTO prepared VALUES (?1,?2,?3)",
            params![h, serde_json::to_string(p)?, p.digest],
        )?;
        Ok(())
    }
    pub fn payment_digest(&self, h: &str) -> Result<Option<String>> {
        Ok(self
            .connection
            .query_row(
                "SELECT digest FROM simulated_payments WHERE invoice_hash=?1",
                [h],
                |r| r.get(0),
            )
            .optional()?)
    }
    pub fn accept_simulated(&self, h: &str, p: &Prepared) -> Result<bool> {
        let tx = self.connection.unchecked_transaction()?;
        let existing: Option<String> = tx
            .query_row(
                "SELECT digest FROM simulated_payments WHERE invoice_hash=?1",
                [h],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(digest) = existing {
            ensure!(
                digest == p.digest,
                "different payment for already recorded invoice requires reconciliation"
            );
            tx.commit()?;
            return Ok(true);
        }
        tx.execute(
            "INSERT INTO simulated_payments VALUES (?1,?2,?3)",
            params![h, p.digest, serde_json::to_string(p)?],
        )?;
        tx.commit()?;
        Ok(false)
    }
    pub fn record_simulated_receipt(&self, h: &str, digest: &str) -> Result<()> {
        self.connection.execute(
            "INSERT OR IGNORE INTO simulated_receipts VALUES (?1,?2)",
            params![h, digest],
        )?;
        let stored: String = self.connection.query_row(
            "SELECT digest FROM simulated_receipts WHERE invoice_hash=?1",
            [h],
            |r| r.get(0),
        )?;
        ensure!(stored == digest, "receipt conflict");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn journal_survives_reopen_and_rejects_replacement() {
        let path =
            std::env::temp_dir().join(format!("linux-tap-test-{}.db", rand::random::<u64>()));
        let i = tap_core::Invoice {
            chain: [42; 32],
            id: [1; 16],
            receiver: [3; 32],
            coin_type: tap_core::USDC.into(),
            amount: 10000,
            created_ms: 1,
            expires_ms: 2,
            merchant: "Test".into(),
        };
        let k = sui_crypto::ed25519::Ed25519PrivateKey::new([9; 32]);
        let p = Prepared::sign(
            &tap_sui::build_direct(&i, k.public_key().derive_address(), 42, 1).unwrap(),
            &k,
        )
        .unwrap();
        {
            let s = Store::open(&path).unwrap();
            s.save_prepared("invoice", &p).unwrap();
            assert!(s.save_prepared("invoice", &p).is_err());
            assert!(!s.accept_simulated("invoice", &p).unwrap());
        }
        {
            let s = Store::open(&path).unwrap();
            assert_eq!(
                s.load_prepared("invoice").unwrap().unwrap().bcs_hex,
                p.bcs_hex
            );
            assert!(s.accept_simulated("invoice", &p).unwrap());
            let mut other = p;
            other.digest = "different".into();
            assert!(s.accept_simulated("invoice", &other).is_err());
        }
        std::fs::remove_file(path).unwrap();
    }
}
