--- 
ID: 532189
Opgave: 1. angreb gennem netupsrv 
---


# Fagtest | Hacker Akademi 2026

I dette repo finder i min besvarelse af fagtesten. 


## Filer

```
> tree
.
├── ding
│   ├── Cargo.lock
│   ├── Cargo.toml
│   ├── README.md
│   └── src
│       └── main.rs
├── GHIDRA.md
├── README.md
├── REPORT.md
├── TSHARK.md
└── ufem
    ├── Cargo.lock
    ├── Cargo.toml
    ├── README.md
    └── src
        └── main.rs
```


## Oversigt

- ding: client til at kalde netupsrv 
- ufem: cli til at unpacke .u5fs images 
- README.md: oversigt over aflevering
- REPORT.md: Selve afleveringen
- GHIDRA.md: Analyse a Netup binary
- TSHARK.md: Analyse af capture.pcap 



## Installation af Rust CLI tools 

ding og ufem kan begge installeres med cargo 

```
cargo install --path ding
cargo install --path ufem 
```



