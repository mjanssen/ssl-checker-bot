use ssl_expiration2::SslExpiration;

pub enum DomainStatus {
    ValidFor(i32),
    Expired,
    ValidationFailed(String),
}

pub fn verify_domain(domain_name: &str) -> DomainStatus {
    match SslExpiration::from_domain_name(domain_name) {
        Ok(expiration) => {
            if expiration.is_expired() {
                return DomainStatus::Expired;
            }

            return DomainStatus::ValidFor(expiration.days());
        }
        Err(err) => DomainStatus::ValidationFailed(err.to_string()),
    }
}
