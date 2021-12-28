use std::io::Read;
use embedded_svc::httpd::Method;
use embedded_svc::httpd::registry::{MiddlewareRegistry, Registry};
use esp_idf_svc;
use esp_idf_svc::httpd::Configuration;
use bal::server::{ServerResource, Error};


type WrappedServer = esp_idf_svc::httpd::Server;
pub struct EspWebServer{
    server: Option<WrappedServer>
}
impl EspWebServer {
    pub fn new()-> EspWebServer {
        EspWebServer{server: None}
    }
}
impl ServerResource for EspWebServer {
    fn create_server(&mut self, registry: MiddlewareRegistry) -> Result<(),Error>{
        let mut server_reg = esp_idf_svc::httpd::ServerRegistry::new();

        for handle in registry.apply_middleware(){
            let server_reg_builder = server_reg.at(handle.uri().as_ref());
            server_reg = match handle.method(){
                Method::Put => {server_reg_builder.put(handle.handler()).map_err(|_|Error::Register)?}
                Method::Get => {server_reg_builder.get(handle.handler()).map_err(|_|Error::Register)?}
                _ => {return Err(Error::Undefined)}
            }

        }
        self.server = Some(server_reg.start(&Configuration::default()).map_err(|_|Error::Start)?);
        Ok(())

    }
}
pub fn create_server(registry: embedded_svc::httpd::registry::MiddlewareRegistry) {
}