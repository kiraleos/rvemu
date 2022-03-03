mod emulator;
mod tests;
use emulator::cpu::Cpu;
fn main() {
    let mut args = std::env::args().skip(1);
    let mut paths: Vec<String> = Vec::new();
    match args.next() {
        Some(path) => paths.push(path),
        None => {
            for entry in std::fs::read_dir("./tests/").unwrap() {
                let path = entry.unwrap().path();
                if path.extension().unwrap_or_default() != "dump" {
                    paths.push(String::from(path.to_str().unwrap()));
                }
            }
        }
    }

    let mut cpu = Cpu::new();
    for path in paths {
        cpu.load(&path);
        let ret = cpu.run(false, true, false);
        // let ret = cpu.run_interactive();
        println!("{}\t\texit code: {}", path, ret);
        // println!("{}", cpu.all_instructions());
    }
}
