//! frontend/rest/services/package_versions.rs
//!
//! The /api/packages-versions call returns all available release versions for a package.

use crate::frontend::rest::services::Future;
use crate::frontend::rest::services::Request;
use crate::frontend::rest::services::Response;
use crate::frontend::rest::services::WebService;
use crate::sources::types::Release;

use hyper::header::{ContentLength, ContentType};

use crate::logging::LoggingErrors;

use url::Url;

use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct PackageVersionInfo {
    version: String,
    name: String,
}

use super::default_future;

pub fn handle(service: &WebService, req: Request) -> Future {
    let framework_ref = service
        .framework
        .read()
        .log_expect("InstallerFramework has been dirtied");

    let valid_url_str = format!("http://localhost{}", req.uri());
    let url = Url::parse(valid_url_str.as_str()).expect("Unable to parse query params from uri");
    let hash_query: HashMap<_, _> = url.query_pairs().into_owned().collect();

    let package_name = hash_query
        .get("package_name")
        .expect("Unable to find package name in url query params");

    let mut current_releases: Result<Vec<Release>, String> = Result::Ok(Vec::new());
    for mut p in framework_ref
        .get_config()
        .expect("InstallerFramework config should be loaded by now")
        .packages
    {
        if &p.name == package_name {
            current_releases = p.source.get_current_releases();
        }
    }

    let mut available_versions = Vec::new();
    for release in current_releases.unwrap() {
        available_versions.push(PackageVersionInfo {
            version: release.version.to_string(),
            name: release.name,
        });
    }

    let json_string = serde_json::to_string(&available_versions)
        .expect("Unable to convert version info to json string");

    default_future(
        Response::new()
            .with_header(ContentLength(json_string.len() as u64))
            .with_header(ContentType::json())
            .with_body(json_string),
    )
}
