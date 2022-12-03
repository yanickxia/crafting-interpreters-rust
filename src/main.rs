use clap::{Parser, ValueEnum};

use crafting_interpreters::runtime::{Runtime, VMRuntime};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum RuntimeType {
    VirtualMachine,
    Interpreter,
}


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    disassemble: bool,

    #[arg(short, long, value_enum)]
    model: RuntimeType,

    #[arg(short, long)]
    file: String,
}

fn main() {
    env_logger::init();
    let args = Args::parse() as Args;

    match args.model {
        RuntimeType::VirtualMachine => {
            let mut vm_runtime = VMRuntime::default();
            vm_runtime.disassemble = args.disassemble;
            vm_runtime.run_file(args.file)
        }
        RuntimeType::Interpreter => {
            Runtime::default().run_file(args.file)
        }
    }
}
