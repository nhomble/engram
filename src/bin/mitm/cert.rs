/// TLS certificate auto-generation for MITM proxy
///
/// On first run, generates a self-signed CA certificate and stores it in ~/.engram-mitm/
/// This CA is used to sign per-connection certificates for HTTPS interception

use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType, KeyPair};
use std::fs;
use std::path::PathBuf;

const CA_CERT_FILE: &str = "ca.crt";
const CA_KEY_FILE: &str = "ca.key";

/// Certificate Authority for signing per-connection certs
pub struct CertificateAuthority {
    cert: Certificate,
    key_pair: KeyPair,
    cert_pem: String,
}

impl CertificateAuthority {
    /// Load or create CA certificate
    ///
    /// If CA doesn't exist, generates new one and prints trust instructions
    pub fn load_or_create() -> Result<Self, Box<dyn std::error::Error>> {
        let ca_dir = ca_directory()?;
        let cert_path = ca_dir.join(CA_CERT_FILE);
        let key_path = ca_dir.join(CA_KEY_FILE);

        if cert_path.exists() && key_path.exists() {
            // Load existing CA
            let cert_pem = fs::read_to_string(&cert_path)?;
            let key_pem = fs::read_to_string(&key_path)?;

            let key_pair = KeyPair::from_pem(&key_pem)?;
            let mut params = CertificateParams::default();
            params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

            let cert = params.self_signed(&key_pair)?;

            Ok(Self { cert, key_pair, cert_pem })
        } else {
            // Generate new CA
            fs::create_dir_all(&ca_dir)?;

            let key_pair = KeyPair::generate()?;
            let mut params = CertificateParams::default();
            params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

            let mut dn = DistinguishedName::new();
            dn.push(DnType::CommonName, "Engram MITM Proxy CA");
            dn.push(DnType::OrganizationName, "Engram");
            params.distinguished_name = dn;

            let cert = params.self_signed(&key_pair)?;
            let cert_pem = cert.pem();
            let key_pem = key_pair.serialize_pem();

            fs::write(&cert_path, &cert_pem)?;
            fs::write(&key_path, &key_pem)?;

            println!("\n=== Engram MITM CA Certificate Generated ===");
            println!("Certificate: {}", cert_path.display());
            println!("\nTo enable HTTPS interception, trust this certificate:");
            println!("\nmacOS:");
            println!("  sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain {}", cert_path.display());
            println!("\nLinux:");
            println!("  sudo cp {} /usr/local/share/ca-certificates/engram-mitm.crt", cert_path.display());
            println!("  sudo update-ca-certificates");
            println!("\nWindows:");
            println!("  Import {} to 'Trusted Root Certification Authorities'", cert_path.display());
            println!("\n============================================\n");

            Ok(Self { cert, key_pair, cert_pem })
        }
    }

    /// Generate a certificate for a specific domain signed by this CA
    pub fn sign_for_domain(&self, domain: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
        let key_pair = KeyPair::generate()?;
        let mut params = CertificateParams::default();
        params.subject_alt_names = vec![
            rcgen::SanType::DnsName(rcgen::Ia5String::try_from(domain.to_string())?),
        ];

        let cert = params.signed_by(&key_pair, &self.cert, &self.key_pair)?;
        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();

        Ok((cert_pem, key_pem))
    }

    /// Get the CA certificate PEM (for clients to trust)
    pub fn cert_pem(&self) -> &str {
        &self.cert_pem
    }
}

/// Get the CA directory path (~/.engram-mitm)
fn ca_directory() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "Cannot determine home directory")?;

    Ok(PathBuf::from(home).join(".engram-mitm"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ca_generation() {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        // Use unique temp directory for testing to avoid parallel test conflicts
        let unique_id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_dir = std::env::temp_dir().join(format!("engram-mitm-test-{}", unique_id));

        // Clean up any previous state
        std::fs::remove_dir_all(&temp_dir).ok();

        std::env::set_var("HOME", &temp_dir);

        let ca = CertificateAuthority::load_or_create().unwrap();

        // Verify CA cert exists
        let cert_path = temp_dir.join(".engram-mitm").join(CA_CERT_FILE);
        assert!(cert_path.exists());

        // Verify we can sign a domain cert
        let (cert_pem, key_pem) = ca.sign_for_domain("example.com").unwrap();
        assert!(cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(key_pem.contains("BEGIN PRIVATE KEY"));

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_ca_persistence() {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let unique_id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_dir = std::env::temp_dir().join(format!("engram-mitm-persist-test-{}", unique_id));

        // Clean up any previous state
        std::fs::remove_dir_all(&temp_dir).ok();

        std::env::set_var("HOME", &temp_dir);

        // Create CA
        let ca1 = CertificateAuthority::load_or_create().unwrap();
        let pem1 = ca1.cert_pem().to_string();

        // Load CA again - should load from disk, not regenerate
        let ca2 = CertificateAuthority::load_or_create().unwrap();
        let pem2 = ca2.cert_pem().to_string();

        // Should be the same
        assert_eq!(pem1, pem2);

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
