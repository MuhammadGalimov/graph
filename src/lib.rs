use std::{result::Result, fmt::Debug, fs::File, io::{self, BufRead}, vec};
use serde::{Serialize, de::DeserializeOwned};

#[derive(Debug, Clone)]
pub struct Graph<T> 
where 
    T: Serialize + DeserializeOwned + Clone 
{
    next_id: NodeId,
    nodes: Vec<Node<T>>
}

impl<T> Graph<T> 
where 
    T: Serialize + DeserializeOwned + Clone 
{
    pub fn new() -> Self {
        Graph { next_id: 0, nodes: vec![] }
    }

    pub fn from_tgf_file(path: &str) -> Result<Self, GraphError> {
        let mut nodes: Vec<Node<T>> = vec![];
        let mut node_ids: Vec<NodeId> = vec![];

        let file = File::open(path)?;
        let lines = io::BufReader::new(file).lines();

        let mut after = true;
        let re_node = regex::Regex::new(r"(\d+)\s(.+)").unwrap();

        for line in lines {
            if let Ok(item) = line {
                if item == "#" {
                    after = false;
                    continue;
                }

                if after {
                    let caps = re_node.captures(&item).unwrap();

                    let id = caps.get(1).unwrap().as_str().parse::<NodeId>().unwrap();
                    node_ids.push(id);
                    let data: T = serde_json::from_str(caps.get(2).unwrap().as_str()).unwrap();

                    for node in nodes.iter() {
                        if node.id() == id {
                            return Err(GraphError::TgfError);
                        }
                    }

                    nodes.push(Node::new(id, data));
                } else {
                    let mut node_index = 0;
                    for (i, caps) in item.split_whitespace().enumerate() {
                        let path = caps.parse::<NodeId>().unwrap();

                        if i == 0 {
                            node_index = path;
                            continue;
                        } else {
                            if !node_ids.contains(&path) {
                                return Err(GraphError::TgfError);
                            }

                            if let Some(node) = nodes.get_mut(node_index) {
                                node.add_path(path);
                            }
                        }
                    }
                }
            }
        }

        let mut next_id = 0;
        for node_id in node_ids.iter() {
            if *node_id > next_id {
                next_id = *node_id;
            }
        }

        Ok(Graph { next_id: next_id + 1, nodes })
    }

    pub fn add_node(&mut self, data: T) -> NodeId {
        self.nodes.push(Node::new(self.next_id, data));

        self.next_id += 1;
        self.next_id - 1
    }

    pub fn remove_node(&mut self, node_id: NodeId) -> Result<(), GraphError> {
        if self.contains(node_id) {
            for node in self.nodes.iter_mut() {
                if node.id() != node_id {
                    node.remove_path(node_id);
                }
            }

            if let Some(index) = self.nodes.iter().position(|x| x.id() == node_id) {
                self.nodes.remove(index);
            }

            Ok(())
        } else {
            Err(GraphError::IdNotExist)
        }
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId) -> Result<(), GraphError> {
        if self.contains(from) && self.contains(to) {
            if let Some(index) = self.nodes.iter_mut().position(|x| x.id() == from) {
                if let Some(node) = self.nodes.get_mut(index) {
                    node.add_path(to);
                }
            }

            Ok(())
        } else {
            Err(GraphError::IdNotExist)
        }
    }

    pub fn remove_edge(&mut self, from: NodeId, to: NodeId) -> Result<(), GraphError> {
        if self.contains(from) && self.contains(to) {
            if let Some(index) = self.nodes.iter().position(|x| x.id() == from) {
                if let Some(node) = self.nodes.get_mut(index) {
                    node.remove_path(to);
                }
            }

            Ok(())
        } else {
            Err(GraphError::IdNotExist)
        }
    }

    pub fn get_tgf(&self) -> String {
        let mut out = String::from("");

        for node in self.nodes.iter() {
            out.push_str(&node.get_tgf_node()[..]);
        }

        out.push_str("#\n");

        for node in self.nodes.iter() {
            out.push_str(&node.get_tgf_paths()[..]);
        }

        out
    }

    fn contains(&self, node_id: NodeId) -> bool {
        for node in self.nodes.iter() {
            if node.id() == node_id {
                return true;
            }
        }

        false
    }

    pub fn get_data(&self, id: &NodeId) -> T {
        self.nodes.get(id.clone()).unwrap().data()
    }

    pub fn get_adjacent_ids(&self, id: &NodeId) -> Vec<NodeId> {
        self.nodes.get(id.clone()).unwrap().paths()
    }

    pub fn node_ids(&self) -> Vec<NodeId> {
        let mut visited: Vec<NodeId> = vec![];

        self.dfs(0, &mut visited);

        visited
    }

    // pub fn node_idsf<'a>(&'a mut self) -> impl Iterator<Item = &'a NodeId> + 'a {
    //     self.visited.clear();
    //     let start: NodeId = 0;

    //     self.dfs(&start);

    //     self.visited.iter()
    // }

    fn dfs(&self, id: NodeId, visited: &mut Vec<NodeId>) {
        visited.push(id);
        for path in self.nodes.get(id).unwrap().paths() {
            if !visited.contains(&path) {
                self.dfs(path, visited);
            }
        }
    }
}

