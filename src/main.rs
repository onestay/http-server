use std::error::Error;
use http_server::server::Server;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let mut server = Server::new("127.0.0.1:8080")?;
    server.run()?;
    Ok(())
}
