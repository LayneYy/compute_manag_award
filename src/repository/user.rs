use mysql::{chrono, Text, PooledConn, Params, Value};
use mysql::prelude::Queryable;
use std::borrow::Borrow;
use std::ops::Add;
use std::time::Duration;
use crate::repository;

//user entity
#[derive(Debug, PartialEq, Eq)]
pub struct User {
    user_id: usize,
    mobile: String,
    real_name: Option<String>,
    inviter_id: Option<usize>,
    superior_id: Option<usize>,
}

impl User {
    //get user name
    pub fn get_name(&self) -> Option<&String> {
        Option::from(&self.real_name)
    }
    //get user id
    pub fn get_user_id(&self) -> usize {
        self.user_id
    }
    //get all invitees
    pub fn get_invitees(&self) -> Option<Vec<User>> {
        let mut con: PooledConn = repository::POOL.get_conn().unwrap();
        if let Ok(mut stm) = con.prep("select user_id,mobile,real_name,inviter_id,superior_id from user where superior_id = ?") {
            let val = vec![Value::from(self.user_id)];
            let op = con
                .exec_map(stm, Params::Positional(val), |(user_id, mobile, real_name, inviter_id, superior_id)| User { user_id, mobile, real_name, inviter_id, superior_id })
                .ok();
            match op {
                Some(v) if v.len() > 0 => {
                    Some(v)
                }
                _ => None
            }
        } else {
            None
        }
    }
    //all managers
    pub fn find_all_manager() -> Vec<User> {
        let mut con: PooledConn = repository::POOL.get_conn().unwrap();
        con.query_map("select u.user_id,u.mobile,u.real_name,u.inviter_id,u.superior_id
    from user u
    where u.agent_level > 0
    and (select count(*) from user u2 where u2.inviter_id = u.user_id and u2.agent_level > 1) >= 3", |(user_id, mobile, real_name, inviter_id, superior_id)| {
            User { user_id, mobile, real_name, inviter_id, superior_id }
        })
            .expect("query error")
    }

    //get profit of user
    pub fn get_user_profit(&self) -> f64 {
        println!("load profit form database.");
        1.0
    }
}