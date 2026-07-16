//! RFC 3161 Time-Stamp Authority support (feature `tsa`).
//!
//! Adds a `signatureTimeStampToken` unsigned attribute to an externally-produced
//! PKCS#7 / CMS SignedData blob, upgrading a basic signature to CAdES-T.
//!
//! # Typical flow
//!
//! ```rust,no_run
//! use normordis_pdf::{DocumentBuilder, SignatureConfig};
//! use normordis_pdf::tsa::timestamp_pkcs7;
//!
//! let config = SignatureConfig::default();
//! let prepared = DocumentBuilder::new("Acta")
//!     .render_prepared_for_signing(config.to_options())?;
//!
//! // pkcs7_der produced externally by an HSM or qualified signing service
//! let pkcs7_der: Vec<u8> = vec![];
//! let pkcs7_with_tst = timestamp_pkcs7(&pkcs7_der, "http://tsa.example.com/tsa")?;
//! let signed_pdf = prepared.embed_signature(&pkcs7_with_tst)?;
//! # Ok::<(), normordis_pdf::NormordisPdfError>(())
//! ```

use crate::{NormordisPdfError, Result};

// ── DER primitives ────────────────────────────────────────────────────────────

fn der_encode_len(n: usize) -> Vec<u8> {
    if n < 0x80 {
        vec![n as u8]
    } else if n <= 0xFF {
        vec![0x81, n as u8]
    } else if n <= 0xFFFF {
        vec![0x82, (n >> 8) as u8, n as u8]
    } else {
        vec![0x83, (n >> 16) as u8, (n >> 8) as u8, n as u8]
    }
}

fn der_wrap(tag: u8, content: &[u8]) -> Vec<u8> {
    let mut out = vec![tag];
    out.extend(der_encode_len(content.len()));
    out.extend_from_slice(content);
    out
}

fn der_seq(parts: &[Vec<u8>]) -> Vec<u8> {
    let content: Vec<u8> = parts.iter().flat_map(|p| p.iter().copied()).collect();
    der_wrap(0x30, &content)
}

fn der_oid(oid_bytes: &[u8]) -> Vec<u8> {
    der_wrap(0x06, oid_bytes)
}

#[cfg(any(feature = "tsa", test))]
fn der_octet_string(data: &[u8]) -> Vec<u8> {
    der_wrap(0x04, data)
}

#[cfg(any(feature = "tsa", test))]
fn der_integer_u64(n: u64) -> Vec<u8> {
    if n == 0 {
        return vec![0x02, 0x01, 0x00];
    }
    let raw = n.to_be_bytes();
    let strip = raw.iter().position(|&b| b != 0).unwrap_or(7);
    let mut content = raw[strip..].to_vec();
    if content[0] & 0x80 != 0 {
        content.insert(0, 0x00);
    }
    der_wrap(0x02, &content)
}

// ── DER element navigator ─────────────────────────────────────────────────────

struct El {
    tag: u8,
    tag_pos: usize,
    content_start: usize,
    content_len: usize,
}

impl El {
    fn full_end(&self) -> usize {
        self.content_start + self.content_len
    }
}

fn read_el(data: &[u8], pos: usize) -> Option<El> {
    let tag = *data.get(pos)?;
    let (content_len, len_bytes) = decode_der_len(&data[pos + 1..])?;
    Some(El {
        tag,
        tag_pos: pos,
        content_start: pos + 1 + len_bytes,
        content_len,
    })
}

fn decode_der_len(data: &[u8]) -> Option<(usize, usize)> {
    match data.first().copied()? {
        b if b < 0x80 => Some((b as usize, 1)),
        0x80 | 0xFF => None,
        b => {
            let n = (b & 0x7F) as usize;
            if n > 4 || data.len() < 1 + n {
                return None;
            }
            let mut len = 0usize;
            for &byte in &data[1..=n] {
                len = (len << 8) | byte as usize;
            }
            Some((len, 1 + n))
        }
    }
}

