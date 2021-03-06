#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
//#[macro_use] extern crate rocket_contrib;

use bookwerx_core_rust::constants as C;
use bookwerx_core_rust::db as D;
use bookwerx_core_rust::model as M;
use bookwerx_core_rust::routes as R;
use bookwerx_core_rust::schema as S;
use bookwerx_core_rust::routz as Z;

use clap::clap_app;
use std::env;
use rocket::config::{Config, Environment};

use juniper::{EmptyMutation, RootNode};
use rocket::response::content;
use rocket::State;
use bookwerx_core_rust::db::MyRocketSQLConn;

use std::sync::{Arc, Mutex};

fn main() {

    // 1. Configure the CLI
    let cli_matcher = clap_app!(bookwerx_core_rust =>
        (version: "2.3.1") // VERSION
        (author: "Thomas Radloff. <bostontrader@gmail.com>")
        (about: "A blind man in a dark room looking for a black cat that's not there.")
        (@arg bind_ip: -b --bind_ip +takes_value "Specifies an IP address for the http server to bind to. Ex: 0.0.0.0")
        (@arg bind_port: -p --bind_port +takes_value "Specifies a port for the http server to bind to.")
        (@arg conn: -c --conn +takes_value "Specifies a connection string to connect to the db. Ex: mysql://root:mysecretpassword@127.0.0.1:3306")
        (@arg dbname: -d --dbname +takes_value "The database name to use.")
        (@arg mode: -m --mode +takes_value "Operating mode. Ex: test, development, or production")
    ).get_matches();

    // 2. Obtain the configuration arguments passed via command line or environment, if any.

    // 2.1 bind_ip value.  Must have.
    let bind_ip_value;
    match cli_matcher.value_of(C::BIND_IP_KEY_CLI) {
        Some(_result) => {
            println!("Rocket will bind to IP address [{}], as set from the command line.", _result);
            bind_ip_value = _result.to_string();
        }
        None =>
            match env::var(C::BIND_IP_KEY_ENV) {
                Ok(_x) => {
                    println!("Rocket will bind to IP address [{}], as set from the environment.", _x);
                    bind_ip_value = _x;
                }

                Err(_) => {
                    println!("Fatal error: No binding IP address is available.");
                    ::std::process::exit(1);
                }
            }
    }

    // 2.2 bind_port value.  Must have.
    let bind_port_value;
    match cli_matcher.value_of(C::BIND_PORT_KEY_CLI) {
        Some(_result) => {
            println!("Rocket will bind to port [{}], as set from the command line.", _result);
            bind_port_value = _result.to_string();
        }
        None =>
            match env::var(C::BIND_PORT_KEY_ENV) {
                Ok(_x) => {
                    println!("Rocket will bind to port [{}], as set from the environment.", _x);
                    bind_port_value = _x;
                }

                Err(_) => {
                    println!("Fatal error: No binding port is available.");
                    ::std::process::exit(1);
                }
            }
    }
    
    
    // 2.3 conn_value.  Must have.
    let conn_value;
    match cli_matcher.value_of(C::CONN_KEY_CLI) {
        Some(_result) => {
            println!("Accessing the db via connection string [{}], as set from the command line.", _result);
            conn_value = _result.to_string();
        }
        None =>
            match env::var(C::CONN_KEY_ENV) {
                Ok(_result) => {
                    println!("Accessing the db via connection string [{}], as set from the environment.", _result);
                    conn_value = _result;
                }

                Err(_) => {
                    println!("Fatal error: No db connection string is available.");
                    ::std::process::exit(1);
                }
            }
    }

    // 2.4 dbname_value.  Must have.
    let dbname_value;
    match cli_matcher.value_of(C::DBNAME_KEY_CLI) {
        Some(_result) => {
            println!("Using db [{}], as set from the command line.", _result);
            dbname_value = _result.to_string();
        }
        None =>
            match env::var(C::DBNAME_KEY_ENV) {
                Ok(_result) => {
                    println!("Using db [{}], as set from the environment.", _result);
                    dbname_value = _result;
                }

                Err(_) => {
                    println!("Fatal error: No db name is available.");
                    ::std::process::exit(1);
                }
            }
    }

    // 2.5 mode_value.  Must have.
    //let mode_value;
    match cli_matcher.value_of(C::MODE_KEY_CLI) {
        Some(_result) => {
            println!("Operating in {} mode, as set from the command line.", _result);
            //mode_value = _result.to_string();
        }
        None =>
            match env::var(C::MODE_KEY_ENV) {
                Ok(_result) => {
                    println!("Operating in {} mode, as set from the environment.", _result);
                    //mode_value = _result;
                }

                Err(_) => {
                    println!("Fatal error: No operating mode is available.");
                    ::std::process::exit(1);
                }
            }
    }


    // 3. Now crank-up rocket!

    // 3.1 First build a configuration hash-map for Rocket
    let mut full_conn = conn_value.to_string();
    full_conn.push('/');
    full_conn.push_str(&dbname_value.to_string());

    let mut hm_inner = std::collections::HashMap::new();
    hm_inner.insert("url".to_string(), full_conn);

    let mut hm_outer = std::collections::HashMap::new();
    hm_outer.insert("mysqldb".to_string(), hm_inner);

    // 3.2 Then build the Rocket configuration object
    let config = Config::build(Environment::Staging)
        .address(bind_ip_value)
        .extra("databases",hm_outer)
        .port(bind_port_value.parse::<u16>().unwrap())
        .finalize().unwrap();

    // 3.3 Configure CORS
    let cors = rocket_cors::CorsOptions {
        send_wildcard: true,
        ..Default::default()
    }
        .to_cors().unwrap();

    println!("{:?}", cors);

    let jdb = M::JunDatabase::new();

    // 3.4 Finally, launch it
    rocket::custom(config)
        .attach(D::MyRocketSQLConn::fairing())
        .attach(cors)
        .manage(jdb)
        .manage(Schema::new(S::Query, EmptyMutation::<M::JunDatabase>::new()))
        .mount("/", routes![
            R::index,

            Z::account::delete_account,
            Z::account::get_account,
            Z::get_account_dist_sum::get_account_dist_sum,
            Z::account::get_accounts,
            Z::account::post_account,
            Z::account::put_account,

            Z::acctcat::delete_acctcat,
            Z::acctcat::get_acctcat,
            Z::acctcat::get_acctcats_for_category,
            Z::acctcat::post_acctcat,
            Z::acctcat::put_acctcat,

            R::post_apikey,

            Z::category::delete_category,
            Z::category::get_category,
            Z::category::get_category_bysym,
            Z::category::get_categories,
            Z::get_category_dist_sums::get_category_dist_sums,
            Z::category::post_category,
            Z::category::put_category,

            Z::currency::delete_currency,
            Z::currency::get_currency,
            Z::currency::get_currencies,
            Z::currency::post_currency,
            Z::currency::put_currency,

            Z::distribution::delete_distribution,
            Z::distribution::get_distribution,
            Z::distribution::get_distributions,
            Z::distribution::get_distributions_for_account,
            Z::distribution::get_distributions_for_tx,
            Z::distribution::post_distribution,
            Z::distribution::put_distribution,

            Z::get_linter_accounts::get_linter_accounts,
            Z::get_linter_categories::get_linter_categories,
            Z::get_linter_currencies::get_linter_currencies,

            Z::transaction::delete_transaction,
            Z::transaction::get_transaction,
            Z::transaction::get_transactions,
            Z::transaction::post_transaction,
            Z::transaction::put_transaction,

            graphiql, get_graphql_handler, post_graphql_handler

        ])
        .launch();
}

type Schema = RootNode<'static, S::Query, EmptyMutation<M::JunDatabase>>;


#[rocket::get("/graphql")]
fn graphiql() -> content::Html<String> {
    juniper_rocket::graphiql_source("/graphql")
}

#[rocket::get("/graphql?<request>")]
fn get_graphql_handler(
    context: State<M::JunDatabase>,
    request: juniper_rocket::GraphQLRequest,
    schema: State<Schema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(&schema, &context)
}


#[rocket::post("/graphql", data = "<request>")]
fn post_graphql_handler(
    context: State<M::JunDatabase>,
    request: juniper_rocket::GraphQLRequest,
    schema: State<Schema>,
    mut conn: MyRocketSQLConn
) -> juniper_rocket::GraphQLResponse {


    {
        let m = &*context.conn;
        let mut m1 = m.lock().unwrap();
        *m1 = Some(conn);
    }

    request.execute(&schema, &context)

}