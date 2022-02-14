use graph::*;

fn main() {
    let gr: Graph<String> = Graph::from_tgf_file("gr.txt").unwrap();

    for node_id in gr.node_ids().iter() {
        println!("Node id: {}", node_id);
        println!("Adjacent nodes ids: {:?}", gr.get_adjacent_ids(node_id));
        println!("Data: {}", gr.get_data(node_id));
    }
}
