use crafting_interpreters::runtime::Runtime;


fn main() {
    env_logger::init();
    let mut runtime = Runtime::default();
    let arg_length = std::env::args().count();
    if arg_length > 2 {
        println!("Usage: jlox [script]");
        std::process::exit(64);
    } else if arg_length == 2 {
        runtime.run_file(std::env::args().nth(1).unwrap());
    } else {
        runtime.run_prompt();
    }
}
