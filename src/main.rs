mod repository;
mod core;

use std::sync::{Arc, RwLock};
use crate::repository::user::User;
use std::collections::LinkedList;
use chrono::NaiveDate;
use rayon::prelude::*;
use std::time::Duration;

//node
struct NodeData {
    user: Arc<User>,
    taken: bool,
    is_manager: bool,
    manager_count: u32,
}

impl NodeData {
    pub fn new(user: Arc<User>, taken: bool, is_manager: bool, manager_count: u32) -> Self {
        Self {
            user,
            taken,
            is_manager,
            manager_count,
        }
    }
    pub fn get_manage_award(&mut self, date: NaiveDate) -> f64 {
        //per list take once
        if self.taken {
            0.0
        } else {
            self.taken = true;
            self.user.get_user_profit(date)
        }
    }
}

struct ManageAwardAccumulator {
    all_managers: Arc<Vec<Arc<User>>>,
    date: Arc<NaiveDate>,
}

impl ManageAwardAccumulator {
    //create an accumulator
    pub fn new(all_managers: Vec<Arc<User>>, date: NaiveDate) -> Self {
        Self { all_managers: Arc::new(all_managers), date: Arc::new(date) }
    }

    pub fn accumulator(&self) -> f64 {
        self.get_all_root()
            .into_par_iter()
            .map(|root: LinkedList<Arc<RwLock<NodeData>>>| {
                let (user_name, user_id) = {
                    let user = &root.front().unwrap().read().unwrap().user;
                    (user.get_name(), user.get_user_id())
                };
                let manage_award = self.create_child(root);
                if let Some(user_name) = user_name {
                    println!("{}'s manage award amount is {}", user_name, manage_award);
                } else {
                    println!("{}'s manage award amount is {}", user_id, manage_award);
                }
                manage_award
            }).sum()
    }
    //produce some new LinkedList according to root.(reuse the all NodeData)
    fn create_child(&self, root: LinkedList<Arc<RwLock<NodeData>>>) -> f64 {
        let (manager_count, ivs) = {
            let data = root.back().expect("can't get tail.").read().unwrap();
            (data.manager_count, data.user.get_invitees())
        };//release read lock
        //if here is enough manager ,sum amount and return result,
        if manager_count == 4 {
            return root.iter()
                .map(|d| {
                    let mut nd = d.write().unwrap();
                    nd.get_manage_award(*self.date.clone())
                })
                .sum();
        }
        if let Some(invitees) = ivs {
            invitees
                .into_par_iter()
                .map(|inv: User| {
                    //copy all nodes of root to current list
                    let mut child_list = LinkedList::new();
                    root.iter().for_each(|n| {
                        child_list.push_back(n.clone());
                    });
                    //previous node's manager count
                    let pre_node_data_mc = { root.back().unwrap().read().unwrap().manager_count };
                    //check if current user is manager
                    let is_manager = self.all_managers
                        .iter()
                        .map(|u| u.get_user_id())
                        .any(|uid| uid == inv.get_user_id());
                    let manager_count = if is_manager {
                        //if current user is manager,the manager count plus 1
                        pre_node_data_mc + 1
                    } else {
                        //if not
                        pre_node_data_mc
                    };
                    let node_data = NodeData::new(Arc::new(inv), false, is_manager, manager_count);
                    //push current node data to back of list
                    child_list.push_back(Arc::new(RwLock::new(node_data)));
                    let child_list = child_list;
                    self.create_child(child_list)
                })
                .sum()
        } else {//has no invitees,return result according to manager count
            //keep 4 - (manager count) nodes behind last manager of current list
            let manager_count = {//the manager count of current list
                root.back().unwrap().read().unwrap().manager_count
            };
            let mut sum = 0.0;
            let mut mc = 0;
            let mut limit = 4 - manager_count;
            for ad in root.iter() {
                if limit == 0 {
                    break;
                }
                let mut d = ad.write().unwrap();
                if d.is_manager {
                    mc += 1;
                }
                if manager_count == mc {
                    limit -= 1;
                }
                sum += d.get_manage_award(*self.date.clone());
            }
            sum
        }
    }

    //build list according to manager
    fn get_all_root(&self) -> Vec<LinkedList<Arc<RwLock<NodeData>>>> {
        self.all_managers
            .iter()
            .map(|mu| {
                let mut l = LinkedList::new();
                let node_data = NodeData::new(mu.clone(), true, true, 0);
                l.push_back(Arc::new(RwLock::new(node_data)));
                l
            })
            .collect()
    }
}

fn main() {
    let start_time = std::time::SystemTime::now();
    let users = User::find_all_manager();
    let all_managers: Vec<Arc<User>> = users
        .into_iter()
        .map(|u| Arc::new(u))
        .collect();
    println!("number of manager is {}", all_managers.len());
    let date = NaiveDate::from_ymd(2020, 2, 1);
    rayon::ThreadPoolBuilder::new().num_threads(50).build_global();
    let accumulator = ManageAwardAccumulator::new(all_managers, date);
    let x = accumulator.accumulator();
    println!("total manage award amount is {}", x);
    let end_time = std::time::SystemTime::now();
    println!("耗时:{}", end_time.duration_since(start_time).ok().unwrap().as_secs());
}