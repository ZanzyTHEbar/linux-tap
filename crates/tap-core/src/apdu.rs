//! Experimental short APDUs; transport acknowledgement never means payment.
use crate::hash;
use anyhow::{Result, ensure};

pub const AID: &[u8] = &[0xf0, 0x4c, 0x54, 0x41, 0x50, 0x00]; // Private laboratory identifier.
pub const MAX_MESSAGE: usize = 8192;
pub const CHUNK: usize = 220;

#[derive(Default)]
pub struct Receiver {
    selected: bool,
    expected: Option<(usize, [u8; 32])>,
    data: Vec<u8>,
    committed: bool,
}
impl Receiver {
    pub fn message(&self) -> Option<&[u8]> {
        self.committed.then_some(self.data.as_slice())
    }
    pub fn process(&mut self, apdu: &[u8]) -> Vec<u8> {
        match self.handle(apdu) {
            Ok(mut b) => {
                b.extend([0x90, 0x00]);
                b
            }
            Err(_) => {
                *self = Self::default();
                vec![0x6a, 0x80]
            }
        }
    }
    fn handle(&mut self, a: &[u8]) -> Result<Vec<u8>> {
        ensure!(
            a.len() >= 5 && a.len() == 5 + a[4] as usize,
            "short APDU length"
        );
        if a[..4] == [0x00, 0xa4, 0x04, 0x00] {
            ensure!(&a[5..] == AID, "unknown application");
            *self = Self::default();
            self.selected = true;
            return Ok(vec![0]);
        }
        ensure!(self.selected && a[0] == 0x80, "not selected");
        let offset = u16::from_be_bytes([a[2], a[3]]) as usize;
        let payload = &a[5..];
        match a[1] {
            0x10 => {
                ensure!(offset == 0 && payload.len() == 34, "OPEN length");
                let length = u16::from_be_bytes([payload[0], payload[1]]) as usize;
                ensure!(length > 0 && length <= MAX_MESSAGE, "message size");
                self.expected = Some((length, payload[2..].try_into()?));
                self.data.clear();
                self.committed = false;
            }
            0x11 => {
                let (length, _) = self.expected.ok_or_else(|| anyhow::anyhow!("not opened"))?;
                ensure!(
                    !self.committed && !payload.is_empty() && payload.len() <= CHUNK,
                    "invalid chunk"
                );
                ensure!(offset + payload.len() <= length, "overflow");
                if offset < self.data.len() {
                    ensure!(
                        self.data.get(offset..offset + payload.len()) == Some(payload),
                        "changed duplicate"
                    );
                } else {
                    ensure!(offset == self.data.len(), "out of order");
                    self.data.extend(payload);
                }
            }
            0x12 => {
                ensure!(offset == 0 && payload.is_empty(), "COMMIT payload");
                let (length, digest) =
                    self.expected.ok_or_else(|| anyhow::anyhow!("not opened"))?;
                ensure!(
                    self.data.len() == length && hash(&self.data) == digest,
                    "incomplete or corrupt"
                );
                self.committed = true;
                return Ok(digest.to_vec());
            }
            _ => anyhow::bail!("unsupported instruction"),
        }
        Ok(vec![])
    }
}

pub fn frames(message: &[u8]) -> Result<Vec<Vec<u8>>> {
    ensure!(
        !message.is_empty() && message.len() <= MAX_MESSAGE,
        "message size"
    );
    let mut select = vec![0, 0xa4, 4, 0, AID.len() as u8];
    select.extend(AID);
    let mut open = vec![0x80, 0x10, 0, 0, 34];
    open.extend((message.len() as u16).to_be_bytes());
    open.extend(hash(message));
    let mut out = vec![select, open];
    for (i, chunk) in message.chunks(CHUNK).enumerate() {
        let offset = ((i * CHUNK) as u16).to_be_bytes();
        let mut a = vec![0x80, 0x11, offset[0], offset[1], chunk.len() as u8];
        a.extend(chunk);
        out.push(a);
    }
    out.push(vec![0x80, 0x12, 0, 0, 0]);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chunks_and_retries() {
        let m = vec![7; 700];
        let f = frames(&m).unwrap();
        let mut r = Receiver::default();
        for a in &f[..f.len() - 1] {
            assert!(r.process(a).ends_with(&[0x90, 0]));
        }
        assert!(r.process(&f[2]).ends_with(&[0x90, 0]));
        assert!(r.message().is_none());
        assert!(r.process(f.last().unwrap()).ends_with(&[0x90, 0]));
        assert_eq!(r.message(), Some(m.as_slice()));
    }
    #[test]
    fn reject_corrupt_duplicate_and_early_commit() {
        let f = frames(&[9; 240]).unwrap();
        let mut r = Receiver::default();
        r.process(&f[0]);
        r.process(&f[1]);
        assert_eq!(r.process(f.last().unwrap()), [0x6a, 0x80]);
        r.process(&f[0]);
        r.process(&f[1]);
        r.process(&f[2]);
        let mut bad = f[2].clone();
        bad[5] = 4;
        assert_eq!(r.process(&bad), [0x6a, 0x80]);
    }
    #[test]
    fn reject_unselected_and_size() {
        assert!(frames(&vec![0; MAX_MESSAGE + 1]).is_err());
        assert_eq!(
            Receiver::default().process(&[0x80, 0x12, 0, 0, 0]),
            [0x6a, 0x80]
        );
    }
}
