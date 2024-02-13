use serde::Deserialize;
use url::Url;

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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TrustService {
    trade_name: String,
    pub information_url: Url,
    certificates: String,
}

pub fn trust_service(autorite_du_certificat: &str) -> TrustService {
    let xml = std::fs::read_to_string("tsl_signed.xml").unwrap();
    let parsed = serde_xml_rs::from_str::<TrustServiceStatusList>(&xml).unwrap();

    let trust_services = parsed
        .list
        .list
        .iter()
        .map(|tsp| TrustService {
            trade_name: tsp.info.trade_name.name.first().unwrap().clone(),
            information_url: tsp
                .info
                .information_uri
                .uri
                .first()
                .unwrap()
                .parse()
                .unwrap(),
            certificates: tsp.services.services[0]
                .info
                .digital_identities
                .digital_ids
                .certificates
                .clone(),
        })
        .collect::<Vec<_>>();

    trust_services
        .iter()
        .find(|ts| ts.trade_name == autorite_du_certificat)
        .unwrap()
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_service() {
        let trust_service = trust_service("FR01");

        assert_eq!(trust_service.trade_name, "FR01");
        assert_eq!(
            trust_service.information_url,
            Url::parse("http://cert.pki-2ddoc.ariadnext.fr/pki-2ddoc.der").unwrap()
        );
        assert_eq!(
            trust_service.certificates,
            "MIIFnzCCA4egAwIBAgIIWGqMmtUG62gwDQYJKoZIhvcNAQENBQAwTjENMAsGA1UEAwwERlIwMTEcMBoGA1UECwwTMDAwMiA1MjA3NjkyMjUwMDAyNzESMBAGA1UECgwJQXJpYWRORVhUMQswCQYDVQQGEwJGUjAeFw0xMjA2MjYxNTIyMzRaFw0yMjA2MjYxNTIyMzRaME4xDTALBgNVBAMMBEZSMDExHDAaBgNVBAsMEzAwMDIgNTIwNzY5MjI1MDAwMjcxEjAQBgNVBAoMCUFyaWFkTkVYVDELMAkGA1UEBhMCRlIwggIiMA0GCSqGSIb3DQEBAQUAA4ICDwAwggIKAoICAQCqKLgVoE+hgLBeJ7t6kf2QFslItd6bkRolj3iZlvtVuYV//WOicYKuGWkjW/K2Zv2PySieo676qAsrEbchxwBMPpx12yCQoBckHxCB9mwiG+uyBiN3bukuPKlRxb7i+yNARt3hT+DDUuYqcwat2G4mP8MbHnCsbS2S1jikwWJeCPCa2vhNXSXVr2hNSFlQQEvfQzaaE4hDmwfrFqbyTMUAsTxSV3G6l513KWSY6ZAgMA9lB3KxMcpqvY7mHTClvAUMQYUKa4199NTJIvS6pNF0eLZ+JCAbMoQgMxAKS8VLRW6ovqgtdExD9fr664lAPAMpXFnb7mcLz9ovIhrOM33BZaJ+3zUwqMn5WYOxAPdbPqaf/ap2E3E07v5CeUgFpy4UdkHarUS6MYlbXY14dITwGpxBWyEB3LVpQE9GP1ZAKf4f+tBstS5m08G3xum6wMdpBVDJ3w8ao7KR+jUJcvzk4av4ZphI8Z5AyeTtLdAXFhffX8B12PmeLLPQmF2VK8zSC4MnkKAwU5D6leASDmgHIdN/mpRwWGEQSZqlKa3LQBXWIauNIHqlK777jgNe+EdifLcXIpoiQCVZCU3k+b9WAio0uWywdIZca8aHpKliGLgbOsyXUWIeskc6Gk/JopisdJwE3osGpU+CNU8Jipq4FMvnq4CL7y9FbEWXGDIEQwIDAQABo4GAMH4wHQYDVR0OBBYEFBGxw8fKDCm4PX28b3V6u3DuL9hJMBIGA1UdEwEB/wQIMAYBAf8CAQAwHwYDVR0jBBgwFoAUEbHDx8oMKbg9fbxvdXq7cO4v2EkwGAYDVR0gBBEwDzANBgsrBgEEAYKqUgoCAjAOBgNVHQ8BAf8EBAMCAUYwDQYJKoZIhvcNAQENBQADggIBAE+yUWliolk2HDG2/Iq2rOprnLqe50ixhvA2f5LB4vzcJSwZal9UxWe8TXgydSb693k+Uw+f+jpWj4j9UQmILNkfOzWNNQbm2GWCCTxsgzdAQRsvV7JesueL4JjnX59bqt45YPOcoKo184x5Bc0eb83drrQaJzRA18WKvpDLiHwUIs7rhX799CPG5BQoLRKQpE3hgGjSswPV6+xZ4HHh3+IUjohxZGZTN+r+mIqKIKBqJVHjy4C+x5X+yvmKpT0SUzaZ+rxcDEEyvLbvxFR2j3gtQx7ytmn3aCfY1pzdixmkqx473r6Kinz9NhoZ9RsnxpDOpnLaB2by0mVWPb6z56ki6GkziO2QbllXqW4UmTWymLAd8zt1nhGq/TUcQJBZIt1S1v9IgwGItfvKuMtFprwhba1YVErBKQU/Zb7WJrhzji3JZVWJZelEbp8iBTOZdAqicvVYTUsDql7G8lgQ4szNugeWryeEtSdG4ZGN7nfrrVsb0C0fNswutS7EeFy/ly4L7dDdA8Y/qKhjK1zbLKBimihbPEJqc4up8tF2tjMtJkqY2Oej4FyxS+TOXZjWJ9FWhnly4yxK0MYW82kw3HB5fsBPvEEnbSCZj70TCcvLRdoM48+r7wZvdZVGYxKQ0vr/T/sRICKboFzysRvhflOREXvnHZUL9UVeV8HqTOgo".to_string()
        );
    }
}
