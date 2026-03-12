use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

#[derive(Copy, Clone)]
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
            panic!("FBE: ERROR READING MAGIC: {magic_asci}");
        }
        println!("FBE: Found magic number: {magic_asci}");
        let version = u32::from_be_bytes(buf[4..8].try_into().unwrap());
        println!("FBE: Version: {0}", version);
        let blocksize = u32::from_be_bytes(buf[8..12].try_into().unwrap());
        println!("FBE: Block size: {0} Bytes", blocksize);
        let blockcount = u32::from_be_bytes(buf[12..16].try_into().unwrap());
        println!("FBE: Block count: {0}", blockcount);
        let rootnode = u32::from_be_bytes(buf[16..20].try_into().unwrap());
        println!("FBE: Root node block index: {0}", rootnode);
        return SuperBlock {
            magic: u32::from_be_bytes(magic_be).try_into().unwrap(),
            version,
            blocksize,
            blockcount,
            rootnode,
        };
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
    reserved1: u32,
}

struct DirNode {
    header: INodeHeader,
    entries: Vec<DirEntry>,
}

struct DirEntry {
    dnode: u32,
    dtype: DTypes,
    name: String,
}

#[repr(u8)]
#[derive(Copy, Clone)]
enum DTypes {
    U5FS_DTYPE_DIR = 1,
    U5FS_DTYPE_FILE = 2,
    U5FS_DTYPE_CDEV = 3,
    U5FS_DTYPE_BDEV = 4,
    U5FS_DTYPE_LNK = 5,
    U5FS_DTYPE_PIPE = 6,
    U5FS_DTYPE_SOCK = 7,
}

impl std::fmt::Display for DTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

struct FileIndirect1 {
    blocks: Vec<u32>,
}

struct FileIndirect2 {
    reserved: u32,
    blocks: Vec<u32>,
}

struct Handle {
    path: String,
    sb: Option<SuperBlock>,
}

impl Handle {
    fn init(path: &str) -> Self {
        println!("INI: Beginning Init..");
        if !path.ends_with(".u5fs") {
            panic!("Not a .u5fs file!");
        }
        let sb = Self::read_super_block(path).unwrap();
        println!("INI: Completed Init..");
        Handle {
            path: path.to_string(),
            sb: Some(sb),
        }
    }

    fn read_super_block(path: &str) -> Result<SuperBlock, Box<dyn std::error::Error>> {
        println!("RSB: Beginning reading super block");
        let mut file = File::open(path)?;
        file.seek(SeekFrom::Start(0))?;
        let mut buf = [0u8; 20];
        file.read_exact(&mut buf)?;
        let sb = SuperBlock::from_be_bytes(&buf);
        println!("RSB: Completed reading super block!");
        return Ok(sb);
    }

    fn fsdump(&self, path_to_dump: &str) {
        println!("RFP: Beginning read_image_and_dump to {path_to_dump}");
        let sb = self.sb.unwrap().clone();
        Self::recursive_dump(sb.rootnode, DTypes::U5FS_DTYPE_DIR, "root", path_to_dump);

        println!("RFP: Read from path complete!");
    }

    fn recursive_dump(inode: u32, dtype: DTypes, name: &str, path: &str) {
        println!("DMP: Beginning recursive dump: {inode} {dtype} {name} {path}");
        let full_path = path.to_owned() + "/" + name;
        println!("DMP: Full path: {full_path}");
        let inode_header: INodeHeader = Self::read_header(inode);
        match dtype {
            DTypes::U5FS_DTYPE_DIR => {
                println!("DMP: Processing dir: {full_path}");
                Self::create_dir(&full_path, inode_header.uid, inode_header.gid);
                let dir_entries = Self::parse_dir_entries(inode);
                for entry in dir_entries {
                    Self::recursive_dump(entry.dnode, entry.dtype, &entry.name, &full_path);
                }
                println!("DMP: Completed dir: {full_path}");
            }
            _ => {
                println!("DMP: Unhandled type: {dtype}")
            }
        }
    }

    fn create_dir(full_path: &str, uid: u32, gid: u32) {
        println!("DIR: Should create next.. {full_path}");
    }

    fn read_header(inode: u32) -> INodeHeader {
        println!("HDR: Should read header: {inode}");
        return INodeHeader {
            gid: 0,
            uid: 0,
            atime_sec: 0,
            atime_nsec: 0,
            mtime_sec: 0,
            mtime_nsec: 0,
            ctime_sec: 0,
            ctime_nsec: 0,
            perm: 0,
            links: 0,
            size: 0,
        };
    }

    fn parse_dir_entries(inode: u32) -> Vec<DirEntry> {
        println!("DEN: Parsing dir entries: {inode}");
        return Vec::new();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = &args[1];
    let arg1 = &args[2];
    let arg2 = &args[3];

    match command.as_str() {
        "fsdump" => {
            println!("CLI: Beginning fsdump..");
            let in_path = arg1.clone();
            let out_path = arg2.clone();
            let handle = Handle::init(&in_path);
            let handle = handle.fsdump(&out_path);
            println!("CLI: Completed fsdump!");
            println!("CLI: Browse files at {out_path}");
        }
        _ => panic!("Unknown command: {command}"),
    }
}
