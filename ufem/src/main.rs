struct INodeHeader {
    uid: u32,
    gid: u32,
    atime_sec: u32,
    atime_nsec: u32,
    mtime_sec: u32,
    mtime_nsec: u32,
    ctime_sec: u32,
    ctime_nsec: u32,
    perm: u16,
    links: u16,
    size: u32,
}

struct INode {
    header: INodeHeader,
}

struct Handle {
    path: String,
    block_size: u32,
    file_system_size: u32,
    block_count: u32,
    version: u32,
    root_node_idx: u32,
    inodes: Vec<INode>,
    bm: Vec<u32>,
}

impl Handle {
    fn init(path: &str) -> Self {
        Handle {
            path: path.to_string(),
            block_size: 0,
            file_system_size: 0,
            block_count: 0,
            version: 0,
            root_node_idx: 0,
            inodes: Vec::new(),
            bm: Vec::new(),
        }
    }

    fn read_super_block(&mut self) {}

    fn read_from_path(path: &str) -> Self {
        println!("RFP: Beginning read from path {path}");
        let mut handle = Self::init(path);
        println!("RFP: Read from path complete!");
        return handle;
    }

    fn dump(&self, path: &str) {
        println!("DMP: Beginning dump..");
        println!("DMP: Dump completed!");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = &args[1];
    let arg1 = &args[2];
    let arg2 = &args[3];

    match command.as_str() {
        "read-and-dump" => {
            println!("CLI: Beginning read-and-dump..");
            let in_path = arg1.clone();
            let out_path = arg2.clone();
            let handle = Handle::read_from_path(&in_path);
            handle.dump(&out_path);
            println!("CLI: Completed read-and-dump!");
            println!("CLI: Browse files at {out_path}");
        }
        _ => panic!("Unknown command: {command}"),
    }
}
