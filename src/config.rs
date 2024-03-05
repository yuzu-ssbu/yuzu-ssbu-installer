//! config.rs
//!
//! Contains Config structures, as well as means of serialising them.

use toml;
use toml::de::Error as TomlError;

use serde_json::{self, Error as SerdeError};

use crate::sources::get_by_name;
use crate::sources::types::Release;

/// Description of the source of a package.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PackageSource {
    pub name: String,
    #[serde(rename = "match")]
    pub match_regex: String,
    #[serde(default)]
    pub allow_version_control: bool,
    pub config: toml::Value,
    #[serde(skip)]
    cached_releases: Option<Result<Vec<Release>, String>>,
}

/// Describes if/how a shortcut should be built for a package.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PackageShortcut {
    pub name: String,
    pub relative_path: String,
    pub description: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub has_desktop_shortcut: bool,
    #[serde(default)]
    pub icon_relative_path: String,
}

/// Extra description for authentication and authorization state for a package
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PackageExtendedDescription {
    #[serde(default)]
    pub no_action_description: Option<String>,
    #[serde(default)]
    pub need_authentication_description: Option<String>,
    #[serde(default)]
    pub need_link_description: Option<String>,
    #[serde(default)]
    pub need_subscription_description: Option<String>,
    #[serde(default)]
    pub need_reward_tier_description: Option<String>,
}

/// Describes a overview of a individual package.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PackageDescription {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub icon: Option<String>,
    pub default: Option<bool>,
    #[serde(default)]
    pub launch_after_install: bool,
    pub source: PackageSource,
    #[serde(default)]
    pub shortcuts: Vec<PackageShortcut>,
    #[serde(default)]
    pub requires_authorization: Option<bool>,
    #[serde(default)]
    pub is_new: Option<bool>,
    #[serde(default)]
    pub extended_description: Option<PackageExtendedDescription>,
}

/// Configuration for validating the JWT token
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JWTValidation {
    pub iss: Option<String>,
    // This can technically be a Vec as well, but thats a pain to support atm
    pub aud: Option<String>,
}

/// The configuration for this release.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthenticationConfig {
    pub pub_key_base64: String,
    pub auth_url: String,
    pub validation: Option<JWTValidation>,
}

/// Describes the application itself.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BaseAttributes {
    pub name: String,
    pub target_url: String,
    #[serde(default)]
    pub recovery: bool,
}

impl BaseAttributes {
    /// Serialises as a JSON string.
    pub fn to_json_str(&self) -> Result<String, SerdeError> {
        serde_json::to_string(self)
    }

    /// Builds a configuration from a specified TOML string.
    pub fn from_toml_str(contents: &str) -> Result<Self, TomlError> {
        toml::from_str(contents)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub installing_message: String,
    /// URL to a new updater, if required
    #[serde(default)]
    pub new_tool: Option<String>,
    pub packages: Vec<PackageDescription>,
    #[serde(default)]
    pub hide_advanced: bool,
    #[serde(default)]
    pub authentication: Option<AuthenticationConfig>,
}

impl Config {
    /// Serialises as a JSON string.
    pub fn to_json_str(&self) -> Result<String, SerdeError> {
        serde_json::to_string(self)
    }

    /// Builds a configuration from a specified TOML string.
    pub fn from_toml_str(contents: &str) -> Result<Self, TomlError> {
        toml::from_str(contents)
    }
}

impl PackageSource {
    pub fn get_current_releases(&mut self) -> Result<Vec<Release>, String> {
        if self.cached_releases.is_none() {
            self.cached_releases = Some(self.get_and_cache_current_releases());
        }
        self.cached_releases
            .take()
            .unwrap_or(Err("No cached releases".to_string()))
    }

    fn get_and_cache_current_releases(&self) -> Result<Vec<Release>, String> {
        let package_handler = match get_by_name(&self.name) {
            Some(v) => v,
            _ => return Err(format!("Handler {} not found", self.name)),
        };

        package_handler.get_current_releases(&self.config)
    }
}