// ── PKCS#7 navigation ─────────────────────────────────────────────────────────

struct NavResult {
    oid_el: El,
    sd: El,
    si_set: El,
    si: El,
    sig_el: El,
    /// Byte index where the new `[1] unsignedAttrs` should start (= `sig_el.full_end()`
    /// when no existing `[1]`, or the tag position of the existing `[1]`).
    insert_pos: usize,
    /// End of the element being replaced (= `insert_pos` when nothing to remove).
    remove_end: usize,
}

macro_rules! nav_el {
    ($data:expr, $pos:expr, $tag:expr, $ctx:expr) => {
        read_el($data, $pos)
            .filter(|e| e.tag == $tag)
            .ok_or_else(|| NormordisPdfError::TsaError($ctx.into()))
    };
    ($data:expr, $pos:expr, $ctx:expr) => {
        read_el($data, $pos).ok_or_else(|| NormordisPdfError::TsaError($ctx.into()))
    };
}

/// Walk the PKCS#7 DER tree to locate all structural components needed to
/// insert or replace the `[1] unsignedAttrs` in the first `SignerInfo`.
fn navigate_pkcs7(data: &[u8]) -> Result<NavResult> {
    // ContentInfo SEQUENCE
    let ci = nav_el!(data, 0, 0x30, "ContentInfo: expected SEQUENCE")?;
    // OID (signedData)
    let oid_el = nav_el!(data, ci.content_start, 0x06, "ContentInfo: expected OID")?;
    // [0] EXPLICIT wrapping SignedData
    let exp0 = nav_el!(data, oid_el.full_end(), 0xA0, "ContentInfo: expected [0] EXPLICIT")?;
    // SignedData SEQUENCE
    let sd = nav_el!(data, exp0.content_start, 0x30, "SignedData: expected SEQUENCE")?;

    // Skip version (INTEGER), digestAlgorithms (SET), encapContentInfo (SEQUENCE)
    let mut sd_pos = sd.content_start;
    for label in ["version", "digestAlgorithms", "encapContentInfo"] {
        sd_pos = read_el(data, sd_pos)
            .ok_or_else(|| {
                NormordisPdfError::TsaError(format!("SignedData: missing {label}"))
            })?
            .full_end();
    }
    // Skip optional certificates [0] and crls [1]
    while matches!(data.get(sd_pos), Some(&0xA0) | Some(&0xA1)) {
        sd_pos = read_el(data, sd_pos)
            .ok_or_else(|| {
                NormordisPdfError::TsaError("SignedData: malformed optional element".into())
            })?
            .full_end();
    }

    // signerInfos SET
    let si_set = nav_el!(data, sd_pos, 0x31, "SignedData: expected signerInfos SET")?;
    // First SignerInfo SEQUENCE
    let si = nav_el!(data, si_set.content_start, 0x30, "signerInfos: expected SEQUENCE")?;

    let mut pos = si.content_start;
    // version INTEGER
    pos = nav_el!(data, pos, 0x02, "SignerInfo: expected version INTEGER")?.full_end();
    // sid (SEQUENCE or context-tagged)
    pos = nav_el!(data, pos, "SignerInfo: expected sid")?.full_end();
    // digestAlgorithm SEQUENCE
    pos = nav_el!(data, pos, 0x30, "SignerInfo: expected digestAlgorithm")?.full_end();
    // optional signedAttrs [0]
    if data.get(pos).copied() == Some(0xA0) {
        pos = nav_el!(data, pos, "SignerInfo: malformed signedAttrs [0]")?.full_end();
    }
    // signatureAlgorithm SEQUENCE
    pos = nav_el!(data, pos, 0x30, "SignerInfo: expected signatureAlgorithm")?.full_end();
    // signature OCTET STRING
    let sig_el = nav_el!(data, pos, 0x04, "SignerInfo: expected signature OCTET STRING")?;
    pos = sig_el.full_end();

    // Check for existing [1] unsignedAttrs
    let (insert_pos, remove_end) = if data.get(pos).copied() == Some(0xA1) {
        let existing = nav_el!(data, pos, "SignerInfo: malformed [1] unsignedAttrs")?;
        (pos, existing.full_end())
    } else {
        (pos, pos)
    };

    let _ = exp0; // used locally to find sd; not needed in NavResult
    Ok(NavResult {
        oid_el,
        sd,
        si_set,
        si,
        sig_el,
        insert_pos,
        remove_end,
    })
}