#[derive(Debug)]
pub enum GraphError {
    IdNotExist,
    IoError,
    TgfError,
}

impl From<io::Error> for GraphError {
    fn from(_error: io::Error) -> Self {
        GraphError::IoError
    }
}

#[derive(Debug, Clone)]
struct Node<T> 
where 
    T: Serialize + DeserializeOwned + Clone 
{
    id: NodeId,
    data: T,
    paths: Vec<NodeId>
}

impl<T> Node<T> 
where 
    T: Serialize + DeserializeOwned + Clone 
{
    fn new(id: NodeId, data: T) -> Self {
        Node { id, data, paths: vec![] }
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn paths(&self) -> Vec<NodeId> {
        self.paths.clone()
    }

    fn data(&self) -> T {
        self.data.clone()
    }

    fn add_path(&mut self, id: NodeId) {
        if !self.paths.contains(&id) {
            self.paths.push(id);
        }
    }

    fn remove_path(&mut self, path: NodeId) {
        if let Some(index) = self.paths.iter().position(|&x| x == path) {
            self.paths.remove(index);
        }
    }

    fn get_tgf_node(&self) -> String {
        format!("{} {}\n", self.id, serde_json::to_string(&self.data).unwrap())
    }

    fn get_tgf_paths(&self) -> String {
        if self.paths.len() > 0 {
            let mut out = format!("{}", self.id);

            for path in self.paths.iter() {
                out.push_str(&format!(" {}", path)[..]);
            }

            out.push_str("\n");

            out    
        } else {
            "".to_string()
        }
    }
}

type NodeId = usize;

#[cfg(test)]
mod tests {
    use crate::*;
    use serde::{Serialize, Deserialize};

    fn init() -> (Graph<String>, usize, usize) {
        let mut graph: Graph<String> = Graph::new();
        let cat_id = graph.add_node("cat".to_string());
        let car_id = graph.add_node("car".to_string());

        (graph, cat_id, car_id)
    }

    #[test]
    #[should_panic]
    fn adding_edge_error() {
        let (mut graph, cat_id, _) = init();
        graph.add_edge(34, cat_id).unwrap();
    }

    #[test]
    fn adding_edge() {
        let (mut graph, cat_id, car_id) = init();

        graph.add_edge(cat_id, car_id).expect("id error");
        graph.add_edge(cat_id, car_id).expect("id error");
        graph.add_edge(car_id, car_id).expect("id error");

        println!("{:?}", graph);
    }

    #[test]
    fn removing_edge() {
        let (mut graph, cat_id, car_id) = init();

        graph.add_edge(cat_id, car_id).expect("id error");
        println!("{:?}", graph);

        graph.add_edge(car_id, car_id).expect("id error");
        println!("{:?}", graph);

        graph.remove_edge(cat_id, car_id).expect("id error");
        graph.remove_edge(car_id, car_id).expect("id error");

        println!("{:?}", graph);
    }

    #[test]
    fn removing_node() {
        let (mut graph, cat_id, car_id) = init();

        graph.add_edge(cat_id, car_id).expect("id error");
        graph.add_edge(car_id, car_id).expect("id error");
        
        graph.remove_node(car_id).expect("id error");

        println!("{:?}", graph);
    }

    #[test]
    fn tgf() {
        let mut graph: Graph<String> = Graph::new();
        let cat_id = graph.add_node("cat".to_string());
        let car_id = graph.add_node("car".to_string());
        let cow_id = graph.add_node("cow".to_string());

        graph.add_edge(cat_id, car_id).expect("id error");
        graph.add_edge(cat_id, cow_id).expect("id error");
        graph.add_edge(cow_id, cat_id).expect("id error");

        assert_eq!(graph.get_tgf(), String::from("0 \"cat\"\n1 \"car\"\n2 \"cow\"\n#\n0 1 2\n2 0\n"));
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct S {
        number: i32,
        string: String
    }

    #[test]
    fn tgf_for_struct() {        
        let mut graph: Graph<S> = Graph::new();
        let cat_id = graph.add_node(S { number: 34, string: "cat".to_string() });
        let car_id = graph.add_node(S { number: 567, string: "car".to_string() });
        let cow_id = graph.add_node(S { number: -44, string: "cow".to_string() });

        graph.add_edge(cat_id, car_id).expect("id error");
        graph.add_edge(cat_id, cow_id).expect("id error");
        graph.add_edge(cow_id, cat_id).expect("id error");

        println!("{}", graph.get_tgf());

        //assert_eq!(graph.get_TGF(), String::from("0 \"cat\"\n1 \"car\"\n2 \"cow\"\n#\n0 1 2\n2 0\n"));
    }

    #[test]
    fn from_tgf_file() {
        let graph: Graph<String> = Graph::from_tgf_file("gr.txt").unwrap();
        println!("{:?}", graph);
    }

    #[test]
    fn get_node_ids() {
        let graph: Graph<String> = Graph::from_tgf_file("gr.txt").unwrap();
        
        for id in graph.node_ids().iter() {
            println!("{}", id);
        }
        
        println!("{:?}", graph);
    }
}
