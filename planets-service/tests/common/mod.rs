use std::collections::HashMap;
use std::env;

use dotenv::dotenv;
use testcontainers::{Container, Docker};
use testcontainers::clients::Cli;
use testcontainers::images::postgres::Postgres;

use planets_service::persistence::connection::{create_connection_pool, PgPool};
use planets_service::run_migrations;

pub fn setup(docker: &Cli) -> (Container<Cli, Postgres>, PgPool) {
    dotenv().ok();
    let pg_container = setup_database(docker);
    let pool = create_connection_pool();
    run_migrations(&pool);
    (pg_container, pool)
}

fn setup_database(docker: &Cli) -> Container<Cli, Postgres> {
    let pg_container = docker.run(get_pg_image());
    let pg_port = pg_container.get_host_port(5432).expect("Can't get port for connection to Postgres");
    env::set_var("DATABASE_URL", format!("postgres://postgres:password@localhost:{}/planets-db", pg_port));
    pg_container
}

fn get_pg_image() -> Postgres {
    let mut env_args = HashMap::new();
    env_args.insert("POSTGRES_DB".to_string(), "planets-db".to_string());
    env_args.insert("POSTGRES_PASSWORD".to_string(), "password".to_string());
    Postgres::with_env_vars(Postgres::default(), env_args)
}
