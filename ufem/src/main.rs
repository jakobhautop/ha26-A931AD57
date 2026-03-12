use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

struct SuperBlock {
    magic: u32,
    version: u32,
    blocksize: u32,
    blockcount: u32,
    rootnode: u32, 
}

impl SuperBlock {
    
    fn from_be_bytes(buf: &[u8; 20]) -> Self {

        let magic_be: [u8; 4] = buf[0..4].try_into().unwrap();
        let magic_asci = std::str::from_utf8(&magic_be).unwrap();

        if magic_asci != "U5FS" {
            panic!("ERROR READING MAGIC: {magic_asci}");
        }

        println!("RSB: Found magic number: {magic_asci}");

        let version = u32::from_be_bytes(buf[4..8].try_into().unwrap());
        println!("RSB: Version: {0}", version);

        let blocksize = u32::from_be_bytes(buf[8..12].try_into().unwrap());
        println!("RSB: Block size: {0} Bytes", blocksize);
        
        let blockcount = u32::from_be_bytes(buf[12..16].try_into().unwrap());
        println!("RSB: Block count: {0}", blockcount);
        
        let rootnode = u32::from_be_bytes(buf[16..20].try_into().unwrap());
        println!("RSB: Root node block index: {0}", rootnode);

        return SuperBlock {
            magic: u32::from_be_bytes(magic_be).try_into().unwrap(),
            version,
            blocksize,
            blockcount,
            rootnode
        }
    } 

}

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

struct FileNode {
    header: INodeHeader, 
    blocks: Vec<u32>,
    indirect1: u32,
    indirect2: u32,
    reserved0: u32,
    reserved1: u32
}

struct DirNode {
    header: INodeHeader,
    entries: Vec<DirEntry>
}

struct DirEntry {
    dnode: u32,
    dtype: DTypes,
    name: String
}

enum DTypes {
    U5FS_DTYPE_DIR,
    U5FS_DTYPE_FILE,
    U5FS_DTYPE_CDEV,
    U5FS_DTYPE_BDEV,
    U5FS_DTYPE_LNK,
    U5FS_DTYPE_PIPE,
    U5FS_DTYPE_SOCK
}

struct FileIndirect1 {
    blocks: Vec<u32>
}

struct FileIndirect2 {
    reserved: u32,
    blocks: Vec<u32>
}


struct Handle {
    path: String,
    sb: Option<SuperBlock>
}

impl Handle {
    fn init(path: &str) -> Self {
        Handle {
            path: path.to_string(),
            sb: None
        }
    }

    fn read_super_block(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("RSB: Beginning reading super block");

        let mut file = File::open(self.path.clone())?;

        file.seek(SeekFrom::Start(0))?;

        let mut buf = [0u8; 20];
        file.read_exact(&mut buf)?;

        let sb = SuperBlock::from_be_bytes(&buf);
        /*
        let magic_be = &buf[0..4];
        let magic_asci = std::str::from_utf8(&magic_be).unwrap();
        println!("RSB: Found magic number: {magic_asci}");

        self.version = u32::from_be_bytes(buf[4..8].try_into().unwrap());
        println!("RSB: Version: {0}", self.version);

        self.block_size = u32::from_be_bytes(buf[8..12].try_into().unwrap());
        println!("RSB: Block size: {0} Bytes", self.block_size);
        self.block_count = u32::from_be_bytes(buf[12..16].try_into().unwrap());
        println!("RSB: Block count: {0}", self.block_count);
        self.root_node_idx = u32::from_be_bytes(buf[16..20].try_into().unwrap());
        println!("RSB: Root node block index: {0}", self.root_node_idx);
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