// ── TSQ builder ───────────────────────────────────────────────────────────────

#[cfg(any(feature = "tsa", test))]
/// Encode a DER `TimeStampReq` that requests a timestamp over `hash`
/// (SHA-256 of the CMS signature value).
fn build_timestamp_request(hash: &[u8]) -> Vec<u8> {
    // OID 2.16.840.1.101.3.4.2.1 (SHA-256)
    const SHA256_OID: &[u8] = &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01];

    let alg_id = der_seq(&[der_oid(SHA256_OID), vec![0x05, 0x00]]);
    let msg_imprint = der_seq(&[alg_id, der_octet_string(hash)]);

    // Nonce: current time in nanoseconds (u64).  Prevents response replay.
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x4E6F726D6F726469); // "Normord" as static fallback

    der_seq(&[
        vec![0x02, 0x01, 0x01],  // version v1(1)
        msg_imprint,
        der_integer_u64(nonce),  // nonce
        vec![0x01, 0x01, 0xFF],  // certReq TRUE
    ])
}

// ── TSR parser ────────────────────────────────────────────────────────────────

#[cfg(feature = "tsa")]
/// Extract the `TimeStampToken` (`ContentInfo`) bytes from a DER-encoded
/// `TimeStampResp`.
fn extract_timestamp_token(tsr: &[u8]) -> Result<Vec<u8>> {
    let outer = read_el(tsr, 0)
        .filter(|e| e.tag == 0x30)
        .ok_or_else(|| NormordisPdfError::TsaError("TSR: expected outer SEQUENCE".into()))?;

    let pki = read_el(tsr, outer.content_start)
        .filter(|e| e.tag == 0x30)
        .ok_or_else(|| NormordisPdfError::TsaError("TSR: expected PKIStatusInfo".into()))?;

    // First element of PKIStatusInfo is `status INTEGER`
    let status_el = read_el(tsr, pki.content_start)
        .filter(|e| e.tag == 0x02)
        .ok_or_else(|| NormordisPdfError::TsaError("TSR: expected status INTEGER".into()))?;

    let status = tsr.get(status_el.content_start).copied().unwrap_or(0xFF);
    // 0 = granted, 1 = grantedWithMods
    if status > 1 {
        return Err(NormordisPdfError::TsaError(format!(
            "TSA rejected request with status {status}"
        )));
    }

    let tst_start = pki.full_end();
    let tst_end = outer.content_start + outer.content_len;
    if tst_start >= tst_end {
        return Err(NormordisPdfError::TsaError(
            "TSR: no timeStampToken in response".into(),
        ));
    }
    Ok(tsr[tst_start..tst_end].to_vec())
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Extract the raw signature value from the first `SignerInfo` of a CMS `SignedData`.
///
/// This is the byte string that must be hashed (SHA-256) when building the TSA request.
pub fn extract_signature_value(pkcs7_der: &[u8]) -> Result<Vec<u8>> {
    let nav = navigate_pkcs7(pkcs7_der)?;
    let sig = &nav.sig_el;
    Ok(pkcs7_der[sig.content_start..sig.content_start + sig.content_len].to_vec())
}

/// Embed a `TimeStampToken` (DER `ContentInfo` from a `TimeStampResp`) into a
/// CMS `SignedData` as the `id-aa-signatureTimeStampToken` unsigned attribute
/// on the first `SignerInfo`.
///
/// The entire PKCS#7 structure is **rebuilt from the inside out** so that DER
/// length fields at all levels are correctly re-encoded — no in-place patching.
///
/// If a `[1] IMPLICIT` unsignedAttrs already exists it is replaced entirely.
pub fn embed_timestamp(pkcs7_der: &[u8], tst_der: &[u8]) -> Result<Vec<u8>> {
    // OID 1.2.840.113549.1.9.16.2.14 (id-aa-signatureTimeStampToken)
    const TST_OID: &[u8] = &[
        0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x09, 0x10, 0x02, 0x0E,
    ];

    // Attribute SEQUENCE { OID, SET { ContentInfo } }
    let tst_attr = der_seq(&[der_oid(TST_OID), der_wrap(0x31, tst_der)]);
    // [1] IMPLICIT SET (unsignedAttrs)
    let new_ua = der_wrap(0xA1, &tst_attr);

    let nav = navigate_pkcs7(pkcs7_der)?;
    let d = pkcs7_der;

    // ── Rebuild from inside out ───────────────────────────────────────────

    // SignerInfo: prefix (up to and including signature) + new [1] + suffix
    let si_prefix = &d[nav.si.content_start..nav.insert_pos];
    let si_suffix = &d[nav.remove_end..nav.si.full_end()];
    let new_si = der_wrap(
        0x30,
        &chain3(si_prefix, &new_ua, si_suffix),
    );

    // signerInfos SET: pre-first (empty in practice) + new_si + post-first
    let si_set_prefix = &d[nav.si_set.content_start..nav.si.tag_pos];
    let si_set_suffix = &d[nav.si.full_end()..nav.si_set.full_end()];
    let new_si_set = der_wrap(
        0x31,
        &chain3(si_set_prefix, &new_si, si_set_suffix),
    );

    // SignedData: before signerInfos + new signerInfos + after signerInfos
    let sd_prefix = &d[nav.sd.content_start..nav.si_set.tag_pos];
    let sd_suffix = &d[nav.si_set.full_end()..nav.sd.full_end()];
    let new_sd = der_wrap(
        0x30,
        &chain3(sd_prefix, &new_si_set, sd_suffix),
    );

    // [0] EXPLICIT wrapping SignedData
    let new_exp0 = der_wrap(0xA0, &new_sd);

    // ContentInfo: OID + new [0]
    let oid_bytes = &d[nav.oid_el.tag_pos..nav.oid_el.full_end()];
    let ci_content: Vec<u8> = oid_bytes.iter().chain(new_exp0.iter()).copied().collect();
    Ok(der_wrap(0x30, &ci_content))
}

/// Send the SHA-256 hash of `signature_value` to a TSA (RFC 3161 over HTTP/S)
/// and return the raw `TimeStampToken` (`ContentInfo`) DER bytes.
///
/// Requires the `tsa` feature.
#[cfg(feature = "tsa")]
pub fn request_timestamp(signature_value: &[u8], tsa_url: &str) -> Result<Vec<u8>> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let hash = Sha256::digest(signature_value);
    let tsq = build_timestamp_request(hash.as_ref());

    let response = ureq::post(tsa_url)
        .set("Content-Type", "application/timestamp-query")
        .send_bytes(&tsq)
        .map_err(|e| NormordisPdfError::TsaError(format!("TSA HTTP error: {e}")))?;

    let mut tsr = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut tsr)
        .map_err(|e| NormordisPdfError::TsaError(format!("TSA response read error: {e}")))?;

    extract_timestamp_token(&tsr)
}

