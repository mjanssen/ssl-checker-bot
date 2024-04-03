use std::{collections::HashMap, time::Duration};

use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE, USER_AGENT};
use ssl_expiration2::SslExpiration;
use prettytable::{Table, row, format};

pub enum CertificateStatus {
    ValidFor(i32),
    Expired,
    ValidationFailed(String),
}

type DomainStatus = Result<u16, reqwest::Error>;

pub struct Checker {
    domains: Vec<String>,
    cron: bool,
}

type DomainStatusList<'a> = HashMap<&'a str, (CertificateStatus, DomainStatus)>;

struct ErrorState {
    error_count: u16,
    warning_count: u16,
}

impl Checker {
    pub fn new<'b>(domains: Vec<String>, cron: bool) -> Self {
        Self { domains, cron }
    }

    pub async fn check_domain_status(&self, domain: &str) -> DomainStatus {
        let domain = format!("https://{domain}");
        let client = reqwest::Client::new();

        let response = client
        .get(domain)
        .timeout(Duration::new(10, 0))
        .header(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36")
        .header(ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9")
        .header(ACCEPT_LANGUAGE, "en-GB,en;q=0.6")
        .header("sec-ch-ua", "\"Brave\";v=\"113\", \"Chromium\";v=\"113\", \"Not-A.Brand\";v=\"24\"")
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", "\"macOS\"")
        .header("sec-fetch-site", "none")
        .header("sec-fetch-mod", "")
        .header("sec-fetch-user", "?1")
        .header("sec-fetch-mode", "navigate")
        .send()
        .await?;

        let status_code = response.status().as_u16();
        Ok(status_code)
    }

    pub fn check_certificate(&self, domain_name: &str) -> CertificateStatus {
        match SslExpiration::from_domain_name(domain_name) {
            Ok(expiration) => {
                if expiration.is_expired() {
                    return CertificateStatus::Expired;
                }

                return CertificateStatus::ValidFor(expiration.days());
            }
            Err(err) => CertificateStatus::ValidationFailed(err.to_string()),
        }
    }

    pub async fn get_domain_statusses(&self) -> DomainStatusList {
        let mut domains_and_status: DomainStatusList = HashMap::new();

        for domain in self.domains.iter() {
            let certificate_status = self.check_certificate(&domain);
            let request_status = self.check_domain_status(&domain).await;

            domains_and_status.insert(domain, (certificate_status, request_status));
        }

        domains_and_status
    }

    pub fn parse_statusses_to_string(&self, statusses: &DomainStatusList) -> String {
        let mut error_state = ErrorState {
            error_count: 0,
            warning_count: 0,
        };

        // Create the table
        let mut table = Table::new();

        // Set table style
        table.set_format(*format::consts::FORMAT_NO_COLSEP);

        // Set table titles
        table.set_titles(row![c->"Domain", c->"SSL Status", c->"HTTP Status"]);

        let _ = statusses
            .iter()
            .for_each(|(domain, (domain_status, request_status))| {
                let cert_status: String = match domain_status {
                    CertificateStatus::Expired => {
                        error_state.error_count += 1;
                        "⌛️ Certificate expired".to_string()
                    }
                    CertificateStatus::ValidationFailed(error) => {
                        error_state.error_count += 1;
                        format!("🛑 {error}")
                    }
                    CertificateStatus::ValidFor(days) => {
                        let message: String = match days {
                            x if x < &2 => {
                                error_state.warning_count += 1;
                                format!("⚠️ Certificate expires soon\n{days} days left")
                            }
                            x if x < &14 => {
                                error_state.warning_count += 1;
                                format!("⚠️ Certificate valid\n{days} days left")
                            }
                            _ => format!("✅ Certificate valid\n{days} days left"),
                        };

                        message
                    }
                };

                let request_status_message: String = match request_status {
                    Ok(status) => format!("✅ {} OK", status),
                    Err(err) => {
                        error_state.error_count += 1;
                        let error = err
                            .to_string()
                            .split(":")
                            .last()
                            .unwrap_or("Request error")
                            .trim()
                            .trim_matches('.')
                            .to_string();

                        format!("🛑 {error}")
                    }
                };

                table.add_row(row![domain, cert_status, request_status_message]);
            });

        let cron_msg = match self.cron {
            true => "Your daily report: ",
            false => "",
        };

        format!(
            "{cron_msg}Found {} warning(s) / {} error(s)\n\nResult Details:\n<pre language=\"text\">\n{table}\n</pre>",
            error_state.warning_count, error_state.error_count
        )
    }
}
