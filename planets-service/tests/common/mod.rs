use std::collections::HashMap;
use std::env;

use async_graphql::{EmptySubscription, Schema};
use dotenv::dotenv;
use testcontainers::{Container, Docker};
use testcontainers::clients::Cli;
use testcontainers::images::postgres::Postgres;

use planets_service::graphql::{Mutation, Query, Subscription};

pub fn setup(docker: &Cli) -> (Schema<Query, Mutation, EmptySubscription>, Container<Cli, Postgres>) {
    dotenv().ok();

    let pg_container = docker.run(get_pg_image());
    let pg_port = pg_container.get_host_port(5432).expect("Can't get port for connection to Postgres");
    // this env var is used by Diesel
    env::set_var("DATABASE_URL", format!("postgres://postgres:password@localhost:{}/planets-db", pg_port));

    (planets_service::setup(), pg_container)
}

fn get_pg_image() -> Postgres {
    let mut env_args = HashMap::new();
    env_args.insert("POSTGRES_DB".to_string(), "planets-db".to_string());
    env_args.insert("POSTGRES_PASSWORD".to_string(), "password".to_string());
    Postgres::with_env_vars(Postgres::default(), env_args)
}