use embedded_svc::httpd::registry::Registry;
use esp_idf_svc::httpd as idf;

fn create_server() {

let server = idf::ServerRegistry::new()
.at("/state")
.get(|_| Ok("Eddig is siman meg tudtam volna!".into()))?
.at("/foo")
.get(|_| bail!("Boo, something happened!"))?
.at("/bar")
.get(|_| {
Response::new(403)
.status_message("No permissions")
.body("You have no permissions to access this page".into())
.into()
})?;

server.start(&Default::default())
}
