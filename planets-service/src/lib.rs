#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate strum;

use std::sync::Arc;

use actix_web::{guard, HttpRequest, HttpResponse, Result, web};
use async_graphql::{Context, Schema};
use async_graphql::http::{GraphQLPlaygroundConfig, playground_source};
use async_graphql_actix_web::{Request, Response, WSSubscription};
use dataloader::non_cached::Loader;
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};

use crate::graphql::{AppSchema, DetailsBatchLoader, Mutation, Query, Subscription};
use crate::persistence::connection::PgPool;

embed_migrations!();

pub mod graphql;
pub mod kafka;
pub mod persistence;

pub fn configure_service(cfg: &mut web::ServiceConfig) {
    cfg
        .service(web::resource("/")
            .route(web::post().to(index))
            .route(web::get().guard(guard::Header("upgrade", "websocket")).to(index_ws))
            .route(web::get().to(index_playground))
        );
}

async fn index(schema: web::Data<AppSchema>, http_req: HttpRequest, req: Request) -> Response {
    let mut query = req.into_inner();

    let maybe_role = common_utils::get_role(http_req);
    if let Some(role) = maybe_role {
        query = query.data(role);
    }

    schema.execute(query).await.into()
}

async fn index_ws(schema: web::Data<AppSchema>, req: HttpRequest, payload: web::Payload) -> Result<HttpResponse> {
    WSSubscription::start(Schema::clone(&*schema), &req, payload)
}

async fn index_playground() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(GraphQLPlaygroundConfig::new("/").subscription_endpoint("/")))
}

pub fn create_schema_with_context(pool: PgPool) -> Schema<Query, Mutation, Subscription> {
    let arc_pool = Arc::new(pool);
    let cloned_pool = Arc::clone(&arc_pool);
    let details_batch_loader = Loader::new(DetailsBatchLoader {
        pool: cloned_pool
    }).with_max_batch_size(10);

    Schema::build(Query, Mutation, Subscription)
        // limits are commented out, because otherwise introspection query won't work
        // .limit_depth(3)
        // .limit_complexity(15)
        .data(arc_pool)
        .data(details_batch_loader)
        .data(kafka::create_producer())
        .finish()
}

pub fn run_migrations(pool: &PgPool) {
    let conn = pool.get().expect("Can't get DB connection");
    embedded_migrations::run(&conn).expect("Failed to run database migrations");
}

type Conn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn get_conn_from_ctx(ctx: &Context<'_>) -> Conn {
    ctx.data::<Arc<PgPool>>().expect("Can't get pool").get().expect("Can't get DB connection")
}
