# TSHARK Analyse noter

Mål: Reverse engineer protokol 
Metode: Empirisk analyse af capture.pcap. Jeg kigger manuelt efter sport og gentagelser. 
Tool: tshark (WireShark terminal tool)
Filer: capture.pcap

## Resume 

Client: 127.11.4.152:42027
Netup: 127.202.3.1:6666
Socket: TCP 
Visse pakke størrelser går igen hvilket kunne indikerer fast typer af beskeder: 
    - 10 bytes (en form for init/handshake?)
    - 14 bytes (en form for anerkendende response, muligvis med instruks?)
    - 223 bytes (fil overførsel?)
    - 6 bytes (en form for "afslutning"?)
Pakke layout:
    - indeks 0: pakke størrelse
    - forskellige headers?
        223: 23 bytes header + 200 payload


Hypotese for protokol:
    - Client-Server Handshake
    - Client: Start overførsel af fil og gem som "uploaded.jpeg"
    - Client-Server overførsel af fil 
    - Client kører muligvis en command /usr/bin/print-jpeg

Opfølgende spørgsmål til analyse i Ghidra
    - Kan vi finde pakke typer? 
    - Kan vi finde logikken bag dynamiske bytes ved fil overførsel?


## PCAP fil 

En .pcap fil er en capture af netværkstraffik, i dette filfælde 
kommunikation mellem DDING-3000 og en 3. part. 

Hver række af filen inkluderer en header og en payload.
tshark cli tool bruges til analyse pakkerne. 


## Helikopter blik

Først starter vi med en overblik:

    - 127.11.4.152:42027 er client 
    - 127.202.3.1:6666 er server (netup)
    - Protokol er TCP
    - Normalt SYN -> SYN ACK -> SYN handshake
    - Fase 1: Initiation:
        - #4 client sender 10 bytes
        - #6 netup sender 10 bytes
        - #8 client sender 23 bytes
        - #9 netup sender 14 bytes
    - Fase 2: Overførsel
        - client sender 223 bytes adgangen
        - server svarer med 14
    - Fase 3: Afslutning
        - Pakker >#204 har forskllige længde
        - indikerer en eller anden form for dialog efter transfer
        - muligvis commands?
    - Encoding:
        - Jeg ser ingen spor af JSON i ASCII
        - 

Ting vi ikke ved endnu
    - Første 10 bytes der sendes af hver part
    - Pakke #8 har 23 bytes inden mønsteret begynder
    - Uklart om de 80 bytes i pakke #204 er en ny del af protokol eller sidste bytes af fil
    - Uklart hvad der sker efter fil overførslen. 
    - Likely er det commands 


### Data

*Fase 1*

Normal TCP handshake og herefter en udveksling af få beskeder

```
tshark -r capture.pcap | head -8
1   0.000000 127.11.4.152 → 127.202.3.1  TCP 74 42027 → 6666 [SYN] Seq=0 Win=65495 Len=0 MSS=65495 SACK_PERM TSval=4131508747 TSecr=0 WS=128
2   0.000041  127.202.3.1 → 127.11.4.152 TCP 74 6666 → 42027 [SYN, ACK] Seq=0 Ack=1 Win=65483 Len=0 MSS=65495 SACK_PERM TSval=2333632585 TSecr=4131508747 WS=128
3   0.000075 127.11.4.152 → 127.202.3.1  TCP 66 42027 → 6666 [ACK] Seq=1 Ack=1 Win=65536 Len=0 TSval=4131508748 TSecr=2333632585
4   0.000360 127.11.4.152 → 127.202.3.1  TCP 76 42027 → 6666 [PSH, ACK] Seq=1 Ack=1 Win=65536 Len=10 TSval=4131508748 TSecr=2333632585
5   0.000387  127.202.3.1 → 127.11.4.152 TCP 66 6666 → 42027 [ACK] Seq=1 Ack=11 Win=65536 Len=0 TSval=2333632585 TSecr=4131508748
6   0.000526  127.202.3.1 → 127.11.4.152 TCP 76 6666 → 42027 [PSH, ACK] Seq=1 Ack=11 Win=65536 Len=10 TSval=2333632585 TSecr=4131508748
7   0.000606 127.11.4.152 → 127.202.3.1  TCP 66 42027 → 6666 [ACK] Seq=11 Ack=11 Win=65536 Len=0 TSval=4131508748 TSecr=2333632585
8   0.000861 127.11.4.152 → 127.202.3.1  TCP 89 42027 → 6666 [PSH, ACK] Seq=11 Ack=11 Win=65536 Len=23 TSval=4131508748 TSecr=2333632585
```

