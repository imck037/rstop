use sysinfo::System;
fn main() {
    let sys = System::new_all();
    println!("{:?}", sys);
    for (pid, process) in sys.processes() {
        println!("Pid: {}  {:?}", pid.as_u32(), process.name());
    }
}
