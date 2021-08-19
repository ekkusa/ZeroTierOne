use std::hash::{Hash, Hasher};

use crate::vl1::{Address, Endpoint, Identity};
use crate::vl1::buffer::Buffer;
use crate::vl1::protocol::PACKET_SIZE_MAX;

/// Maximum number of endpoints allowed in a Locator.
pub const LOCATOR_MAX_ENDPOINTS: usize = 32;

/// A signed object generated by nodes to inform the network where they may be found.
///
/// By default this will just enumerate the roots used by this node, but nodes with
/// static IPs can also list physical IP/port addresses where they can be reached with
/// no involvement from a root at all.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Locator {
    pub(crate) subject: Address,
    pub(crate) signer: Address,
    pub(crate) timestamp: i64,
    pub(crate) endpoints: Vec<Endpoint>,
    pub(crate) signature: Vec<u8>,
}

impl Locator {
    /// Create and sign a new locator.
    ///
    /// If a node is creating its own locator the subject will be the address from the
    /// signer identity. Proxy signing is when these do not match and is only done by
    /// roots to create locators for old versions of ZeroTier that do not create their
    /// own. Proxy locators are always superseded by self-signed locators.
    ///
    /// This returns None if an error occurs, which can only be something indicating a
    /// bug like too many endpoints or the identity lacking its secret keys.
    pub fn create(signer_identity: &Identity, subject: Address, ts: i64, endpoints: &[Endpoint]) -> Option<Locator> {
        if endpoints.len() > LOCATOR_MAX_ENDPOINTS {
            return None;
        }

        let mut loc = Locator {
            subject,
            signer: signer_identity.address(),
            timestamp: ts,
            endpoints: endpoints.to_vec(),
            signature: Vec::new()
        };
        loc.endpoints.sort_unstable();
        loc.endpoints.dedup();

        let mut buf: Buffer<{ PACKET_SIZE_MAX }> = Buffer::new();
        if loc.marshal_internal(&mut buf, true).is_err() {
            return None;
        }
        signer_identity.sign(buf.as_bytes()).map(|sig| {
            loc.signature = sig;
            loc
        })
    }

    /// Check if this locator should replace one that is already known.
    pub fn should_replace(&self, other: &Self) -> bool {
        if self.subject == self.signer && other.subject != other.signer {
            true
        } else if self.subject != self.signer && other.subject == other.signer {
            false
        } else {
            self.timestamp > other.timestamp
        }
    }

    #[inline(always)]
    pub fn subject(&self) -> Address {
        self.subject
    }

    #[inline(always)]
    pub fn signer(&self) -> Address {
        self.signer
    }

    #[inline(always)]
    pub fn is_proxy_signed(&self) -> bool {
        self.subject != self.signer
    }

    #[inline(always)]
    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    #[inline(always)]
    pub fn endpoints(&self) -> &[Endpoint] {
        self.endpoints.as_slice()
    }

    pub fn verify_signature(&self, signer_identity: &Identity) -> bool {
        let mut buf: Buffer<{ PACKET_SIZE_MAX }> = Buffer::new();
        if self.marshal_internal(&mut buf, true).is_ok() {
            if signer_identity.address() == self.signer {
                signer_identity.verify(buf.as_bytes(), self.signature.as_slice())
            } else {
                false
            }
        } else {
            false
        }
    }

    fn marshal_internal<const BL: usize>(&self, buf: &mut Buffer<BL>, exclude_signature: bool) -> std::io::Result<()> {
        buf.append_u64(self.subject.to_u64())?;
        buf.append_u64(self.signer.to_u64())?;
        buf.append_u64(self.timestamp as u64)?;
        debug_assert!(self.endpoints.len() < 65536);
        buf.append_u16(self.endpoints.len() as u16)?;
        for e in self.endpoints.iter() {
            e.marshal(buf)?;
        }
        buf.append_u16(0)?;
        if !exclude_signature {
            debug_assert!(self.signature.len() < 65536);
            buf.append_u16(self.signature.len() as u16)?;
            buf.append_bytes(self.signature.as_slice())?;
        }
        Ok(())
    }

    #[inline(always)]
    pub fn marshal<const BL: usize>(&self, buf: &mut Buffer<BL>) -> std::io::Result<()> {
        self.marshal_internal(buf, false)
    }

    pub fn unmarshal<const BL: usize>(buf: &Buffer<BL>, cursor: &mut usize) -> std::io::Result<Self> {
        let subject = Address::from(buf.read_u64(cursor)?);
        let signer = Address::from(buf.read_u64(cursor)?);
        let timestamp = buf.read_u64(cursor)? as i64;
        let endpoint_count = buf.read_u16(cursor)? as usize;
        if endpoint_count > LOCATOR_MAX_ENDPOINTS {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "too many endpoints"));
        }
        let mut endpoints: Vec<Endpoint> = Vec::new();
        endpoints.reserve(endpoint_count);
        for _ in 0..endpoint_count {
            endpoints.push(Endpoint::unmarshal(buf, cursor)?);
        }
        *cursor += buf.read_u16(cursor)? as usize;
        let signature_len = buf.read_u16(cursor)? as usize;
        let signature = buf.read_bytes(signature_len, cursor)?;
        Ok(Locator {
            subject,
            signer,
            timestamp,
            endpoints,
            signature: signature.to_vec(),
        })
    }
}

impl Hash for Locator {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if !self.signature.is_empty() {
            state.write(self.signature.as_slice());
        } else {
            state.write_u64(self.signer.to_u64());
            state.write_i64(self.timestamp);
            for e in self.endpoints.iter() {
                e.hash(state);
            }
        }
    }
}