*Fase 2*

- Konstant udveksling af pakker. 
- Client sender 223 bytes og server svarer med 14. 
- Pakker #9 til #202.
- Det er interessant at dette mønster starter med at NETUP sender 14 bytes og først da svarer client med 223. 
- Da vi ved der er tale om en underlig protokol kan det være at disse 14 bytes er NETUP der siger hvilke bytes fra filen den vil be om eller noget andet mystisk.

Eksempel:

```
 10   0.001289 127.11.4.152 → 127.202.3.1  TCP 289 42027 → 6666 [PSH, ACK] Seq=34 Ack=25 Win=65536 Len=223 TSval=4131508749 TSecr=2333632586
11   0.001459  127.202.3.1 → 127.11.4.152 TCP 80 6666 → 42027 [PSH, ACK] Seq=25 Ack=257 Win=65536 Len=14 TSval=2333632586 TSecr=4131508749
12   0.001556 127.11.4.152 → 127.202.3.1  TCP 289 42027 → 6666 [PSH, ACK] Seq=257 Ack=39 Win=65536 Len=223 TSval=4131508749 TSecr=2333632586
13   0.001641  127.202.3.1 → 127.11.4.152 TCP 80 6666 → 42027 [PSH, ACK] Seq=39 Ack=480 Win=65536 Len=14 TSval=2333632586 TSecr=4131508749
14   0.001718 127.11.4.152 → 127.202.3.1  TCP 289 42027 → 6666 [PSH, ACK] Seq=480 Ack=53 Win=65536 Len=223 TSval=4131508749 TSecr=2333632586
15   0.001807  127.202.3.1 → 127.11.4.152 TCP 80 6666 → 42027 [PSH, ACK] Seq=53 Ack=703 Win=65536 Len=14 TSval=2333632586 TSecr=4131508749
```

*Fase 3* 

- Afsluttende fase
- Uklart om #204 er de sidste 80 bytes af filen eller en ny del af protokol
- Herefter udveksling af pakker af forskellig størrelse
    - #205: S -> 14B
    - #206: C -> 14B
    - #207: S -> 6B 
    - #208: C -> 19B
    - #209: S -> 14B
    - #210: C -> 34B
    - #211: S -> 15B
    - #212: C -> 14B
    - #213: S -> 6B
    - #214: C -> 19B
    - #215: S -> 6B
- Herefter normalt FIN ACK -> FIN ACK -> ACK flow


Data: 
```
 203   0.024271  127.202.3.1 → 127.11.4.152 TCP 80 6666 → 42027 [PSH, ACK] Seq=1369 Ack=21665 Win=65536 Len=14 TSval=2333632609 TSecr=4131508772
204   0.024339 127.11.4.152 → 127.202.3.1  TCP 146 42027 → 6666 [PSH, ACK] Seq=21665 Ack=1383 Win=65536 Len=80 TSval=4131508772 TSecr=2333632609
205   0.024401  127.202.3.1 → 127.11.4.152 TCP 80 6666 → 42027 [PSH, ACK] Seq=1383 Ack=21745 Win=65536 Len=14 TSval=2333632609 TSecr=4131508772
206   0.024454 127.11.4.152 → 127.202.3.1  TCP 80 42027 → 6666 [PSH, ACK] Seq=21745 Ack=1397 Win=65536 Len=14 TSval=4131508772 TSecr=2333632609
207   0.024571  127.202.3.1 → 127.11.4.152 TCP 72 6666 → 42027 [PSH, ACK] Seq=1397 Ack=21759 Win=65536 Len=6 TSval=2333632609 TSecr=4131508772
208   0.024836 127.11.4.152 → 127.202.3.1  TCP 85 42027 → 6666 [PSH, ACK] Seq=21759 Ack=1403 Win=65536 Len=19 TSval=4131508772 TSecr=2333632609
209   0.024959  127.202.3.1 → 127.11.4.152 TCP 80 6666 → 42027 [PSH, ACK] Seq=1403 Ack=21778 Win=65536 Len=14 TSval=2333632609 TSecr=4131508772
210   0.025025 127.11.4.152 → 127.202.3.1  TCP 100 42027 → 6666 [PSH, ACK] Seq=21778 Ack=1417 Win=65536 Len=34 TSval=4131508773 TSecr=2333632609
211   0.026747  127.202.3.1 → 127.11.4.152 TCP 81 6666 → 42027 [PSH, ACK] Seq=1417 Ack=21812 Win=65536 Len=15 TSval=2333632611 TSecr=4131508773
212   0.026852 127.11.4.152 → 127.202.3.1  TCP 80 42027 → 6666 [PSH, ACK] Seq=21812 Ack=1432 Win=65536 Len=14 TSval=4131508774 TSecr=2333632611
213   0.026940  127.202.3.1 → 127.11.4.152 TCP 72 6666 → 42027 [PSH, ACK] Seq=1432 Ack=21826 Win=65536 Len=6 TSval=2333632611 TSecr=4131508774
214   0.027054 127.11.4.152 → 127.202.3.1  TCP 85 42027 → 6666 [PSH, ACK] Seq=21826 Ack=1438 Win=65536 Len=19 TSval=4131508775 TSecr=2333632611
215   0.027245  127.202.3.1 → 127.11.4.152 TCP 72 6666 → 42027 [PSH, ACK] Seq=1438 Ack=21845 Win=65536 Len=6 TSval=2333632612 TSecr=4131508775
```

