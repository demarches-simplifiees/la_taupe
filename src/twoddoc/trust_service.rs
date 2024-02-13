use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct TrustServiceStatusList {
    #[serde(rename = "TrustServiceProviderList")]
    list: TrustServiceProviderList,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct TrustServiceProviderList {
    #[serde(rename = "TrustServiceProvider", default)]
    list: Vec<TrustServiceProvider>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct TrustServiceProvider {
    #[serde(rename = "TSPInformation")]
    info: TSPInformation,
    #[serde(rename = "TSPServices")]
    services: TSPServices,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct TSPInformation {
    #[serde(rename = "TSPTradeName")]
    trade_name: TSPTradeName,
    #[serde(rename = "TSPInformationURI")]
    information_uri: TSPInformationURI,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct TSPTradeName {
    #[serde(rename = "Name", default)]
    name: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct TSPInformationURI {
    #[serde(rename = "URI", default)]
    uri: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct TSPServices {
    #[serde(rename = "TSPService", default)]
    services: Vec<TSPService>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct TSPService {
    #[serde(rename = "ServiceInformation")]
    info: ServiceInformation,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct ServiceInformation {
    #[serde(rename = "ServiceDigitalIdentity")]
    digital_identities: ServiceDigitalIdentity,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct ServiceDigitalIdentity {
    #[serde(rename = "DigitalId")]
    digital_ids: DigitalId,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct DigitalId {
    #[serde(rename = "X509Certificate")]
    certificates: String,
}
