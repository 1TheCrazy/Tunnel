use sha2::{Digest, Sha256};
use std::{
    fmt,
    sync::Arc,
};

use rustls::{
    client::danger::{
        HandshakeSignatureValid,
        ServerCertVerified,
        ServerCertVerifier,
    },
    crypto::{
        verify_tls12_signature,
        verify_tls13_signature,
        CryptoProvider,
    },
    pki_types::{
        CertificateDer,
        ServerName,
        UnixTime,
    },
    DigitallySignedStruct,
    Error as RustlsError,
    SignatureScheme,
};

pub struct FingerprintVerifier {
    pub expected_sha256: Option<[u8; 32]>,
    pub server_name: String,
    pub blindly_trusted_fingerprint_recieved: Box<dyn Fn(String) + Send + Sync + 'static>,
    pub provider: Arc<CryptoProvider>,
}

impl fmt::Debug for FingerprintVerifier {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        formatter
            .debug_struct("FingerprintVerifier")
            .finish_non_exhaustive()
    }
}

impl ServerCertVerifier for FingerprintVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, RustlsError> {
        

        /*
         * end_entity contains the exact DER bytes of the leaf
         * certificate presented by the server.
         */
        let actual_sha256: [u8; 32] =
            Sha256::digest(end_entity.as_ref()).into();

        if let Some(expected_sha256) = self.expected_sha256 {
            if actual_sha256 != expected_sha256 {
                return Err(RustlsError::General(format!(
                    "TLS certificate fingerprint mismatch: \
                    expected {}, received {}",
                        hex::encode_upper(expected_sha256),
                        hex::encode_upper(actual_sha256),
                    )));
            }
        }
        // We don't have an expected value, therefore we blindly trust the host
        else {
            // Same-Name check is redundant, but just to be sure
            if server_name.to_str() != self.server_name {
                return Err(RustlsError::General(format!(
                    "Wanted to trust initial fingerprint, but the host didn't match"
                )));
            }

            (self.blindly_trusted_fingerprint_recieved)(hex::encode_upper(actual_sha256));
        }

        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        certificate: &CertificateDer<'_>,
        signature: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        verify_tls12_signature(
            message,
            certificate,
            signature,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        certificate: &CertificateDer<'_>,
        signature: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        verify_tls13_signature(
            message,
            certificate,
            signature,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(
        &self,
    ) -> Vec<SignatureScheme> {
        self.provider
            .signature_verification_algorithms
            .supported_schemes()
    }
}