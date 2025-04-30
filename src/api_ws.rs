use actix::{Actor, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::sync::{Arc, Mutex};
use crate::matrix2d::Matrix2D;
use actix::ActorContext;

pub struct Matrix2DWs {
    _matrix: Arc<Mutex<Matrix2D>>,
}

impl Actor for Matrix2DWs {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Matrix2DWs {
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.text("Welcome to Matrix2D WebSocket!");
    }
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(txt)) => {
                ctx.text(format!("Echo: {}", txt));
            }
            Ok(ws::Message::Binary(bin)) => {
                ctx.binary(bin);
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => (),
        }
    }
}

pub async fn ws_matrix2d_handler(
    req: HttpRequest,
    stream: web::Payload,
    matrix: web::Data<Arc<Mutex<Matrix2D>>>,
) -> Result<HttpResponse, Error> {
    let ws = Matrix2DWs {
        _matrix: matrix.get_ref().clone(),
    };
    ws::start(ws, &req, stream)
}
