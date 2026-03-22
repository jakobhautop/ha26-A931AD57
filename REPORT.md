--- 
ID: 532189
Valgt mission: angreb via netupsrv 
---  

# Rapport

Denne rapport indeholder en beskrivelse af hvordan jeg har unpacket .u5fs images
samt hvad jeg har fundet frem til i min analyse af netup protokolen. 

# Resume 

Det lykkedes at unpacke operation.u5fs og vælge opgave 1 i /netupsrv. 
capture.pcap og netup binarien analyseres hhv i tshark og ghidra. 
Det lykkedes ikke at lave en client der kan snakke med netupsrv og
protokollen er kun knap delvist gennemskuet i følgende rapport. 

# Valg af opgave

Efter at have kigget på alle 3 opgaver:

- Angreb via netup
- Angreb via update2d 
- Angreb via firmware 

Har jeg valgt 1. opgave fordi:

- Mulighed for at lære mere om reverse engineering
    - Felt jeg er helt grøn i
    - Første erfaring med tshark og ghidra 
- Jeg tror jeg kunne "performe bedre" i de andre opgaver
    - u5fs packer virker triviel (famoust last words)
    - update2d daemon virker også mere lige til
- Men opgaven handler hovedsageligt om 
    - at dokumentere sin process 
    - og vise hvordan man tænker 
- Så jeg har valgt at bruge min tid på den opgave jeg kunne lære mest af
    - Så kan vi alle se hvordan jeg tænker i noget jeg er helt ny med 
    - Og forhåbentlig går det ikke helt galt 

Yderligere: 
Om aftenerne tirs-tors 17-19/3 timeboxede jeg 2 timer til hver opgave for at
se hvilken der var sjovest at arbejde. Det er klart opgave 1 der har den stejleste kurve for mig. 
Og det virkede som den fedeste opgave at løse i weekenden 21-22/3. 

**Updatering pr. 22/03/2026: Okay reverse engineering af netup var meget svære end jeg troede, men ret sjovt :3 

# Udpakning af operation.u5fs 

## Intro

Givet er en operation.u5fs fil der følger specifikationen i u5fs.pdf. 
Dette er filsystemet der passer til fentium kernellen. 

Det lykkedes at lave en unpacker "ufem" (u5) der kan læse en .u5fs fil og producere
et fil træ:

```
> ufem unpack operation.u5fs dump
...

> tree dump 
dump
└── root
    ├── BURNAFTERREADING.md
    ├── firmware
    │   ├── data.u5fs
    │   ├── femtium
    │   ├── femtium.sw
    │   ├── README-fmt5.md
    │   ├── README.md
    │   ├── reference.pdf
    │   ├── root.u5fs
    │   ├── start-emulator
    │   └── u5fs.pdf
    ├── full-chain
    │   └── README.md
    ├── netupsrv
    │   ├── capture.pcap
    │   ├── dog.jpg
    │   ├── netup
    │   ├── README.md
    │   └── update2d-wrapper
    ├── README.md
    └── update2d
        ├── images
        │   ├── imgA.u2d
        │   ├── imgB1.u2d
        │   ├── imgB2.u2d
        │   ├── imgC1.u2d
        │   ├── imgC2.u2d
        │   ├── imgD.u2d
        │   └── key.priv
        ├── README.md
        ├── src
        │   ├── arc4.c
        │   ├── arc4.h
        │   ├── crc32.c
        │   ├── crc32.h
        │   ├── debugging.h
        │   ├── hmac_sha256.c
        │   ├── hmac_sha256.h
        │   ├── logging.c
        │   ├── logging.h
        │   ├── main.c
        │   ├── README
        │   ├── ringbuf.c
        │   ├── ringbuf.h
        │   ├── sha256.c
        │   ├── sha256.h
        │   ├── tcp_server.c
        │   ├── tcp_server.h
        │   ├── update2d.c
        │   ├── update2d.h
        │   ├── watchdir.c
        │   ├── watchdir.h
        │   ├── xfrm_arc4.c
        │   ├── xfrm_arc4.h
        │   ├── xfrm_file.c
        │   ├── xfrm_file.h
        │   ├── xfrm_lz4.c
        │   ├── xfrm_lz4.h
        │   ├── xfrm_uudecode.c
        │   ├── xfrm_uudecode.h
        │   ├── xfrm_zlib.c
        │   ├── xfrm_zlib.h
        │   ├── xfrm.c
        │   └── xfrm.h
        └── update2d
```

## Rationaler og løsning 

