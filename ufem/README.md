
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

Fix 2) 
