use mysql::Pool;
use lazy_static::lazy_static;

pub mod user;

const URL: &str = "connection url of mysql";
lazy_static! {
    pub static ref POOL :Pool = get_pool();
}


pub fn get_pool() -> Pool {
    Pool::new(URL).expect("can't establish connection!")
}