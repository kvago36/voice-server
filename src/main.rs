use std::pin::Pin;
use std::task::Poll;
use actix::prelude::*;
use actix_web::{App, Error, HttpRequest, HttpResponse, HttpServer, web};
use actix_web_actors::ws;
use futures::stream;
use tonic::client::Grpc;
use tonic::transport::channel::{Channel};
use tonic::service::{Interceptor};
use tonic::Status;
use tonic::transport::{Body};
// {
// "id": "ai-speechkit",
// "address": "transcribe.api.cloud.yandex.net:443"
// },

// id каталога b1g1a228bcepdtc37q7j

// y0__xDRku_EBxjB3RMg0tymrBLzriFcICRVXmgRNGBQUpUIVuXZvg

// {
// "iamToken": "t1.9euelZqXnZPNzciWi5fLlJCXnZHOnO3rnpWaj5rJiZOSlZqcnIvNy5iUlcbl8_c2BBpC-e82Hkxx_d3z93YyF0L57zYeTHH9zef1656VmpKVx8ydlIrHzsibzpzPl8eU7_zF656VmpKVx8ydlIrHzsibzpzPl8eU.W6e0G8cvv067FNhcolUcKG0Kkaef7OAJ5f7YWhA8G1T8uAHUWYjhrIZD1aJFhPAotbZhfnUKfoN4R73F0jOuCg",
// "expiresAt": "2025-02-22T19:33:29.567079113Z"
// }

static TOKEN: &str = "t1.9euelZqXnZPNzciWi5fLlJCXnZHOnO3rnpWaj5rJiZOSlZqcnIvNy5iUlcbl8_c2BBpC-e82Hkxx_d3z93YyF0L57zYeTHH9zef1656VmpKVx8ydlIrHzsibzpzPl8eU7_zF656VmpKVx8ydlIrHzsibzpzPl8eU.W6e0G8cvv067FNhcolUcKG0Kkaef7OAJ5f7YWhA8G1T8uAHUWYjhrIZD1aJFhPAotbZhfnUKfoN4R73F0jOuCg";
static FOLDER_ID: &str = "b1g1a228bcepdtc37q7j";


pub mod api {
    tonic::include_proto!("speechkit.stt.v3");
}

use api::recognizer_client::RecognizerClient;
use api::{StreamingOptions, StreamingRequest, streaming_request};

/// Актор WebSocket-соединения
struct MyWebSocket;

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;
}

/// Обрабатываем сообщения от клиента
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Binary(audio)) => {
                println!("Получено: {:?}", audio);
            }
            Ok(ws::Message::Text(text)) => {
                println!("Получено: {}", text);
                ctx.text(format!("Эхо: {}", text)); // Отправляем эхо-ответ клиенту
            }
            Ok(ws::Message::Close(_)) => {
                println!("Клиент отключился");
            }
            _ => (),
        }
    }
}

/// WebSocket-обработчик
async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(MyWebSocket, &req, stream)
}

struct MyInterceptor;

impl Interceptor for MyInterceptor {
    fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        request.metadata_mut().insert("authorization", TOKEN.parse().unwrap());
        request.metadata_mut().insert("x-folder-id", FOLDER_ID.parse().unwrap());
        // println!("{:?}", request.metadata());
        Ok(request)
    }
}

/// Главный сервер
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let channel = tonic::transport::Endpoint::new("https://stt.api.cloud.yandex.net:443").unwrap().connect().await.unwrap();

    let mut client = RecognizerClient::with_interceptor(
        channel,
        MyInterceptor {},
    );

    // Создаем поток запросов
    let requests = stream::iter(vec![
        StreamingRequest {
            event: Some(streaming_request::Event::SessionOptions(
                StreamingOptions::default()
            ))
        }
    ]);

    let response = client.recognize_streaming(requests).await.unwrap();

    println!("response: {:?}", response.into_inner().message().await.unwrap());

    Ok(())

    // HttpServer::new(|| App::new().route("/ws", web::get().to(ws_index)))
    //     .bind("127.0.0.1:8080")?
    //     .run()
    //     .await
}
