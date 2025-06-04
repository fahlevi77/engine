#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceProtocol {
    TCP,
    UDP,
    SCTP,
}

#[derive(Debug, Clone)]
pub struct ServiceDeploymentInfo {
    pub service_protocol: ServiceProtocol,
    pub is_pulling: bool,
    pub secured: bool,
    pub port: u16,
    pub deployment_properties: std::collections::HashMap<String, String>,
}

impl ServiceDeploymentInfo {
    pub fn new(service_protocol: ServiceProtocol, port: u16, secured: bool) -> Self {
        Self {
            service_protocol,
            is_pulling: false,
            secured,
            port,
            deployment_properties: std::collections::HashMap::new(),
        }
    }

    pub fn new_default(port: u16, secured: bool) -> Self {
        Self::new(ServiceProtocol::TCP, port, secured)
    }

    pub fn default() -> Self {
        Self {
            service_protocol: ServiceProtocol::TCP,
            is_pulling: true,
            secured: false,
            port: 0,
            deployment_properties: std::collections::HashMap::new(),
        }
    }

    pub fn add_deployment_properties(&mut self, props: std::collections::HashMap<String, String>) {
        for (k, v) in props.into_iter() {
            self.deployment_properties.entry(k).or_insert(v);
        }
    }
}
