//! gdrive/mod.rs
//!
//! Contains the Google Drive API implementation of a release source.
use std::collections::HashMap;

use reqwest::header::USER_AGENT;
use reqwest::StatusCode;

use scraper::{Html, Selector};
use serde_json;

use crate::sources::types::*;


use crate::http::build_client;

pub struct GDriveReleases {}

/// The configuration for this release.
#[derive(Serialize, Deserialize)]
struct GDriveConfig {
    file_id: String,
}

impl GDriveReleases {
    pub fn new() -> Self {
        GDriveReleases {}
    }
}

impl ReleaseSource for GDriveReleases {
    fn get_current_releases(&self, config: &TomlValue) -> Result<Vec<Release>, String> {
        // Reparse our Config as strongly typed
        let config: GDriveConfig = match config.clone().try_into() {
            Ok(v) => v,
            Err(v) => return Err(format!("Failed to parse release config: {:?}", v)),
        };

        let mut results: Vec<Release> = Vec::new();

        // Build the HTTP client up
        let client = build_client()?;
        let mut response = client
            .get(&format!(
                "https://drive.usercontent.google.com/download?id={}",
                config.file_id.as_str()
            ))
            .send()
            .map_err(|x| format!("Error while sending HTTP request: {:?}", x))?;

        match response.status() {
            StatusCode::OK => {}
            StatusCode::FORBIDDEN => {
                return Err(
                    "Google Drive is rate limiting you. Try moving to a internet connection \
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
        
        let document = Html::parse_document(&body);
        let selector = Selector::parse("#download-form").expect("Unable to parse html response");

        let mut params = vec![("id", config.file_id.as_str())]; 
        if let Some(form_element) = document.select(&selector).next() {
            let hidden_input_selector = Selector::parse("input[type=hidden]").unwrap();
            for hidden_input in form_element.select(&hidden_input_selector) {
                let name = hidden_input.value().attr("name").expect("name tag not found");
                let value = hidden_input.value().attr("value").expect("value tag not found");
                info!("Found hidden input: name='{}', value='{}'", name, value);
                params.push((name, value));
            }
        } else {
            info!("Element with ID 'download-form' not found");
        }
        let url = reqwest::Url::parse_with_params("https://drive.usercontent.google.com/download", &params).expect("Unable to generate get URL");
        // let response = client.get(url).send().map_err(|x| format!("Error while sending HTTP request: {:?}", x))?;

        // match response.status() {
        //     StatusCode::OK => {}
        //     StatusCode::FORBIDDEN => {
        //         return Err(
        //             "Google Drive is rate limiting you. Try moving to a internet connection \
        //              that isn't shared, and/or disabling VPNs."
        //                 .to_string(),
        //         );
        //     }
        //     _ => {
        //         return Err(format!("Bad status code: {:?}.", response.status()));
        //     }
        // }

        let name = "yuzu_gdrive_test.zip";
        let files = vec![File {
            name: name.to_string(),
            url: url.to_string(),
            requires_authorization: false,
        }];

        results.push(Release {
            name: name.to_string(),
            version: Version::new_number(1),
            files,
        });
        
        Ok(results)
    }
}
