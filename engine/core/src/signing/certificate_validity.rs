use std::time::{SystemTime, UNIX_EPOCH};

use x509_cert::Certificate;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CertificateValidity {
    Current,
    Expired,
    NotYetValid,
}

impl CertificateValidity {
    pub(crate) fn of(certificate: &Certificate, at: SystemTime) -> CertificateValidity {
        let validity = certificate.tbs_certificate().validity();
        if at < UNIX_EPOCH + validity.not_before.to_unix_duration() {
            CertificateValidity::NotYetValid
        } else if at > UNIX_EPOCH + validity.not_after.to_unix_duration() {
            CertificateValidity::Expired
        } else {
            CertificateValidity::Current
        }
    }
}