## Payloads

Herfra kunne det være interessant at kigge på bytes forskkellige pakker af samme størrelse for at se om der er noget der går igen? (headers eller lignende?)

### Pakker af 10 bytes 

Efter TCP handshake: 
- #4 client sender 10 bytes
- #6 NETUP svarer med 10 bytes.
- Hvis man sammenligner den eneste forskel byte #2
- Så er spørgsmålet hvad man kan få ud af det her? 
    - En form for handshake?
    - Man deler en secret? Eller hver sin nøgle? Eller bliver enige om en version?
- Jeg checkede også disse for ASCII men det ligner ikke noget.. 

Data:
```
tshark -r capture.pcap -Y "frame.number==4" -T fields -e data
0a 70 01 b6 00 96 14 66 15 06
tshark -r capture.pcap -Y "frame.number==6" -T fields -e data
0a 50 01 b6 00 96 14 66 15 06
```

### Pakken med 23 bytes 

Inden fil 223-14 mønsteret begynder sender client 23 bytes. 
Lad os se hvad der sker i den:

```
tshark -r capture.pcap -Y "frame.number==8" -T fields -e data
17 61 35 20 06 10 0c 75 70 6c 6f 61 64 65 64 2e 6a 70 67 00 00 4b fb

tshark -r capture.pcap -Y "frame.number==8" -T fields -e data | xxd -r -p
a5
   uploaded.jpgK�%
```

Hvad jeg ser:

- ASCII representationen indikerer at pakken har to dele:
    - 1) Jeg gætter på disse bytes er "start fil overførsel og gem til"
    - 2) Fil navn 

### Pakker af 14 bytes 

Netup svarer gentagende gange med 14 bytes. Lad os se hvad der sker i dem.

- Jeg kan se med `tshark -r capture.pcap -Y "tcp.len==14" -x` at pakkerne er meget ens 
- Men der er små variationer, lad os checke md5sum af disse og sorterer pr. hash og list pakkerne

```
tshark -r capture.pcap -Y "tcp.len==14" -T fields -e frame.number -e data \
| while read num data; do
  echo -n "$data" | xxd -r -p | md5sum | awk "{print \$1, \"$num\"}"
done \
| sort \
| awk '{map[$1]=map[$1]" "$2} END {for (h in map) print h, "->", map[h]}'
8909c448b45256ed35086c6ae8377516 ->  101 103 105 107 109 11 111 113 115 117 119 121 123 125 127 129 13 131 133 135 137 139 141 143 145 147 15 151 153 155 157 163 165 167 169 17 171 173 175 177 179 181 183 185 189 191 193 195 197 199 201 203 21 23 25 27 29 31 33 35 37 39 41 43 45 47 49 51 53 55 57 59 61 63 65 67 69 73 75 77 79 81 83 85 87 89 91 93 95 97 99
47116fb7d2aa5a19511e30d6705b04bc ->  206 212
ccd0ed7247c9de4f9ee9961e0cd5837f ->  205
ecb8f477dcc391884a49a039e42cdc67 ->  209
aa86ad2fbe691a9ddfa6c71cf7dd388c ->  149 159 161 187 19 71
9f2c5c901a4fbca4bb13ec217c24da5a ->  9
```

- Okay sorry det er lidt bøvlet command.
- Men vi ser 6 forskellige pakker af 14 bytes
- Lang de fleste pakker har samme md5sum 
- Lad os prøve at sammenligne dem byte for byte via pakke numer og xxd 

