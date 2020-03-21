mod repository;
mod core;

use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use crate::repository::user::User;
use std::collections::LinkedList;
use mysql::{PooledConn, Pool};
use std::borrow::Borrow;

//node
struct NodeData {
    pub user: Arc<User>,
    pub taken: bool,
    pub is_manager: bool,
    pub manager_count: u32,
}

impl NodeData {
    pub fn get_manage_award(&mut self) -> f64 {
        if self.taken {
            0.0
        } else {
            self.taken = true;
            self.user.get_user_profit()
        }
    }
}

struct ManageAwardAccumulator {
    all_managers: Arc<Vec<Arc<User>>>,
    pool: Arc<Pool>,
}

impl ManageAwardAccumulator {
    pub fn accumulator(&self) -> f64 {
        self.get_all_root().into_iter().for_each(|root| {
            let amount = self.create_child(root);
            println!("manage award is :{}", amount);
        });
        5.2
    }
    //produce some new LinkedList according to root.(reuse the all NodeData)
    fn create_child(&self, root: LinkedList<Arc<RwLock<NodeData>>>) -> f64 {
        let (manager_count, ivs) = {
            let mut data = root.back().expect("can't get tail.").read().unwrap();
            (data.manager_count, data.user.get_invitees())
        };//release read lock
        //if here is enough manager ,sum amount and return result,
        if manager_count == 4 {
            return root.iter()
                .map(|d| {
                    let mut nd = d.write().unwrap();
                    nd.get_manage_award()
                })
                .sum();
        }
        if let Some(invitees) = ivs {
            invitees
                .into_iter()
                .map(|inv| {
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
                    let node_data = NodeData {
                        user: Arc::new(inv),
                        taken: false,
                        is_manager,
                        manager_count,
                    };
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
                sum += d.get_manage_award();
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
                let node_data = NodeData {
                    user: mu.clone(),
                    taken: true,
                    is_manager: true,
                    manager_count: 0,
                };
                l.push_back(Arc::new(RwLock::new(node_data)));
                l
            })
            .collect()
    }
}

fn main() {
    let pool = repository::get_pool();
    let arc_pool = Arc::new(pool);
    let users = User::find_all_manager();
    let all_managers: Vec<Arc<User>> = users
        .into_iter()
        .map(|u| Arc::new(u))
        .collect();
    let all_managers = Arc::new(all_managers);
    let accumulator = ManageAwardAccumulator { all_managers, pool: arc_pool };
    accumulator.accumulator();
}