/// Convenience: hash the signature value in `pkcs7_der`, obtain a
/// `TimeStampToken` from `tsa_url`, and embed it as an unsigned attribute,
/// returning a CAdES-T PKCS#7 blob ready for `PreparedPdf::embed_signature`.
///
/// Requires the `tsa` feature.
#[cfg(feature = "tsa")]
pub fn timestamp_pkcs7(pkcs7_der: &[u8], tsa_url: &str) -> Result<Vec<u8>> {
    let sig_value = extract_signature_value(pkcs7_der)?;
    let tst_der = request_timestamp(&sig_value, tsa_url)?;
    embed_timestamp(pkcs7_der, &tst_der)
}

// ── Helper ────────────────────────────────────────────────────────────────────

fn chain3(a: &[u8], b: &[u8], c: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(a.len() + b.len() + c.len());
    out.extend_from_slice(a);
    out.extend_from_slice(b);
    out.extend_from_slice(c);
    out
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a minimal but structurally valid CMS PKCS#7 for testing.
    fn make_test_pkcs7(sig_bytes: &[u8]) -> Vec<u8> {
        const SIGNED_DATA_OID: &[u8] =
            &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x07, 0x02];
        const SHA256_OID: &[u8] = &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01];
        const DATA_OID: &[u8] = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x07, 0x01];
        const RSA_OID: &[u8] = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x01, 0x01];

        let alg_sha256 = der_seq(&[der_oid(SHA256_OID), vec![0x05, 0x00]]);
        let alg_rsa = der_seq(&[der_oid(RSA_OID), vec![0x05, 0x00]]);

        // IssuerAndSerialNumber { Name{}, serialNumber=1 }
        let sid = der_seq(&[der_seq(&[]), vec![0x02, 0x01, 0x01]]);

        let signer_info = der_seq(&[
            vec![0x02, 0x01, 0x01], // version
            sid,
            alg_sha256.clone(),
            alg_rsa,
            der_octet_string(sig_bytes),
        ]);

        let signed_data = der_seq(&[
            vec![0x02, 0x01, 0x01],
            der_wrap(0x31, &alg_sha256),
            der_seq(&[der_oid(DATA_OID)]),
            der_wrap(0x31, &signer_info),
        ]);

        der_seq(&[
            der_oid(SIGNED_DATA_OID),
            der_wrap(0xA0, &signed_data),
        ])
    }

    #[test]
    fn der_encode_len_short() {
        assert_eq!(der_encode_len(0), [0x00]);
        assert_eq!(der_encode_len(127), [0x7F]);
    }

    #[test]
    fn der_encode_len_long() {
        assert_eq!(der_encode_len(128), [0x81, 0x80]);
        assert_eq!(der_encode_len(256), [0x82, 0x01, 0x00]);
    }

    #[test]
    fn der_integer_u64_zero() {
        assert_eq!(der_integer_u64(0), [0x02, 0x01, 0x00]);
    }

    #[test]
    fn der_integer_u64_no_sign_padding_needed() {
        // 1 = 0x01, high bit clear → no padding
        assert_eq!(der_integer_u64(1), [0x02, 0x01, 0x01]);
    }

    #[test]
    fn der_integer_u64_sign_padding_needed() {
        // 0x80 has high bit set → needs 0x00 prefix
        let enc = der_integer_u64(0x80);
        assert_eq!(&enc, &[0x02, 0x02, 0x00, 0x80]);
    }

    #[test]
    fn build_tsq_is_sequence() {
        let tsq = build_timestamp_request(&[0u8; 32]);
        assert_eq!(tsq[0], 0x30, "TSQ must start with SEQUENCE");
        assert!(tsq.len() > 50, "TSQ must contain nonce and other fields");
    }

    #[test]
    fn build_tsq_contains_sha256_oid() {
        const SHA256_OID: &[u8] = &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01];
        let tsq = build_timestamp_request(&[0xABu8; 32]);
        assert!(
            tsq.windows(SHA256_OID.len()).any(|w| w == SHA256_OID),
            "TSQ must contain SHA-256 OID"
        );
    }

    #[test]
    fn navigate_pkcs7_finds_signature() {
        let sig: Vec<u8> = (0u8..32).collect();
        let pkcs7 = make_test_pkcs7(&sig);
        let nav = navigate_pkcs7(&pkcs7).expect("navigate_pkcs7");
        let got = &pkcs7[nav.sig_el.content_start..nav.sig_el.content_start + nav.sig_el.content_len];
        assert_eq!(got, sig.as_slice());
    }

    #[test]
    fn extract_signature_value_round_trips() {
        let expected: Vec<u8> = (0u8..64).collect();
        let pkcs7 = make_test_pkcs7(&expected);
        let got = extract_signature_value(&pkcs7).expect("extract_signature_value");
        assert_eq!(got, expected);
    }

    #[test]
    fn embed_timestamp_adds_unsigned_attrs() {
        let sig: Vec<u8> = (0u8..32).collect();
        let pkcs7 = make_test_pkcs7(&sig);
        // Synthetic TST: any valid DER SEQUENCE
        let fake_tst = der_seq(&[vec![0x01, 0x01, 0xFF]]);

        let patched = embed_timestamp(&pkcs7, &fake_tst).expect("embed_timestamp");

        assert_eq!(patched[0], 0x30, "result must start with SEQUENCE");
        assert!(patched.len() > pkcs7.len(), "result must be larger");
        assert!(
            patched.windows(1).any(|w| w == [0xA1]),
            "result must contain [1] tag for unsignedAttrs"
        );
        // Signature bytes are preserved
        assert!(
            patched.windows(sig.len()).any(|w| w == sig.as_slice()),
            "original signature bytes must survive embed"
        );
    }

    #[test]
    fn embed_timestamp_result_is_navigable() {
        let sig: Vec<u8> = (10u8..42).collect();
        let pkcs7 = make_test_pkcs7(&sig);
        let fake_tst = der_seq(&[vec![0x02, 0x01, 0x07]]); // INTEGER 7
        let patched = embed_timestamp(&pkcs7, &fake_tst).expect("embed_timestamp");

        // Navigation must succeed on the patched result
        let nav = navigate_pkcs7(&patched).expect("navigate patched PKCS#7");
        let recovered = &patched[nav.sig_el.content_start..nav.sig_el.content_start + nav.sig_el.content_len];
        assert_eq!(recovered, sig.as_slice(), "signature preserved in patched PKCS#7");
    }

    #[test]
    fn embed_timestamp_replaces_existing_unsigned_attrs() {
        let sig: Vec<u8> = (0u8..32).collect();
        let pkcs7 = make_test_pkcs7(&sig);

        let tst1 = der_seq(&[vec![0x01, 0x01, 0xFF]]); // BOOLEAN TRUE
        let tst2 = der_seq(&[vec![0x01, 0x01, 0x00]]); // BOOLEAN FALSE (same size)

        let after_first = embed_timestamp(&pkcs7, &tst1).expect("first embed");
        let after_second = embed_timestamp(&after_first, &tst2).expect("second embed");

        assert_eq!(
            after_first.len(),
            after_second.len(),
            "replacing same-size TST must keep total length"
        );
        // Original signature must still be intact
        let nav = navigate_pkcs7(&after_second).expect("navigate after second embed");
        let sig_got = &after_second[nav.sig_el.content_start..nav.sig_el.content_start + nav.sig_el.content_len];
        assert_eq!(sig_got, sig.as_slice());
    }

    /// Network test — disabled by default.
    /// Run with: `cargo test --features tsa -- --include-ignored tsa_network`
    #[cfg(feature = "tsa")]
    #[test]
    #[ignore = "requires network access to a real TSA (https://freetsa.org/tsr)"]
    fn tsa_network_freetsa_org() {
        let sig: Vec<u8> = (0u8..64).collect();
        let pkcs7 = make_test_pkcs7(&sig);

        let result = timestamp_pkcs7(&pkcs7, "https://freetsa.org/tsr");
        assert!(result.is_ok(), "timestamp_pkcs7 failed: {result:?}");

        let patched = result.unwrap();
        assert!(patched.len() > pkcs7.len(), "patched must be larger");

        // Navigation must succeed on the timestamped result
        navigate_pkcs7(&patched).expect("navigate timestamped PKCS#7 must not fail");
    }
}