```
tshark -r capture.pcap -Y "frame.number==101" -T fields -e data
0e 57 00 d0 00 c9 00 00 00 00 00 00 00 c8
tshark -r capture.pcap -Y "frame.number==206" -T fields -e data
0e 63 00 10 00 09 00 00 00 00 00 00 00 08
tshark -r capture.pcap -Y "frame.number==209" -T fields -e data
0e 4f 00 10 00 09 00 00 00 00 00 00 00 08
tshark -r capture.pcap -Y "frame.number==149" -T fields -e data
0e 57 00 cf 00 c8 00 00 00 00 00 00 00 c7
tshark -r capture.pcap -Y "frame.number==9" -T fields -e data
0e 41 00 10 00 09 00 00 00 00 00 00 00 08
```
- 1. byte kunne være en pakke type (ens for alle?)
- 2., 4., 6. og 14. byte er forskellige 
- resten er ens? 
- Dette tegner et billede af samme type men med 4 variationer 
- hvis vi leger denne pakke "anerkender noget" kunne disse 4 slags være
    - Typen af det der bliver anerkendt
    - Størrelsen af der bliver anerkendt
    - Success eller ej? 
    - Lad os lave et set for hver byte placering
        - 2: 57 (2), 63 (1), 4f (1), 57 (1), 41 (1) 
        - 4: d0 (1), 10 (3), cf (1)
        - 6: c9 (1), 98 (3), c8 (1)
        - 14: c8 (1), 08 (3), c7 (1) 
    - 14. byte er altid en mindre end 6.? 
- Måske kunne man navle pille mere her men jeg går videre 

### Pakker af 223 bytes 

Det kunne også være sjovt at se bytes i pakkerne med 223 bytes for at spotte headers 
eller andre mønstre. 

Det er for meget data at dumpe her. Men eksempelvis:

```
tshark -r capture.pcap -Y "tcp.len==223" -T fields -e data \
| while read line; do
  echo "$line" | xxd -r -p | xxd -c 16 -g 1
  echo
done
00000000: df 77 63 6c 34 54 00 00 00 00 00 00 00 08 00 00  .wcl4T..........
00000010: 00 00 00 00 00 00 c8 ff d8 ff e0 00 10 4a 46 49  .............JFI
00000020: 46 00 01 01 01 00 48 00 48 00 00 ff db 00 43 00  F.....H.H.....C.
00000030: 10 0b 0c 0e 0c 0a 10 0e 0d 0e 12 11 10 13 18 28  ...............(
00000040: 1a 18 16 16 18 31 23 25 1d 28 3a 33 3d 3c 39 33  .....1#%.(:3=<93
00000050: 38 37 40 48 5c 4e 40 44 57 45 37 38 50 6d 51 57  87@H\N@DWE78PmQW
00000060: 5f 62 67 68 67 3e 4d 71 79 70 64 78 5c 65 67 63  _bghg>Mqypdx\egc
00000070: ff db 00 43 01 11 12 12 18 15 18 2f 1a 1a 2f 63  ...C......./../c
00000080: 42 38 42 63 63 63 63 63 63 63 63 63 63 63 63 63  B8Bccccccccccccc
00000090: 63 63 63 63 63 63 63 63 63 63 63 63 63 63 63 63  cccccccccccccccc
000000a0: 63 63 63 63 63 63 63 63 63 63 63 63 63 63 63 63  cccccccccccccccc
000000b0: 63 63 63 63 63 ff c2 00 11 08 01 f4 01 f4 03 01  ccccc...........
000000c0: 11 00 02 11 01 03 11 01 ff c4 00 1a 00 00 03 01  ................
000000d0: 01 01 01 00 00 00 00 00 00 00 00 00 00 00 01     ...............
```

Ved at manuelt skimme pakkerne i gennem kan jeg se alle følger
en overordnet struktur. Mit gæt er:

[< - header af 23 bytes - >][< - payload af 200 bytes - >] 

For#di der lader til at være en semi-fast struktur i de første 89 bytes efterfulgt
af en række bytes der ændres mellem hver pakke. Man kan kalde headers for semi-struktureret
fordi nogle enkelte byte positioner følger et mønster og ændres mellem pakkerne.
For at holde det kort er mit gæt for file-transfer pakkerne at de følger dette 223-byte layout: 

Layout:
df 77 @1 @2 @3 @4 00 00 00 00 00 00 00 08 00 00 
00 00 00 00 @5 @6 c8 <- PA YL OA D- -- -- -- -- 
-- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- 
-- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- 
-- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- 
-- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- 
-- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- 
-- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- 
-- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- 
-- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- 
-- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- 
-- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- 
-- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- 
-- -- -- -- -- -- -- -- -- -- -- -- -- -- ->

