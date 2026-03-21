
# Things to document 

## 1) Bug i dir 
Til at starte med lavede jeg en bug i måden jeg læste dirs.
Havde forstået specs som at header.size på var en fixed size for hver entry struct.

Jeg opgade denne fejl ved at 
- observere at jeg kun havde BURNAFTERREADING.md i mit dump
- `strings operation.u5fs` viste der var meget mere + filstørelsn af operation.u5fs passede ikke med en enkelt .md file 

Fix: 
I stedet for at læse dir entries i chunks af header.size løber vi bare blokken igennem
mens vi løbende laver et nyt dir entries hver gang name er en null byte. 

## 2) Bugs i fejl reading
Dump fejler når den skal læse filen dump/root/update2d/images/imgB1.u2d:

```
BLK: Beginning read of block 400794285
BLK: Reading from byte 975,884,288

thread 'main' (71586) panicked at src/main.rs:312:35:
called `Result::unwrap()` on an `Err` value: Error { kind: UnexpectedEof, message: "failed to fill whole buffer" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Hvis vi kører en stat kan vi forsøger at læse fra bytes (975,884,288) som overgår fil størelsen (524,288,000)

```
user@cognitio:~$ stat Desktop/operation.u5fs
  File: Desktop/operation.u5fs
  Size: 524,288,000 	Blocks: 1024000    IO Block: 4096   regular file
Device: 8,2	Inode: 311312      Links: 1
Access: (0644/-rw-r--r--)  Uid: (    0/    root)   Gid: (    0/    root)
Access: 2026-03-15 09:35:53.220886360 +0000
Modify: 2026-03-10 00:26:54.000000000 +0000
Change: 2026-03-10 00:28:20.916000000 +0000
 Birth: 2026-03-10 00:28:18.204000000 +0000
```

Fix 1) brug u64 til at udregne block index istedet for u32 * u32 fordi vi ku få overflow. 
Dette gav dog:

```
FIL: Searching indices from indirect2 block 400794285
BLK: Beginning read of block 400794285
BLK: Reading from byte 1641653391360

thread 'main' (71689) panicked at src/main.rs:312:35:
called `Result::unwrap()` on an `Err` value: Error { kind: UnexpectedEof, message: "failed to fill whole buffer" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Hvis man dividerer 1641653391360/4096 får man faktisk den rigtige block. Så aritmetikken er ok. 
Så må det være block index der er forkert...

Fix 2) Jeg lagde mærke til at u5fs_file_indirect2 i speccen har 4 reserverede bytes.
Så introducerede et offset på 4 bytes inden jeg læser blokke fra indirect2..
Dette løste ikke problemet heller. 

Fix 3) for at indsnævre problemet yderligere lavede jeg en debug function
som jeg tjekker de blok-indexes jeg finder med:

```
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
```

Dette viser overraskende at det er når jeg læser fra indirect1 fejlen er:

>> DEBUG: dump/root/update2d/images/imgB1.u2d: 
Found invalid block 3009847678 (max is 128000). Indirect1 blocks

Fix 4) 
Jeg læser koden igennem men kan ikke ummidelbart se fejlen. Tænker at dette måske er lavet for at 
drille og at man "med vilje" har encoded indirect1 blocks med le istedet for be. 
Så jeg prøver at bytte rundt til le (på trods af spec siger det skal være be).
Dette løser heller ikke problemet.


Fix 5)
Tilføjer mere debug og det ligner vi læser forkerte indices:
FIL: Reading indirect1 as BE between 4036 and 4040

Blocksize bude være 4096 og pr. spec burde indirect1 være på mellem (4096-16, 4096-12). 
Dvs 4080-4084. Jeg opdaterer måden vi læser bytes og det virker endelig! 

## 3) Null bytes over alt

Da jeg prøver at køre ./update2d med default values får jeg "Error: Code 2000".
Jeg læser C koden og kan se det betyder forkert signatur.
Men det image jeg har fået udleveret kan umuligt have forkert signatur???
Det må betyde filen er skrevet forkert så signaturen ikke passer.. 

./update2d --onlysigned --keyfile=images/key.priv --logfile=/dev/null --verbose < images/imgA.u2d
Error: Code 2000

fix 1) Jeg opdager en fejl i ufem: vi skriver kun hele blocks. Jeg har glemt den lille detalje
at filer jo selvfølgelig ikke har størrelse af perfect blocksize. 
Jeg retter ufem til at tage højde for file size.. lad os se hvad der sker.. 

        let mut bytes_remaining = size;
        for block_index in blocks_to_read {
            println!("FIL: Loading block {block_index} into {path}");
            let data = self.get_block(block_index);
            let mut bytes_to_read = data.len();
            if bytes_remaining < blocksize {
                bytes_to_read = bytes_remaining as usize;
            }
            file.write_all(&data[..bytes_to_read]).unwrap();
            println!("FIL: Completed loading block {block_index} into {path}");
            bytes_remaining -= blocksize;
        }
