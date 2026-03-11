use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

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

    fn read_super_block(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("RSB: Beginning reading super block");

        let mut file = File::open(self.path.clone())?;

        file.seek(SeekFrom::Start(0))?;

        let mut buf = [0u8; 16];
        file.read_exact(&mut buf)?;

        let magic = u32::from_be_bytes(buf[0..4].try_into().unwrap());

        println!("Found magic number: {magic}");

        /* block_size: u32::from_be_bytes(buf[4..8].try_into().unwrap()),
        #inode_count: u32::from_be_bytes(buf[8..12].try_into().unwrap()),
        #root_inode: u32::from_be_bytes(buf[12..16].try_into().unwrap()),
        */

        println!("RSB: Completed reading super block!");

        return Ok(());
    }

    fn read_from_path(path: &str) -> Self {
        println!("RFP: Beginning read from path {path}");

        if !path.ends_with(".u5fs") {
            panic!("Not a .u5fs file!");
        }

        let mut handle = Self::init(path);

        handle.read_super_block().unwrap();

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
