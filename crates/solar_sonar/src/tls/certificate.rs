use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CertFileKind {
    AuthorityPrivate,
    AuthorityPublic,
    EntityPrivate,
    EntityPublic,
}

impl CertFileKind {
    pub fn filepath(&self, dir: &Path, name: &str) -> PathBuf {
        match self {
            Self::AuthorityPrivate => dir.join(format!("authority_{name}.key.pem")),
            Self::AuthorityPublic => dir.join(format!("authority_{name}.pem")),
            Self::EntityPrivate => dir.join(format!("entity_{name}.key.pem")),
            Self::EntityPublic => dir.join(format!("entity_{name}.pem")),
        }
    }
}

pub(crate) struct CertFiles {
    pub(crate) authority_private: PathBuf,
    pub(crate) authority_public: PathBuf,
    pub(crate) entity_private: PathBuf,
    pub(crate) entity_public: PathBuf,
}

impl CertFiles {
    pub(crate) fn new(certs_dir: &Path, cert_name: &str) -> Self {
        Self {
            authority_private: CertFileKind::AuthorityPrivate.filepath(certs_dir, cert_name),
            authority_public: CertFileKind::AuthorityPublic.filepath(certs_dir, cert_name),
            entity_private: CertFileKind::EntityPrivate.filepath(certs_dir, cert_name),
            entity_public: CertFileKind::EntityPublic.filepath(certs_dir, cert_name),
        }
    }
    
    pub(crate) fn exist(&self) -> bool {
        self.authority_private.exists()
        && self.authority_public.exists()
        && self.entity_private.exists()
        && self.entity_public.exists()
    }
    
    pub(crate) fn write(&self, authority: &CertAuthority, entity: &CertEntity) -> SolarResult<()> {
        let issuer = authority.issuer();
        fs::write(&self.authority_private, issuer.key().serialize_pem())
            .map_err(|e| SolarError::write(e, &self.authority_private))?;
        fs::write(&self.authority_public, issuer.pem())
            .map_err(|e| SolarError::write(e, &self.authority_public))?;
        fs::write(&self.entity_private, entity.keypair.serialize_pem())
            .map_err(|e| SolarError::write(e, &self.entity_private))?;
        fs::write(&self.entity_public, entity.cert.pem())
            .map_err(|e| SolarError::write(e, &self.entity_public))?;
        
        Ok(())
    }
}

#[ouroboros::self_referencing]
pub(crate) struct CertAuthority {
    keypair: rcgen::KeyPair,
    #[covariant]
    #[borrows(keypair)]
    issuer: rcgen::CertifiedIssuer<'this, &'this rcgen::KeyPair>,
}

impl CertAuthority {
    pub(crate) fn issuer(&self) -> &rcgen::CertifiedIssuer<'_, &'_ rcgen::KeyPair> {
        self.borrow_issuer()
    }
}

pub(crate) struct CertEntity {
    keypair: rcgen::KeyPair,
    cert: rcgen::Certificate,
}

pub(crate) fn init_certs(cert_files: &CertFiles, org_name: &str) -> SolarResult<()> {
    if let Some(certs_dir) = cert_files.authority_private.parent() && !certs_dir.exists() {
        fs::create_dir_all(certs_dir)
            .map_err(|e| SolarError::mkdir(e, certs_dir))?;
    }
    
    let authority_keypair = rcgen::KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256)
        .map_err(|_| SolarError::msg("Unable to generate TLS certificates"))?;
    
    let mut authority_params = rcgen::CertificateParams::default();
    authority_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    authority_params.key_usages.extend([
        rcgen::KeyUsagePurpose::DigitalSignature,
        rcgen::KeyUsagePurpose::KeyCertSign,
    ]);
    authority_params.distinguished_name.push(rcgen::DnType::CommonName, "solar_sonar_authority");
    authority_params.distinguished_name.push(rcgen::DnType::OrganizationName, org_name);
    
    let authority = CertAuthority::try_new(authority_keypair, |keypair| -> SolarResult<_> {
        rcgen::CertifiedIssuer::self_signed(authority_params, keypair)
            .map_err(|_| SolarError::msg("Unable to generate TLS certificates"))
    })?;
    
    let server_keypair = rcgen::KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256)
        .map_err(|_| SolarError::msg("Unable to generate TLS certificates"))?;
    
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    
    let mut server_params = rcgen::CertificateParams::default();
    server_params.is_ca = rcgen::IsCa::NoCa;
    server_params.use_authority_key_identifier_extension = true;
    server_params.key_usages.push(rcgen::KeyUsagePurpose::DigitalSignature);
    server_params.extended_key_usages.push(rcgen::ExtendedKeyUsagePurpose::ServerAuth);
    server_params.distinguished_name.push(rcgen::DnType::CommonName, "solar_sonar_server");
    server_params.distinguished_name.push(rcgen::DnType::OrganizationName, org_name);
    server_params.subject_alt_names.push(rcgen::SanType::IpAddress(ip));
    
    let server_cert = server_params.signed_by(&server_keypair, authority.issuer())
        .map_err(|_| SolarError::msg("Unable to generate TLS certificates"))?;
    
    let entity = CertEntity { keypair: server_keypair, cert: server_cert };
    cert_files.write(&authority, &entity)
}
