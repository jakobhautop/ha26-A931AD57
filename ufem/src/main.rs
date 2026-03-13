use std::fs::File;
use std::fs::create_dir;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
// use std::os::unix::fs::chown; temporarily not used

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
    unknown = 0,
    U5fsDtypeDir = 1,
    U5fsDtypeFile = 2,
    U5fsDtypeCdev = 3,
    U5fsDtypeBdev = 4,
    U5fsDtypeLnk = 5,
    U5fsDtypePipe = 6,
    U5fsDtypeSock = 7,
}

impl std::fmt::Display for DTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

impl From<u8> for DTypes {
    fn from(v: u8) -> Self {
        match v {
            1 => DTypes::U5fsDtypeDir,
            2 => DTypes::U5fsDtypeFile,
            3 => DTypes::U5fsDtypeCdev,
            4 => DTypes::U5fsDtypeBdev,
            5 => DTypes::U5fsDtypeLnk,
            6 => DTypes::U5fsDtypePipe,
            7 => DTypes::U5fsDtypeSock,
            _ => {
                println!("  WARN: invalid dtype {0}", v);
                DTypes::unknown
            }
        }
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
        create_dir(path_to_dump);
        self.recursive_dump(sb.rootnode, DTypes::U5fsDtypeDir, "root", path_to_dump);

        println!("RFP: Read from path complete!");
    }

    fn recursive_dump(&self, inode: u32, dtype: DTypes, name: &str, dir: &str) {
        println!("DMP: Beginning recursive dump: {inode} {dtype} {name} {dir}");
        let path = dir.to_owned() + "/" + name;
        println!("DMP: Path: {path}");
        let block = self.get_block(inode);
        let (inode_header, _) = Self::read_header(&block);
        match dtype {
            DTypes::U5fsDtypeDir => {
                println!("DMP: Processing dir: {path}");
                Self::create_dir_with_perm(&path, inode_header.uid, inode_header.gid).unwrap();
                let dir_entries = self.parse_dir_entries(inode);
                for entry in dir_entries {
                    self.recursive_dump(entry.dnode, entry.dtype, &entry.name, &path);
                }
                println!("DMP: Completed dir: {path}");
            }
            _ => {
                println!("DMP: Unhandled type: {dtype}")
            }
        }
    }

    fn create_dir_with_perm(path: &str, uid: u32, gid: u32) -> std::io::Result<()> {
        println!("DIR: Creating {path} with uid {uid} and gid {gid}");
        create_dir(path)?;
        println!("DIR: Completed creating {path}");
        println!("DIR: Chown {uid} {gid} [TEMPORARILY IGNORED]");
        //chown(path, Some(uid), Some(gid))?;
        println!("DIR: Completed creation of {path} with permissions");
        Ok(())
    }

    fn get_block(&self, block_index: u32) -> Vec<u8> {
        println!("BLK: Beginning read of block {block_index}");
        let mut file = File::open(self.path.clone()).unwrap();
        let byte_index = self.sb.unwrap().blocksize * block_index;
        println!("BLK: Reading from byte {byte_index}");
        file.seek(SeekFrom::Start(byte_index.into())).unwrap();
        let mut buf = vec![0u8; self.sb.unwrap().blocksize.try_into().unwrap()];
        file.read_exact(&mut buf).unwrap();
        println!("BLK: Completed reading block {block_index}");
        return buf;
    }

