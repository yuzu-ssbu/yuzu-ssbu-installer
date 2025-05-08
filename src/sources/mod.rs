//! sources/mod.rs
//!
//! Contains backends to various release distribution services.

pub mod types;

pub mod github;

pub mod gitlab;

pub mod patreon;

use self::types::ReleaseSource;

/// Returns a ReleaseSource by a name, if possible
pub fn get_by_name(name: &str) -> Option<Box<dyn ReleaseSource>> {
    match name {
        "github" => Some(Box::new(github::GithubReleases::new())),
        "gitlab" => Some(Box::new(gitlab::GitlabReleases::new())),
        "patreon" => Some(Box::new(patreon::PatreonReleases::new())),
        _ => None,
    }
}
