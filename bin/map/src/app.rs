use crate::{
    info::NodeExtra,
    list::StatefulList,
    types::{NodeId, NodeInfo},
};
use std::collections::HashMap;

// the information here should be continually updating
// nodes are added from the result of rpc requests

#[derive(Clone)]
pub struct App {
    pub node_list: StatefulList,
}

impl App {
    pub fn new() -> App {
        let mut hashmap: HashMap<NodeId, NodeInfo> = HashMap::new();

        let node_id = Self::get_node_id();
        let node_info = Self::get_node_info();

        // TODO: fix this
        for id in node_id.iter() {
            for info in node_info.iter() {
                hashmap.insert(id.to_string(), info.to_string());
            }
        }

        let node_info = NodeExtra::new();
        App { node_list: StatefulList::new(hashmap, node_info) }
    }

    fn get_node_id() -> Vec<String> {
        let mut node_list = Vec::new();
        for num in 1..100 {
            let new_nodes = format!("\nNode {}\n", num);
            node_list.push(new_nodes);
        }
        node_list
    }

    fn get_node_info() -> Vec<String> {
        let mut node_info = Vec::new();
        for num in 1..100 {
            //let new_info = format!("\nConnections: {}\n", num);
            let new_info = "";
            node_info.push(new_info.to_string());
        }
        node_info
    }

    // every 5 seconds
    // TODO: implement this
    //fn update(&mut self) {
    //    let node = self.node.remove(0);
    //    self.nodes.push(node);
    //}
}
