use mysql::{chrono, PooledConn, Params, Value};
use mysql::prelude::Queryable;
use crate::repository;
use chrono::{NaiveDate, NaiveTime, Datelike};

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
    #[warn(dead_code)]
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
        if let Ok(stm) = con.prep("select user_id,mobile,real_name,inviter_id,superior_id from user where superior_id = ?") {
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
    pub fn get_user_profit(&self, date: NaiveDate) -> f64 {
        let time = NaiveTime::from_hms(0, 0, 0);
        let start_time = date.and_time(time);
        let end_time = if date.month() == 12 {
            NaiveDate::from_ymd(date.year() + 1, 1, 1).and_time(time)
        } else {
            NaiveDate::from_ymd(date.year(), date.month() + 1, 1).and_time(time)
        };
        let mut con: PooledConn = repository::POOL.get_conn().unwrap();
        let start_time = Value::Date(start_time.year() as u16, start_time.month() as u8, start_time.day() as u8, 0, 0, 0, 0);
        let end_time = Value::Date(end_time.year() as u16, end_time.month() as u8, end_time.day() as u8, 0, 0, 0, 0);
        let user_id = Value::Int(self.user_id as i64);
        if let Ok(Some(amount)) = con.exec_first("select ifnull(sum(sharing_amount),0) from \
        profit_sharing where user_id = ? and sharing_time between ? and ?",
                                                 vec![user_id, start_time, end_time]) {
            amount
        } else {
            0.0
        }
    }
}