use herdr_git_graph::{launch, run, PANE_LABEL};
use std::io::{self, Read};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--launch-decision") {
        let mut input = String::new();
        let _ = io::stdin().read_to_string(&mut input);
        println!("{}", launch::launch_decision(&input, PANE_LABEL));
        return;
    }
    if args.iter().any(|a| a == "--launch-decision-tab") {
        let mut input = String::new();
        let _ = io::stdin().read_to_string(&mut input);
        println!("{}", launch::launch_decision_tab(&input, PANE_LABEL));
        return;
    }
    if let Err(e) = run() {
        eprintln!("herdr-git-graph: {e}");
        std::process::exit(1);
    }
}
