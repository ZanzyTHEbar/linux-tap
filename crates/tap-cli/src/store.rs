use anyhow::{Result, ensure};
use rusqlite::{Connection, OptionalExtension, params};
use std::{os::unix::fs::OpenOptionsExt, path::Path};
use tap_core::SignedInvoice;
use tap_sui::Prepared;

pub struct Store {
    connection: Connection,
}
impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        // Create private journals. No production encrypted keystore is implemented here.
        let _file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(path)?;
        let connection = Connection::open(path)?;
        connection.busy_timeout(std::time::Duration::from_secs(5))?;
        connection.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;
            CREATE TABLE IF NOT EXISTS demo_invoice (id INTEGER PRIMARY KEY CHECK(id=1), value TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS prepared (invoice_hash TEXT PRIMARY KEY, value TEXT NOT NULL, digest TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS simulated_payments (invoice_hash TEXT PRIMARY KEY, digest TEXT NOT NULL UNIQUE, value TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS simulated_receipts (invoice_hash TEXT PRIMARY KEY, digest TEXT NOT NULL);")?;
        Ok(Self { connection })
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
