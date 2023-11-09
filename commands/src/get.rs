


use libnetplan::libnetplan::{Parser, State};

pub fn get() {
    let parser = Parser::new();
    parser.load_yaml_hierarchy("/");

    let state = State::new();
    state.import_parser_state(&parser);
    state.dump_yaml();

    for netdef in state {
        println!("{}", netdef.name);
    }
}