    fn read_header(block: &Vec<u8>) -> (INodeHeader, u8) {
        println!("HDR: Beginning reading header..");

        let bytes_reserved = 4;
        let bytes_uid = 4;
        let bytes_gid = 4;
        let bytes_atime_sec = 4;
        let bytes_atime_nsec = 4;
        let bytes_mtime_sec = 4;
        let bytes_mtime_nsec = 4;
        let bytes_ctime_sec = 4;
        let bytes_ctime_nsec = 4;
        let bytes_perm = 2;
        let bytes_links = 2;
        let bytes_size = 4;

        let mut from = bytes_reserved;
        let mut to = from + bytes_uid;
        let uid = u32::from_be_bytes(block[from..to].try_into().unwrap());
        println!("HDR: UID: {uid}. From: {from}. To: {to}");
        from = to;

        to = from + bytes_gid;
        let gid = u32::from_be_bytes(block[from..to].try_into().unwrap());
        println!("HDR: GID: {gid}. From: {from}. To: {to}");
        from = to;

        to = from + bytes_atime_sec;
        let atime_sec = u32::from_be_bytes(block[from..to].try_into().unwrap());
        println!("HDR: Atime_sec: {atime_sec}. From: {from}. To: {to}");
        from = to;

        to = from + bytes_atime_nsec;
        let atime_nsec = u32::from_be_bytes(block[from..to].try_into().unwrap());
        println!("HDR: Atime_nsec: {atime_nsec}. From: {from}. To: {to}");
        from = to;

        to = from + bytes_mtime_sec;
        let mtime_sec = u32::from_be_bytes(block[from..to].try_into().unwrap());
        println!("HDR: Mtime_sec: {mtime_sec}. From: {from}. To: {to}");
        from = to;

        to = from + bytes_mtime_nsec;
        let mtime_nsec = u32::from_be_bytes(block[from..to].try_into().unwrap());
        println!("HDR: Mtime_nsec: {mtime_nsec}. From: {from}. To: {to}");
        from = to;

        to = from + bytes_ctime_sec;
        let ctime_sec = u32::from_be_bytes(block[from..to].try_into().unwrap());
        println!("HDR: Mtime_sec: {ctime_sec}. From: {from}. To: {to}");
        from = to;

        to = from + bytes_ctime_nsec;
        let ctime_nsec = u32::from_be_bytes(block[from..to].try_into().unwrap());
        println!("HDR: Mtime_nsec: {ctime_nsec}. From: {from}. To: {to}");
        from = to;

        to = from + bytes_perm;
        let perm = u16::from_be_bytes(block[from..to].try_into().unwrap());
        println!("HDR: PERM: {perm}. From: {from}. To: {to}");
        from = to;

        to = from + bytes_links;
        let links = u16::from_be_bytes(block[from..to].try_into().unwrap());
        println!("HDR: Links: {links}. From: {from}. To: {to}");
        from = to;

        to = from + bytes_size;
        let size = u32::from_be_bytes(block[from..to].try_into().unwrap());
        println!("HDR: Size: {size}. From: {from}. To: {to}");
        from = to;

        println!("HDR: Completed reading header..");

        let header_length = bytes_reserved
            + bytes_uid
            + bytes_gid
            + bytes_atime_sec
            + bytes_atime_nsec
            + bytes_mtime_sec
            + bytes_mtime_nsec
            + bytes_ctime_sec
            + bytes_ctime_nsec
            + bytes_perm
            + bytes_links
            + bytes_size;

        return (
            INodeHeader {
                uid,
                gid,
                atime_sec,
                atime_nsec,
                mtime_sec,
                mtime_nsec,
                ctime_sec,
                ctime_nsec,
                perm,
                links,
                size,
            },
            header_length.try_into().unwrap(),
        );
    }

    fn parse_dir_entries(&self, inode: u32) -> Vec<DirEntry> {
        println!("DEN: Parsing dir entries: {inode}");
        println!("DEN: Reading block..");
        let block = self.get_block(inode);
        println!("DEN: Completed reading block..");
        println!("DEN: Reading header..");
        let (header, header_length) = Self::read_header(&block);
        println!("DEN: Completed reading header.. Size: {0}", header.size);
        println!("DEN: Reading entries bytes");
        let dir_entries_start = header_length as usize;
        let dir_entries_stop = self.sb.unwrap().blocksize as usize;
        let dir_entries_bytes = &block[dir_entries_start..dir_entries_stop];
        println!("DEN: Completed reading entries bytes");
        println!("DEN: Reading dir entries from bytes");
        let dir_entries: Vec<DirEntry> = dir_entries_bytes
            .chunks(header.size.try_into().unwrap())
            .map(|chunk| {
                let name_bytes = &chunk[5..];
                let name_null_byte = name_bytes
                    .iter()
                    .position(|&b| b == 0)
                    .unwrap_or(name_bytes.len());
                let name = String::from_utf8(name_bytes[..name_null_byte].to_vec()).unwrap();
                println!("DEN: entry {name}");
                let dnode = u32::from_be_bytes(chunk[0..4].try_into().unwrap());
                let dtype = DTypes::from(u8::from_be_bytes(chunk[4..5].try_into().unwrap()));
                DirEntry { dnode, dtype, name }
            })
            .collect();
        println!("DEN: Completed reading dir entries from bytes");
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
