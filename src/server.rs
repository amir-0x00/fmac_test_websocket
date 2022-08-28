use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use colored::Colorize;
use once_cell::sync::Lazy;
use tokio::sync::{mpsc::UnboundedSender, Mutex};

use std::{
    sync::RwLock,
    time::{Duration, Instant},
};

use actix::prelude::*;

// pub const DEFAULT_IP: &str = "94.237.92.96";
pub const DEFAULT_IP: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 6379;

static WEB_SCOKET_LIST: Lazy<RwLock<Vec<Addr<MyWebSocket>>>> = Lazy::new(|| {
    let list = Vec::new();
    RwLock::new(list)
});

// static SERVER_SENDER: Lazy<Mutex<Option<UnboundedSender<SenderMsg>>>> =
//     Lazy::new(|| Mutex::new(None));

#[derive(Debug, Clone)]
pub enum SenderMsg {
    Disconnected,
    Text(String),
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            // websocket route
            .service(web::resource("/ws").route(web::get().to(ws_index)))
            // enable logger
            .wrap(middleware::Logger::default())
    })
    .bind((DEFAULT_IP, DEFAULT_PORT))?
    .run()
    .await
}

/// WebSocket handshake and start `MyWebSocket` actor.
async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let ws_id = req
        .headers()
        .get("sec-websocket-key")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    ws::start(MyWebSocket::new(ws_id), &req, stream)
}

/// websocket connection is long running connection, it easier
/// to handle with an actor
pub struct MyWebSocket {
    id: String,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
}

impl MyWebSocket {
    pub fn new(id: String) -> Self {
        Self {
            id,
            hb: Instant::now(),
        }
    }
}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        WEB_SCOKET_LIST.write().unwrap().push(ctx.address());
        println!("{}: {}", "[io] Connect".green(), self.id.to_string().cyan());
    }
}

/// Handler for `ws::Message`
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // process websocket messages
        println!("WS: {msg:?}");
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
    fn finished(&mut self, ctx: &mut Self::Context) {
        println!(
            "{}: {}",
            "[io] Disconnect".red(),
            self.id.to_string().cyan()
        );
        WEB_SCOKET_LIST
            .write()
            .unwrap()
            .retain(|addr| addr != &ctx.address());
        // super.finished(ctx);
    }
}