Denne opgave blev løst i aftenerne tirs-fredag 10-13/3.  
Da jeg modtog opgaven var det klart at der var flere opgaver inde i operation.u5fs
men det uvidst hvilken slags opgaver det ville være. 

Denne uvished ledte til følgende rationaler.

Overvejelser:

- Hvor grundigt (production ready) tool skal vi lave?
- Krav til kode, performance, test osv? 
- Hvor elegant en løsning skal man lave vs. bare noget der virker? 
- Basalt ser hvor meget effort skal investeres i denne del af opgaven? 

Mulige stier: 

- 1) Næste opgave fortsætter måske af samme spor (mere .u5fs muligvis at lave nye images eller lignende)
- 2) Næste opgave er måske noget helt andet 

Hvis 1: 

- Så giver der mening at bygge en smart model fra start
    - God logisk model for u5fs implementation
    - Som er let at udvide med "create new" eller "edit/add/delete" individuelle filer 
    - Brug tid på at undersøge filsystemer og best practices, algorithmer etc 
- Og det giver også god mening lige at tænke sig om en ekstra gang 

Hvis 2: 

- Så giver det mening "bare at lave noget der virker" 
- som er godt nok til at man ikke ligner en kæmpe amatør :#
- tilgengæld kommer man hurtigere videre til næste opgave og har derfor mere tid til denne 


Derfor:

- Find ud af hurtigst muligt hvad næste opgave er
- Derfra kan du justere din strategi når du ved om det er sti 1 eller 2 
- Gå med Rust
    - nem (og sikker) adgang til at læse og manipulerer bytes fra disk 
    - så har du stadig en god base hvis det er sti 1
    - men lad være med at bruge energi på overhead som ikke får os tættere på løsning 
    - TUI/CLI only


### Implementering

---
Løsning: CLI tool
Kode: ufem/main.rs 
Install: `cargo install --path .`
CLI: `ufem unpack operation.u5fs dump`
Filer: u5fs.pdf, operation.u5fs 
--- 

#### Metode

1. CLI indgang der læser args via stdin 
2. Ved `ufem unpack <image> <path>` intantieres et `handle`
    - `handle` er et object der repræsenterer det u5fs image man arbjeder med 
3. Man kalder metoder på handle som læser et u5fs image og dumper et fil træ. 

#### Algoritme:

1. Læs superblock

```
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
```


2. Gem blocksize, size, rootnode

```
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
        let sb = Self::read_super_block(path).unwrap(); <-- gemmer super block på handle her 
        println!("INI: Completed Init..");
        Handle {
            path: path.to_string(),
            sb: Some(sb),
        }
    }
```

3. Rekursivt gå fra root node og besøg alle nodes

```
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
```

4. Lav hver node ud fra inode typen som beskrevet i u5fs.pdf 

Det ville være for meget at dumpe al koden her, se funktionerne: 

- ufem/src/main.rs#create_dir_with_perm
- ufem/src/main.rs#parse_dir_entries
- ufem/src/main.rs#self.dump_file 

Overordnet struktur:

- Map typen af inode til en af ovenstående funktioner
- Individuelle filer og dirs bliver dannet med Rusts standard librarys fil API 
- *Obs jeg har ikke root derfor er chown kommenteret ud i dir koden. Dette er en mangel. 

#### Indlæsning af bytes

Jeg bruger de layouts der er beskrevet i u5fs.pdf til at læse byte arrays 
og parse dem til nogle få structs i Rust koden. Der er med vilje brugt .unwrap()
alle steder for at få en panic med det samme der var en fejl. 

# Angreb via netupsrv

--- 
Værktøjer: tshark + ghidra 
filer: TSHARK.md, GHIDRA.md
løst: nej 
--- 

## Mål 

Pr. opgave definitionen er målet at lave en client der kan snakke med netup serveren. 
Denne client skal laves ved at reverse engineere protokollen der bruges til at kommunikere med serveren. 
Til rådighed har man et netværks capture (capture.pcap) og en binary for serveren. 

## Metode 

For at reverse engineere protokolen må jeg gå explorativt og empirisk til værks. 
Jeg vil studere pakkerne i capture filen via tshark og decompilere netup med ghidra.

Min overordnede plan er:

1. Gennemskue store dele af protokol via capture.pcap 
2. Derefter finde detaljer om packets og protokolen via Ghidra 
3. Implementere en ping request til serveren (eller noget der minder om aliveness)
4. Derefter udvide min client

### Dokumentation 

