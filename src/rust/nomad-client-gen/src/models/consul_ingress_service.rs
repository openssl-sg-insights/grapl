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
pub struct ConsulIngressService {
    #[serde(rename = "Hosts", skip_serializing_if = "Option::is_none")]
    pub hosts: Option<Vec<String>>,
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl ConsulIngressService {
    pub fn new() -> ConsulIngressService {
        ConsulIngressService {
            hosts: None,
            name: None,
        }
    }
}