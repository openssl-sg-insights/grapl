/*
 * Nomad
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 1.1.4
 * Contact: support@hashicorp.com
 * Generated by: https://openapi-generator.tech
 */

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct EphemeralDisk {
    #[serde(rename = "Migrate", skip_serializing_if = "Option::is_none")]
    pub migrate: Option<bool>,
    #[serde(rename = "SizeMB", skip_serializing_if = "Option::is_none")]
    pub size_mb: Option<i32>,
    #[serde(rename = "Sticky", skip_serializing_if = "Option::is_none")]
    pub sticky: Option<bool>,
}

impl EphemeralDisk {
    pub fn new() -> EphemeralDisk {
        EphemeralDisk {
            migrate: None,
            size_mb: None,
            sticky: None,
        }
    }
}