Jeg har været lidt i tvivl om hvordan jeg har skulle dokumentere denne quest. Men har valgt følgende
fremgangs måde:

    - Et dokument pr. toool: TSHARK.md + GHIDRA.md 
    - Hvert dokument er noter men jeg arbejder 
    - Denne rapport bliver brugt til opsumere findings
    - Denne fase utvivlsom indeholder meget gætværk
    - Derfor må man forvente at jeg kan finde ud af nogle ting 
        via disse tools. Og andre ting lærer jeg måske først
        når jeg implementerer klienten. 

## Protokol 

### Analyse i tshark

tshark er et CLI tool til wireshark (et mere kendt værktøj til at analysere netværks traffik). 
Dette værktøj laver mig analysere netværkspakkerne i capture.pcap og pbga disse kan man deducere dele af
hvordan protokolen fungerer. 

Her er hvad (jeg tror) jeg ved:

    - Ren TCP pakke kommunikation
    - Ingen JSON, XML eller lignende encoding
    - Forskellige faser: handshake, overførsel, kommandoer, afslutning
    - Forskellige typer af pakker til forskellige faser
    - dynamisk data i headers der forskellig fra payload men ændrer sig fra pakke til pakke 

Se nærmere i TSHARK.md 

### Analyse i ghidra 

Ghidra er et GUI tool udviklet af det Amerikanske NSA til at analysere binaries.

Vi finder følgende message types: 

```
netup/protocol..dict.RegisterMessageType[*netup/messages.WriteResponse]
netup/protocol..dict.RegisterMessageType[*netup/messages.WriteRequest]
netup/protocol..dict.RegisterMessageType[*netup/messages.UnlinkResponse]
netup/protocol..dict.RegisterMessageType[*netup/messages.UnlinkRequest]
netup/protocol..dict.RegisterMessageType[*netup/messages.ReadResponse]
netup/protocol..dict.RegisterMessageType[*netup/messages.ReadRequest]
netup/protocol..dict.RegisterMessageType[*netup/messages.PingResponse]
netup/protocol..dict.RegisterMessageType[*netup/messages.PingRequest]
netup/protocol..dict.RegisterMessageType[*netup/messages.OpenResponse]
netup/protocol..dict.RegisterMessageType[*netup/messages.OpenRequest]
netup/protocol..dict.RegisterMessageType[*netup/messages.ExecResponse]
netup/protocol..dict.RegisterMessageType[*netup/messages.ExecRequest]
netup/protocol..dict.RegisterMessageType[*netup/messages.Error]
netup/protocol..dict.RegisterMessageType[*netup/messages.CloseResponse]
netup/protocol..dict.RegisterMessageType[*netup/messages.CloseRequest]
netup/protocol..dict.RegisterMessageType[*netup/messages.AllocateResponse]
netup/protocol..dict.RegisterMessageType[*netup/messages.AllocateRequest]
```

#### Her går jeg i stå 

Det var planen at Ghidra skulle hjælpe mig med at se byte layouts af message types.
Men det er ikke lykkedes grundet manglende kompetence på min side.
Og da tiden er ved at rende ud lukker jeg analysen her. 

Det lykkedes at bekræfte:

1. netup laver en tcp listener i Go 
2. netup har en eller anden switch på message type 

Det første jeg ville finde ud af var 1) hvad den switcher på og 2) hvilket bytearray 
der skulle til via TCP for at trigger en ping request. Men så langt kom jeg ikke. 

# Reflesion 

Opgaven lagde op til at man skulle reverse enginere netup, som er den server 
der står for at modtage firmware updates til DDING-3000. Hvis jeg kunne have
fundet ud af hvordan protokollen havde virket i tide havde muligvis været muligt
at finde en exploit i protokolen og muligvis i server implementationen. 

En sådan exploit kunne have ladet os:

    - Poste vores egne firmware updates?
    - Muligvis plante andre filer (backdors eller lign.) i DDING-3000

Og dermed at givet os et foothold der senere kunne bruges til at exhiltrere 
efterretninger om gæsterne på hotel Hilbert. 

Det er ærgeligt jeg ikke når i mål og måske var det en fejl at jeg valgte at 
gå med reverse engineering når jeg er helt grøn. Men det var fedt at prøve
og jeg har fået rigtig meget respekt for det håndværk og blod på tanden. 

Undervejs har jeg forsøgt at dokumentere mine tanker gennem noterne i TSHARK.md og GHIDRA.md.
Hvis man er interesseret har jeg også dokumenteret et par bugs i ufem/README.md jeg
opdagede og løste mens jeg har læste de andre opgaver igennem. 