Hvor de dynamiske bytes følger disse mønstre: 

@1 @2 @3 @4 @5 @6
=================
63 6c 34 54 00 00
c3 6c 2f 27 00 c8
20 cb 39 f5 01 90
4d df 50 40 02 58
65 21 59 8c 03 20
31 ad 5a 5c 03 e7
bf 0f 68 0f 04 af

I decimaler:

 @1   @2  @3  @4  @5  @6 
=======================
 99  108  52  84   0   0
195  108  47  39   0 200
 32  203  57 245   1 144
 77  223  80  64   2  88
101   33  89 140   3  32
 49  173  90  92   3 231
191   15 104  15   4 175

Ikke et klart mønster i disse. 
Undtagen @5 der inkrementerer eller gentager sig. 


### Pakken af 80 bytes 

Denne var uklart om den var indeholdte en payload og var 
en del af file transfer - eller om den var sin egen slags besked.
Hvis hypotesen om at headeren er 89 bytes er sand: så er den ikke
da de 80 bytes ville være mindre end headeren? 

Lad os inspicere den (pakke nummer 204):

```
tshark -r capture.pcap -Y "frame.number==204" -T fields -e data \
| while read line; do
  echo "$line" | xxd -r -p | xxd -c 16 -g 1
  echo
done
00000000: 50 77 51 86 1b 9b 00 00 00 00 00 00 00 08 00 00  
00000010: 00 00 00 00 4b c2 39 88 98 10 ea 3c 13 6f 86 58  
00000020: 89 65 f1 84 8c 81 b6 48 64 cd ea 3d c3 62 6a f5  
00000030: 12 00 93 c1 90 5d d7 6f 32 25 1c 96 0b 88 0f 88  
00000040: 61 92 6b 1c 30 b2 77 49 83 98 1e 18 fc 3f ff d9  
```

Noter her
- 1. byte er ikke længere df (kunne være anden pakke typpe)
- Men udover dette kunne det godt ligne samme 20 bytes header. 

### Andre pakker 

Efter len 80 pakken ser vi følgende pakker

```
tshark -r capture.pcap -Y "frame.number>204" -T fields -e data \
| while read line; do
  echo "$line" | xxd -r -p | xxd -c 16 -g 1
  echo
done
00000000: 0e 57 00 41 00 3a 00 00 00 00 00 00 00 39        .W.A.:.......9

00000000: 0e 63 00 10 00 09 00 00 00 00 00 00 00 08        .c............

00000000: 06 43 00 00 00 01                                .C....

00000000: 13 6f 20 67 04 ca 0c 75 70 6c 6f 61 64 65 64 2e  .o g...uploaded.
00000010: 6a 70 67                                         jpg

00000000: 0e 4f 00 10 00 09 00 00 00 00 00 00 00 08        .O............

00000000: 22 65 47 c7 07 3c 00 00 00 00 00 00 00 08 13 2f  "eG..<........./
00000010: 75 73 72 2f 62 69 6e 2f 70 72 69 6e 74 2d 6a 70  usr/bin/print-jp
00000020: 65 67                                            eg

00000000: 0f 45 00 09 00 01 00 00 00 00 00 00 00 00 00     .E.............

00000000: 0e 63 00 10 00 09 00 00 00 00 00 00 00 08        .c............

00000000: 06 43 00 00 00 01                                .C....

00000000: 13 75 20 67 04 ca 0c 75 70 6c 6f 61 64 65 64 2e  .u g...uploaded.
00000010: 6a 70 67                                         jpg

00000000: 06 55 00 00 00 01                                .U....
```

Noter
- Klarer paths i ASCII delen "uploaded.jpg" + "/usr/bin/print-jpeg"
- gad vide om /usr/bin/print-jpeg er en binary eller et script?
- Første byte er størrelsen på pakken! Ej okay obvious..

### JPEG specifikke ting 

Vi kan lede efter special JPG meta data og måske lærer noget mere? 

Vi kan søge efter strings:

```
tshark -r capture.pcap -Y "tcp.len>0" -T fields -e data \
| xxd -r -p | strings | grep -i "jfif\|exif\|jpeg\|jpg"
uploaded.jpg
JFIF
uploaded.jpg
/usr/bin/print-jpeg
uploaded.jpg
```

Og find JFIF i pakke nummer 10:

```
tshark -r capture.pcap -Y "tcp contains \"JFIF\"" \
-T fields -e frame.number
10
```

Vi kan se ovenfor at pakke nummer 10 er den første af 223 bytes. 



