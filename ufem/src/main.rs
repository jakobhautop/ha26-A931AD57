use std::fs::File;
use std::fs::create_dir;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
// use std::os::unix::fs::chown; temporarily not used
use std::fs::OpenOptions;

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

#[derive(Clone)]
struct DirEntry {
    dnode: u32,
    dtype: DTypes,
    name: String,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
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
        let (inode_header, header_length) = Self::read_header(&block);
        match dtype {
            DTypes::U5fsDtypeDir => {
                println!("DMP: Processing dir: {path}");
                println!("DMP: Creating dir {path}");
                Self::create_dir_with_perm(&path, inode_header.uid, inode_header.gid).unwrap();
                println!("DMP: Completed creating dir {path}");
                let dir_entries = self.parse_dir_entries(inode);
                println!(
                    "DMP: Beginning processing {:?} entries found in dir node {inode}",
                    dir_entries.len()
                );
                for entry in dir_entries {
                    println!(
                        "DMP: Beginning recursive dump of {0} ({1}) in {path} at block index {2}",
                        entry.name, entry.dtype, entry.dnode
                    );
                    self.recursive_dump(entry.dnode, entry.dtype, &entry.name, &path);
                    println!(
                        "DMP: Completed recursive dump of {0} ({1}) in {path} at block index {2}",
                        entry.name, entry.dtype, entry.dnode
                    );
                }
                println!("DMP: Completed processing entries");
                println!("DMP: Completed dir: {path}");
            }
            DTypes::U5fsDtypeFile => {
                println!("DMP: Processing file: {path}");
                self.dump_file(block, &path, header_length, inode_header.size);
                println!("DMP: Completed processing file {path}");
            }
            _ => {
                println!("DMP: Unhandled type: {dtype}")
            }
        }
    }

    fn dump_file(&self, node_data: Vec<u8>, path: &str, header_length: u8, size: u32) {
        
        let blockcount = self.sb.unwrap().blockcount;       
        fn debug_scan_blocks(blocks: &Vec<u32>, note: &str, path: &str, blockcount: u32) {
            for b in blocks {

                if *b > blockcount {
                    println!(
                        ">> DEBUG: {0}: Found invalid block {1} (max is {2}). {3}",
                        path, b, blockcount, note
                    );
                }
            }
        }

        println!("FIL: Beginning dump of {size} bytes to file {path}");
        let blocksize = self.sb.unwrap().blocksize;
        let direct_blocks_from = header_length as usize;
        let direct_blocks_to = blocksize as usize - header_length as usize - 16; // 16 = 4 indirect1 + 4 indirect2 + 8 reserved
        let mut blocks_to_read: Vec<u32> = node_data[direct_blocks_from..direct_blocks_to]
            .chunks(4)
            .map(|chunk| u32::from_be_bytes(chunk.try_into().unwrap()))
            .filter(|idx| *idx != 0)
            .collect();

        debug_scan_blocks(&blocks_to_read, "Direct blocks", path, blockcount);

        let indirect1_from = direct_blocks_to;
        let indirect1_to = indirect1_from + 4;
        let indirect1_index =
            u32::from_be_bytes(node_data[indirect1_from..indirect1_to].try_into().unwrap());

        if indirect1_index != 0 {
            println!("FIL: Searching block indices from indirect1..");
            let indirect1_data = self.get_block(indirect1_index);
            let indirect1_blocks: Vec<u32> = indirect1_data
                .chunks(4)
                .map(|chunk| u32::from_be_bytes(chunk.try_into().unwrap()))
                .filter(|idx| *idx != 0)
                .collect();

            debug_scan_blocks(&indirect1_blocks, "Indirect1 blocks", path, blockcount);

            blocks_to_read.extend(indirect1_blocks.clone());
            println!(
                "FIL: Found and added {:?} blocks from indirect1..",
                indirect1_blocks.len()
            );

            let indirect2_from = indirect1_to;
            let indirect2_to = indirect2_from + 4;
            let indirect2_index =
                u32::from_be_bytes(node_data[indirect2_from..indirect2_to].try_into().unwrap());

            if indirect2_index != 0 {
                println!("FIL: Searching block indices from indirect2..");
                let indirect2_data = self.get_block(indirect2_index);
                let indirect2_data_blocks = &indirect2_data[4..];
                let indirect2_indirect1_blocks: Vec<u32> = indirect2_data_blocks
                    .chunks(4)
                    .map(|chunk| u32::from_be_bytes(chunk.try_into().unwrap()))
                    .filter(|idx| *idx != 0)
                    .collect();

                println!(
                    "FIL: Found {:?} indirect2 blocks",
                    indirect2_indirect1_blocks
                );

                let indirect2_blocks: Vec<u32> = indirect2_indirect1_blocks
                    .into_iter()
                    .flat_map(|idx| {
                        println!("FIL: Searching indices from indirect2 block {idx}");
                        let data: Vec<u8> = self.get_block(idx);
                        data.chunks(4)
                            .map(|chunk| u32::from_be_bytes(chunk.try_into().unwrap()))
                            .filter(|idx| *idx != 0)
                            .collect::<Vec<_>>()
                    })
                    .collect();

                debug_scan_blocks(&indirect2_blocks, "Indirect2 blocks", path, blockcount);

                blocks_to_read.extend(indirect2_blocks.clone());
                println!("FIL: Found and added {:?} blocks", indirect2_blocks.len());
            }
        }

        let mut blocks_to_load = size.div_ceil(blocksize);
        println!(
            "FIL: Estimated blocks needed: {blocks_to_load}. Blocks found: {:?}",
            blocks_to_read.len()
        );

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        println!("FIL: Prepared {path} for dumping data");

        for block_index in blocks_to_read {
            println!("FIL: Loading block {block_index} into {path}");
            let data = self.get_block(block_index);
            file.write_all(&data).unwrap();
            println!("FIL: Completed loading block {block_index} into {path}");
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
        let byte_index = self.sb.unwrap().blocksize as u64 * block_index as u64;
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
        println!("DEN: Parsing dir entries of block: {inode}");
        println!("DEN: Reading block: {inode}");
        let block = self.get_block(inode);
        println!("DEN: Completed reading block {inode}");
        println!("DEN: Reading header from block {inode}");
        let (header, header_length) = Self::read_header(&block);
        println!(
            "DEN: Completed reading header from block {inode}.. Size: {0}",
            header.size
        );
        println!("DEN: Reading bytes containing entries from block {inode}");
        let dir_entries_start = header_length as usize;
        let dir_entries_stop = self.sb.unwrap().blocksize as usize;
        let dir_entries_bytes = &block[dir_entries_start..dir_entries_stop];
        println!("DEN: Completed reading bytes containing entries from block {inode}");
        /*
        println!("<Node : {inode}>");
        println!("{:?}", dir_entries_bytes);
        println!("</Node : {inode}>");
        */
        println!("DEN: Beginning reading dir entries from bytes from block {inode}");

        let mut dir_entries = Vec::new();
        let mut idx = 0;
        let mut dnode_buf: [u8; 4] = [0, 0, 0, 0];
        let mut dtype_buf: [u8; 1] = [0];
        let mut name_buf = Vec::new();
        for b in dir_entries_bytes.into_iter() {
            if idx <= 3 {
                dnode_buf[idx] = *b;
                idx += 1;
            } else if idx == 4 {
                dtype_buf[0] = *b;
                idx += 1;
            } else if idx > 4 && *b != 0 {
                name_buf.push(*b);
            } else {
                let name = String::from_utf8(name_buf).unwrap();
                let dnode = u32::from_be_bytes(dnode_buf);
                let dtype = DTypes::from(u8::from_be_bytes(dtype_buf));
                println!(
                    "DEN <DEBUG> dnode: {0} dtype: {1} name: {2}",
                    dnode, dtype, name
                );
                dir_entries.push(DirEntry { dnode, dtype, name });
                idx = 0;
                dnode_buf = [0, 0, 0, 0];
                dtype_buf = [0];
                name_buf = Vec::new();
            }
        }

        /*
        let dir_entries: Vec<DirEntry> = dir_entries_bytes
            .chunks(header.size.try_into().unwrap())
            .map(|chunk| {
                let name_bytes = &chunk[5..];
                let name_null_byte = name_bytes
                    .iter()
                    .position(|&b| b == 0)
                    .unwrap_or(name_bytes.len());
                let name = String::from_utf8(name_bytes[..name_null_byte].to_vec()).unwrap();
                let dnode = u32::from_be_bytes(chunk[0..4].try_into().unwrap());
                let dtype = DTypes::from(u8::from_be_bytes(chunk[4..5].try_into().unwrap()));
                println!(
                    "DEN <DEBUG> dnode: {0} dtype: {1} name: {2}",
                    dnode, dtype, name
                );
                DirEntry { dnode, dtype, name }
            })
            .filter(|entry| entry.dtype != DTypes::unknown)
            .collect();


        dir_entries.clone().into_iter().map(|e| {
            println!(
                "DEN <DEBUG> VALID dnode: {0} dtype: {1} name: {2}",
                e.dnode, e.dtype, e.name
            );
        });
        println!(
            "DEN: Completed gathering {:?} valid entries from block {inode}",
            dir_entries.len()
        );
        */
        return dir_entries;
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
