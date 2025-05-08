//! gitlab/mod.rs
//!
//! Contains the Gitlab API implementation of a release source.

use chrono::{DateTime, Utc};
use reqwest::header::USER_AGENT;
use reqwest::StatusCode;

use serde_json;

use crate::sources::types::*;

use crate::http::build_client;

pub struct GitlabReleases {}

/// The configuration for this release.
#[derive(Serialize, Deserialize)]
struct GitlabConfig {
    base_url: String,
    project_id: u64,
}

impl GitlabReleases {
    pub fn new() -> Self {
        GitlabReleases {}
    }
}

impl ReleaseSource for GitlabReleases {
    fn get_current_releases(&self, config: &TomlValue) -> Result<Vec<Release>, String> {
        // Reparse our Config as strongly typed
        let config: GitlabConfig = match config.clone().try_into() {
            Ok(v) => v,
            Err(v) => return Err(format!("Failed to parse release config: {:?}", v)),
        };

        let mut results: Vec<Release> = Vec::new();

        // Build the HTTP client up
        let client = build_client()?;
        let mut response = client
            .get(&format!(
                "https://{}/api/v4/projects/{}/releases",
                config.base_url,
                config.project_id
            ))
            .header(USER_AGENT, "liftinstall (j-selby)")
            .send()
            .map_err(|x| format!("Error while sending HTTP request: {:?}", x))?;

        match response.status() {
            StatusCode::OK => {}
            StatusCode::FORBIDDEN => {
                return Err(
                    "Git is rate limiting you. Try moving to a internet connection \
                     that isn't shared, and/or disabling VPNs."
                        .to_string(),
                );
            }
            _ => {
                return Err(format!("Bad status code: {:?}.", response.status()));
            }
        }

        let body = response
            .text()
            .map_err(|x| format!("Failed to decode HTTP response body: {:?}", x))?;

        let result: serde_json::Value = serde_json::from_str(&body)
            .map_err(|x| format!("Failed to parse response: {:?}", x))?;

        let result: &Vec<serde_json::Value> = result
            .as_array()
            .ok_or_else(|| "Response was not an array!".to_string())?;

        // Parse JSON from server
        for entry in result.iter() {
            let mut files = Vec::new();

            let id: u64 = match entry["released_at"].as_str() {
                Some(r) => {
                    let datetime: DateTime<Utc> = r.parse().expect("Failed to parse datetime");
                    datetime.timestamp() as u64
                },
                None => return Err("JSON payload missing information about ID".to_string()),
            };

            let name: String = match entry["name"].as_str() {
                Some(v) => v.to_string(),
                None => return Err("JSON payload missing information about name".to_string()),
            };

            let assets = match entry["assets"].as_object() {
                Some(v) => v,
                None => return Err("JSON payload not an object".to_string()),
            };

            let links = match assets["links"].as_array() {
                Some(v) => v,
                None => return Err("JSON payload not an array".to_string()),
            };

            for link in links.iter() {
                let string = match link["name"].as_str() {
                    Some(v) => v,
                    None => {
                        return Err(
                            "JSON payload missing information about release name".to_string()
                        );
                    }
                };

                let url = match link["url"].as_str() {
                    Some(v) => v,
                    None => {
                        return Err(
                            "JSON payload missing information about release URL".to_string()
                        );
                    }
                };

                files.push(File {
                    name: string.to_string(),
                    url: url.to_string(),
                    requires_authorization: false,
                });
            }

            results.push(Release {
                name: name,
                version: Version::new_number(id),
                files,
            });
        }

        Ok(results)
    }
}
