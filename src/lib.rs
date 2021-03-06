use actix_rt::time::delay_for;
use actix_web::{dev::Server, middleware, web, App, HttpRequest, HttpServer};

use std::{io, net::TcpListener, time::Duration};

#[derive(Debug, Clone, Default)]
pub struct ApiManager {
    server: Option<Server>,
}

async fn ping(_request: HttpRequest) -> &'static str {
    // Emulate long response.
    delay_for(Duration::from_millis(250)).await;

    "pong"
}

impl ApiManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn start_server(&mut self, listener: TcpListener) -> io::Result<()> {
        assert!(
            self.server.is_none(),
            "An attempt to start another server instance"
        );

        log::trace!("Start server requested");

        let addr = listener.local_addr()?;
        let server = HttpServer::new(|| {
            App::new()
                .wrap(middleware::Logger::default())
                .service(web::resource("/ping").to(ping))
        })
        .disable_signals()
        .listen(listener)?
        .run();
        self.server = Some(server);

        log::info!("Service instance has been started on {}", addr);

        Ok(())
    }

    pub async fn stop_server(&mut self) {
        log::trace!("Stop server requested");

        if let Some(server) = self.server.take() {
            server.stop(false).await;

            log::info!("Service instance has been stopped");
        }
    }
}
