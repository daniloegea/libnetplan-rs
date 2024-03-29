use libnetplan::parser::Parser;
use libnetplan::state::State;

pub fn get(key: &String) {
    let parser = Parser::new();
    if let Err(error) = parser.load_yaml_hierarchy("/") {
        println!("error: {error:?}");
        return;
    }

    let state = State::new();
    _ = state.import_parser_state(parser);

    if key == "all" {
        let yaml = state.dump_yaml().unwrap();
        print!("{yaml}");
    } else {
        let yaml = state.dump_yaml_subtree(key).unwrap();
        print!("{yaml}");
    }
}
