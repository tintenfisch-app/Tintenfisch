use std::time::{SystemTime, UNIX_EPOCH};

use cms::cert::CertificateChoices;
use cms::content_info::ContentInfo;
use cms::signed_data::{SignedData, SignerIdentifier, SignerInfo};
use der::asn1::OctetString;
use der::{Decode, Encode, SliceReader};
use x509_cert::Certificate;
use x509_cert::ext::pkix::SubjectKeyIdentifier;
use x509_cert::time::Time;

use crate::signing::certificate_validity::CertificateValidity;
use crate::signing::digest_algorithm::DigestAlgorithm;
use crate::signing::identity_status::IdentityStatus;
use crate::signing::oids;
use crate::signing::signature_status::SignatureStatus;
use crate::signing::signing_identity::{common_name, public_key};

pub struct CmsCheck {
    pub status: SignatureStatus,
    pub identity: Option<IdentityStatus>,
    pub signer: Option<String>,
}

impl CmsCheck {
    pub fn run(cms: &[u8], signed: &[&[u8]], now: SystemTime) -> CmsCheck {
        let signed_data = SliceReader::new(cms)
            .and_then(|mut reader| ContentInfo::decode(&mut reader))
            .ok()
            .filter(|info| info.content_type == oids::SIGNED_DATA)
            .and_then(|info| info.content.decode_as::<SignedData>().ok());
        let Some(signed_data) = signed_data else {
            return CmsCheck::broken("the signature is not readable");
        };
        let Some(signer_info) = signed_data.signer_infos.0.iter().next() else {
            return CmsCheck::broken("the signature names no signer");
        };
        let Some(certificate) = signer_certificate(&signed_data, &signer_info.sid) else {
            return CmsCheck::broken("the signer's certificate is missing");
        };
        let signer = Some(common_name(certificate.tbs_certificate().subject()));
        let status = verdict(signer_info, certificate, signed);
        let identity = (status == SignatureStatus::Intact).then(|| identity(signer_info, certificate, now));
        CmsCheck { status, identity, signer }
    }

    fn broken(problem: &str) -> CmsCheck {
        CmsCheck { status: SignatureStatus::Broken(problem.to_string()), identity: None, signer: None }
    }
}

fn identity(signer_info: &SignerInfo, certificate: &Certificate, now: SystemTime) -> IdentityStatus {
    let tbs = certificate.tbs_certificate();
    let validity = tbs.validity();
    match CertificateValidity::of(certificate, signing_time(signer_info).unwrap_or(now)) {
        CertificateValidity::Expired => {
            IdentityStatus::Expired(format!("the certificate was valid until {}", day(&validity.not_after)))
        }
        CertificateValidity::NotYetValid => {
            IdentityStatus::NotYetValid(format!("the certificate is valid from {} only", day(&validity.not_before)))
        }
        CertificateValidity::Current if tbs.issuer() == tbs.subject() => IdentityStatus::Unverified(
            "the certificate is self-signed, so nobody vouches for the signer's name".to_string(),
        ),
        CertificateValidity::Current => IdentityStatus::Unverified(format!(
            "the issuer {} is not checked against a list of trusted authorities",
            common_name(tbs.issuer())
        )),
    }
}

fn signing_time(signer_info: &SignerInfo) -> Option<SystemTime> {
    let attribute = signer_info.signed_attrs.as_ref()?.iter().find(|attribute| attribute.oid == oids::SIGNING_TIME)?;
    let encoded = attribute.values.iter().next()?.to_der().ok()?;
    Time::from_der(&encoded).ok().map(|time| UNIX_EPOCH + time.to_unix_duration())
}

fn day(time: &Time) -> String {
    let date = time.to_date_time();
    format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day())
}

fn verdict(signer_info: &SignerInfo, certificate: &Certificate, signed: &[&[u8]]) -> SignatureStatus {
    let Some(digest_algorithm) = DigestAlgorithm::from_oid(signer_info.digest_alg.oid) else {
        return SignatureStatus::Unsupported(format!("the hash {} cannot be checked", signer_info.digest_alg.oid));
    };
    let signature_oid = signer_info.signature_algorithm.oid;
    let hash = if signature_oid == oids::RSA_ENCRYPTION {
        Some(digest_algorithm)
    } else {
        DigestAlgorithm::of_rsa_signature(signature_oid)
    };
    let Some(hash) = hash else {
        return SignatureStatus::Unsupported(format!("signatures made with {signature_oid} cannot be checked yet"));
    };
    let digest = digest_algorithm.digest(signed);
    let message = match &signer_info.signed_attrs {
        Some(attributes) => {
            let claimed = attributes
                .iter()
                .find(|attribute| attribute.oid == oids::MESSAGE_DIGEST)
                .and_then(|attribute| attribute.values.iter().next())
                .and_then(|value| value.decode_as::<OctetString>().ok());
            if claimed.as_ref().map(OctetString::as_bytes) != Some(digest.as_slice()) {
                return SignatureStatus::Broken("the signed bytes have changed".to_string());
            }
            match attributes.to_der() {
                Ok(encoded) => hash.digest(&[&encoded]),
                Err(_) => return SignatureStatus::Broken("the signed attributes are not readable".to_string()),
            }
        }
        None => digest,
    };
    let Some(key) = public_key(certificate) else {
        return SignatureStatus::Unsupported("only RSA signatures can be checked so far".to_string());
    };
    match key.verify(hash.pkcs1v15(), &message, signer_info.signature.as_bytes()) {
        Ok(()) => SignatureStatus::Intact,
        Err(_) => SignatureStatus::Broken("the signature does not match the signer's certificate".to_string()),
    }
}

fn signer_certificate<'d>(signed_data: &'d SignedData, sid: &SignerIdentifier) -> Option<&'d Certificate> {
    signed_data.certificates.as_ref()?.0.iter().find_map(|choice| match choice {
        CertificateChoices::Certificate(certificate) => {
            let tbs = certificate.tbs_certificate();
            let matches = match sid {
                SignerIdentifier::IssuerAndSerialNumber(issuer) => {
                    tbs.issuer() == &issuer.issuer && tbs.serial_number() == &issuer.serial_number
                }
                SignerIdentifier::SubjectKeyIdentifier(wanted) => {
                    tbs.get_extension::<SubjectKeyIdentifier>().ok().flatten().is_some_and(|(_, own)| &own == wanted)
                }
            };
            matches.then_some(certificate)
        }
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn the_signing_time_in_the_signature_decides_whether_the_certificate_had_expired() {
        let cms =
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/certificates/signed-with-signing-time.p7s"))
                .unwrap();
        let while_valid = UNIX_EPOCH + Duration::from_secs(1_561_939_200);
        let check = CmsCheck::run(&cms, &[b"Tintenfisch signing time sample\n"], while_valid);
        assert_eq!(check.status, SignatureStatus::Intact);
        assert_eq!(check.signer.as_deref(), Some("Tintenfisch Expired Signer"));
        assert_eq!(
            check.identity,
            Some(IdentityStatus::Expired("the certificate was valid until 2020-01-01".to_string()))
        );
    }
}
