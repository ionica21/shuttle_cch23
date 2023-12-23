use actix::{Actor, StreamHandler};
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

struct PingPongServer {
    game_started: bool,
}

impl Actor for PingPongServer {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for PingPongServer {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(ws::Message::Text(text)) = msg {
            match text.to_string().as_str() {
                "serve" => self.game_started = true,
                "ping" if self.game_started => ctx.text("pong"),
                _ => (), // Ignore other messages
            }
        }
    }
}

#[get("/19/ws/ping")]
async fn ping_pong(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(
        PingPongServer {
            game_started: false,
        },
        &req,
        stream,
    )
}
