//! Copyright (c) 2025 Trung Do <dothanhtrung@pm.me>.

use crate::api::SearchQuery;
use crate::ConfigData;
use actix_files::Files;
use actix_web::rt::time::interval;
use actix_web::web::Data;
use actix_web::{get, web, HttpResponse, Responder};
use actix_web_lab::extract::Query;
use actix_web_lab::{
    sse::{self, Sse},
    util::InfallibleStream,
};
use futures_util::future;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tera::Tera;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{error, info, warn};

pub fn scope_config(cfg: &mut web::ServiceConfig) {
    let tera = Tera::new("res/html/**/*").unwrap();

    cfg.app_data(Data::new(tera))
        .service(index)
        .service(get_item)
        .service(maintenance)
        .service(civitai)
        .service(tag)
        .service(setting)
        .service(job)
        .service(event_stream)
        .service(Files::new("/assets", "res/assets"))
        .service(Files::new("/css", "res/css"))
        .service(Files::new("/js", "res/js"));
}

#[derive(Serialize, Deserialize)]
pub enum EventMsgLevel {
    Info,
    Warn,
    Error,
}

#[derive(Serialize)]
pub struct EventMsg {
    pub level: EventMsgLevel,
    pub msg: String,
}

pub struct Broadcaster {
    inner: Mutex<BroadcasterInner>,
}

#[derive(Debug, Clone, Default)]
pub struct BroadcasterInner {
    clients: Vec<mpsc::Sender<sse::Event>>,
}
impl Broadcaster {
    /// Constructs new broadcaster and spawns ping loop.
    pub fn create() -> Arc<Self> {
        let this = Arc::new(Broadcaster {
            inner: Mutex::new(BroadcasterInner::default()),
        });

        Broadcaster::spawn_ping(Arc::clone(&this));

        this
    }

    /// Pings clients every 10 seconds to see if they are alive and remove them from the broadcast
    /// list if not.
    fn spawn_ping(this: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));

            loop {
                interval.tick().await;
                this.remove_stale_clients().await;
            }
        });
    }

    /// Removes all non-responsive clients from broadcast list.
    async fn remove_stale_clients(&self) {
        let clients = self.inner.lock().clients.clone();

        let mut ok_clients = Vec::new();

        for client in clients {
            if client.send(sse::Event::Comment("ping".into())).await.is_ok() {
                ok_clients.push(client.clone());
            }
        }

        self.inner.lock().clients = ok_clients;
    }

    /// Registers client with broadcaster, returning an SSE response body.
    pub async fn new_client(&self) -> Sse<InfallibleStream<ReceiverStream<sse::Event>>> {
        let (tx, rx) = mpsc::channel(10);

        tx.send(sse::Data::new("connected").into()).await.unwrap();

        self.inner.lock().clients.push(tx);

        Sse::from_infallible_receiver(rx)
    }

    /// Broadcasts `msg` to all clients.
    pub async fn broadcast(&self, msg: EventMsg) {
        let clients = self.inner.lock().clients.clone();

        if let Ok(msg) = sse::Data::new_json(msg) {
            let send_futures = clients.iter().map(|client| client.send(msg.clone().into()));

            // try to send to all clients, ignoring failures
            // disconnected clients will get swept up by `remove_stale_clients`
            let _ = future::join_all(send_futures).await;
        }
    }

    pub async fn info(&self, msg: &str) {
        info!(msg);
        let msg = EventMsg {
            level: EventMsgLevel::Info,
            msg: msg.to_string(),
        };
        self.broadcast(msg).await;
    }

    pub async fn warn(&self, msg: &str) {
        warn!(msg);
        let msg = EventMsg {
            level: EventMsgLevel::Warn,
            msg: msg.to_string(),
        };
        self.broadcast(msg).await;
    }

    pub async fn error(&self, msg: &str) {
        error!(msg);
        let msg = EventMsg {
            level: EventMsgLevel::Error,
            msg: msg.to_string(),
        };
        self.broadcast(msg).await;
    }
}

#[get("/events")]
async fn event_stream(broadcaster: Data<Broadcaster>) -> impl Responder {
    broadcaster.new_client().await
}

#[get("/")]
async fn index(tmpl: Data<Tera>, query_params: Query<SearchQuery>) -> impl Responder {
    let mut ctx = tera::Context::new();
    ctx.insert("search", &query_params.search);

    match tmpl.render("index.html", &ctx) {
        Ok(template) => HttpResponse::Ok().content_type("text/html").body(template),
        Err(e) => HttpResponse::Ok()
            .content_type("text/html")
            .body(format!("Template error: {e}")),
    }
}

#[get("/item/{id}")]
async fn get_item(tmpl: Data<Tera>, id: web::Path<i64>) -> impl Responder {
    let mut ctx = tera::Context::new();
    ctx.insert("id", &id.into_inner());
    match tmpl.render("item.html", &ctx) {
        Ok(template) => HttpResponse::Ok().content_type("text/html").body(template),
        Err(e) => HttpResponse::Ok()
            .content_type("text/html")
            .body(format!("Template error: {e}")),
    }
}

#[get("/maintenance")]
async fn maintenance(tmpl: Data<Tera>) -> impl Responder {
    let ctx = tera::Context::new();
    match tmpl.render("maintenance.html", &ctx) {
        Ok(template) => HttpResponse::Ok().content_type("text/html").body(template),
        Err(e) => HttpResponse::Ok()
            .content_type("text/html")
            .body(format!("Template error: {e}")),
    }
}

#[get("/civitai")]
async fn civitai(tmpl: Data<Tera>, config_data: Data<ConfigData>) -> impl Responder {
    let mut ctx = tera::Context::new();
    let config = config_data.config.read().await;
    ctx.insert("config", &config.civitai);
    match tmpl.render("civitai.html", &ctx) {
        Ok(template) => HttpResponse::Ok().content_type("text/html").body(template),
        Err(e) => HttpResponse::Ok()
            .content_type("text/html")
            .body(format!("Template error: {e}")),
    }
}

#[get("/tag/{name}")]
async fn tag(tmpl: Data<Tera>) -> impl Responder {
    let ctx = tera::Context::new();
    match tmpl.render("tag.html", &ctx) {
        Ok(template) => HttpResponse::Ok().content_type("text/html").body(template),
        Err(e) => HttpResponse::Ok()
            .content_type("text/html")
            .body(format!("Template error: {e}")),
    }
}

#[get("/setting")]
async fn setting(tmpl: Data<Tera>) -> impl Responder {
    let ctx = tera::Context::new();
    match tmpl.render("config.html", &ctx) {
        Ok(template) => HttpResponse::Ok().content_type("text/html").body(template),
        Err(e) => HttpResponse::Ok()
            .content_type("text/html")
            .body(format!("Template error: {e}")),
    }
}

#[get("/job")]
async fn job(tmpl: Data<Tera>) -> impl Responder {
    let ctx = tera::Context::new();
    match tmpl.render("job.html", &ctx) {
        Ok(template) => HttpResponse::Ok().content_type("text/html").body(template),
        Err(e) => HttpResponse::Ok()
            .content_type("text/html")
            .body(format!("Template error: {e}")),
    }
}
