mod cpu;
mod tests;
use cpu::Cpu;

fn main() {
    let mut args = std::env::args().skip(1);
    let mut paths: Vec<String> = Vec::new();
    match args.next() {
        Some(path) => paths.push(path),
        None => {
            for entry in std::fs::read_dir("./tests/").unwrap() {
                let path = entry.unwrap().path();
                paths.push(String::from(path.to_str().unwrap()));
            }
        }
    }

    let mut print_flag = false;
    let mut debug_flag = false;
    let flag = args.next().unwrap_or_default();
    match flag.as_str() {
        "-p" => print_flag = true,
        "-d" => debug_flag = true,
        "-pd" | "-dp" => {
            debug_flag = true;
            print_flag = true;
        }
        _ => {}
    }

    let mut cpu = Cpu::new();
    for path in paths {
        cpu.load(&path);
        let ret = cpu.run(debug_flag, print_flag);
        println!("{}\n\texit code: {}", path, ret);
    }
}
