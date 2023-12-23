use crate::AppState;
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, StreamHandler};
use actix_web::{get, post, web, Error, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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

#[derive(Message, Deserialize, Serialize, Clone)]
#[rtype(result = "()")]
struct Tweet {
    user: String,
    message: String,
}

#[derive(Message)]
#[rtype(result = "()")]
struct Connect {
    addr: Addr<WsConnection>,
    user: String,
}

#[derive(Message)]
#[rtype(result = "()")]
struct Disconnect {
    addr: Addr<WsConnection>,
}

pub struct Room {
    users: HashMap<Addr<WsConnection>, String>,
    view_count: Arc<Mutex<usize>>,
}

impl Actor for Room {
    type Context = Context<Self>;
}

impl Room {
    fn new(view_count: Arc<Mutex<usize>>) -> Self {
        Room {
            users: HashMap::new(),
            view_count,
        }
    }
}

impl Handler<Connect> for Room {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        self.users.insert(msg.addr, msg.user);
    }
}

impl Handler<Disconnect> for Room {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        self.users.remove(&msg.addr);
    }
}

impl Handler<Tweet> for Room {
    type Result = ();

    fn handle(&mut self, msg: Tweet, _ctx: &mut Context<Self>) {
        for (addr, _) in self.users.iter() {
            let mut view_count = self.view_count.lock().unwrap();
            *view_count += 1;
            addr.do_send(msg.clone());
        }
    }
}

struct WsConnection {
    user: String,
    room_addr: Addr<Room>,
}

impl Actor for WsConnection {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.room_addr.do_send(Connect {
            addr: ctx.address(),
            user: self.user.clone(),
        });
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        self.room_addr.do_send(Disconnect {
            addr: ctx.address(),
        });
    }
}

impl Handler<Tweet> for WsConnection {
    type Result = ();

    fn handle(&mut self, msg: Tweet, ctx: &mut ws::WebsocketContext<Self>) {
        let tweet_json = serde_json::to_string(&msg).unwrap_or_default();
        ctx.text(tweet_json);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConnection {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, _ctx: &mut Self::Context) {
        if let Ok(ws::Message::Text(text)) = item {
            if let Ok(message_json) = serde_json::from_str::<Value>(&text.to_string()) {
                if let Some(Value::String(message)) = message_json.get("message") {
                    if message.len() <= 128 {
                        let tweet = Tweet {
                            user: self.user.clone(),
                            message: message.clone(),
                        };
                        self.room_addr.do_send(tweet);
                    }
                }
            }
        }
    }
}

#[get("/19/ws/room/{room}/user/{user}")]
async fn room(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<(i32, String)>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let (room, user) = path.into_inner();
    let mut rooms = data.rooms.lock().await;

    let room_addr = rooms
        .entry(room)
        .or_insert_with(|| Room::new(data.view_count.clone()).start())
        .clone();
    ws::start(WsConnection { user, room_addr }, &req, stream)
}

#[post("/19/reset")]
async fn reset_views(data: web::Data<AppState>) -> impl Responder {
    if let Ok(mut view_count) = data.view_count.lock() {
        *view_count = 0;
        return HttpResponse::Ok();
    }
    HttpResponse::InternalServerError()
}

#[get("/19/views")]
async fn get_views(data: web::Data<AppState>) -> impl Responder {
    if let Ok(view_count) = data.view_count.lock() {
        return HttpResponse::Ok().body(view_count.to_string());
    }
    HttpResponse::InternalServerError().body("Failed to get view count!")
}
