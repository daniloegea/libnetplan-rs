


use libnetplan::libnetplan::{Parser, State};

pub fn get() {
    let parser = Parser::new();
    if let Err(error) = parser.load_yaml_hierarchy("/") {
        println!("error: {error:?}");
        return;
    }

    let state = State::new();
    state.import_parser_state(&parser);
    let yaml = state.dump_yaml().unwrap();
    println!("{yaml}");

    for netdef in state {
        println!("{}", netdef.name);
    }
}
