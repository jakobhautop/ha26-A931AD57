# Analyse i Ghidra

Ghidra er et OS tool til at decompile og analysere binaries. 
Det er udviklet af NSA til dette formål og det decompiler binær kode til C. 

## Metode

    - Siden vi har en binary gætter på at jeg skal lede efter spor fra frameworks
        - Go (Prøve operationen var en go server), C/++, Rust etc. 
    - Jeg vil starte ude fra og ind, gren for gren:
        - Prøv at finde main og finde ud af hvad der sker der
        - Hvilke funktioner kalder main?
        - Hvad sker der i disse osv. osv
        - Specielt vil jeg kigge efter ting der rimmer på TCP og sockets 
    - Jeg forsøger at lappe de huller jeg fandt TSHARK.md
        - Pattern i headers? 
        - Dynamiske bytes mellem pakker med data?
    - Jeg vil samtidigt undersøge vigtige system-kald
        - Kan jeg se hvornår der skrives til filer?
        - Kan jeg se om der bliver kaldt kommandoer via. et system API? 
        - Hvordan skabes inputs til disse kald? Er det noget man kan bruge? 

## Setup 

Jeg loader min netup binary ind i Ghidra og tager den derfra. 

## Main

Jeg ser at: 
    - måske er det go kode? (gopanic())
    - Kald til netup/protocol.Socket + Connection
    - kald til os.mkdirAll(), os.Getwd(), 


### Ghidra -> Main 

```

/* WARNING: Removing unreachable block (ram,0x005001d1) */
/* WARNING: Removing unreachable block (ram,0x00500216) */
/* WARNING: Unknown calling convention -- yet parameter storage is locked */

void main.main(void)

{
  uint8 *puVar1;
  runtime.funcval *fn;
  internal/abi.ITab *extraout_RAX;
  void *pvVar2;
  internal/abi.ITab *piVar3;
  uintptr *in_R11;
  long unaff_R14;
  code **in_XMM15_Qa;
  error eVar5;
  string sVar6;
  string sVar7;
  string listen;
  interface_{} e;
  interface_{} e_00;
  string format;
  interface_{} e_01;
  string sVar8;
  interface_{} e_02;
  []interface_{} a;
  []interface_{} a_00;
  multireturn{string;error}.conflict1 mVar9;
  []string elem;

/// Dette kunne være et spor til protokol

  multireturn{netup/protocol.Socket *;error} mVar10;
  multireturn{netup/protocol.Connection *;error} mVar11;

/// 

  code *local_50;
  netup/protocol.Socket *local_48;
  uint8 *local_40;
  int iStack_38;
  undefined *local_30;
  undefined8 uStack_28;
  undefined1 local_20 [16];
  code **local_10;
  string path;
  string format_00;
  internal/abi.ITab *piVar4;
  
                    /* Unresolved local var: netup/protocol.Socket * listener@[???]
                       Unresolved local var: error err@[???] */
  while (&local_10 <= *(code ****)(unaff_R14 + 0x10)) {
    runtime.morestack_noctxt();
  }

/// Her laves et dir med sVar8? 

  sVar8.len = 5;
  sVar8.str = &DAT_005467f7;
  local_10 = in_XMM15_Qa;
  eVar5 = os.MkdirAll(sVar8,0x1e8);

/// 

/// 1) Denne if fører til socket listen ..  
  if (eVar5.tab == (internal/abi.ITab *)0x0) {
/// 
    mVar9 = os.Getwd();
    pvVar2 = mVar9.err.data;
    piVar3 = mVar9.err.tab;
    main.absSpoolDir.len = mVar9.dir.len;
    puVar1 = mVar9.dir.str;
    if (runtime.writeBarrier._0_4_ != 0) {
      puVar1 = (uint8 *)runtime.gcWriteBarrier2();
      *in_R11 = (uintptr)puVar1;
      in_R11[1] = (uintptr)main.absSpoolDir.str;
    }
    main.absSpoolDir.str = puVar1;

/// 2) denne if fører til socket listen .. 
    if (piVar3 == (internal/abi.ITab *)0x0) {
/// 
      iStack_38 = main.absSpoolDir.len;
      uStack_28 = 5;
      local_30 = &DAT_005467f7;
      elem.len = 2;
      elem.array = (string *)&local_40;
      elem.cap = 2;
      local_40 = puVar1;
      sVar6 = path.Join(elem);
      path.len = sVar6.len;
      if (runtime.writeBarrier._0_4_ != 0) {
        main.absSpoolDir.len = path.len;
        puVar1 = (uint8 *)runtime.gcWriteBarrier2();
        *in_R11 = (uintptr)puVar1;
        in_R11[1] = (uintptr)main.absSpoolDir.str;
        sVar6.len = main.absSpoolDir.len;
        sVar6.str = puVar1;
      }
      path.str = sVar6.str;
      main.absSpoolDir = sVar6;
      sVar7 = path.Clean(path);
      main.absSpoolDir.len = sVar7.len;
      if (runtime.writeBarrier._0_4_ != 0) {
        puVar1 = (uint8 *)runtime.gcWriteBarrier2();
        *in_R11 = (uintptr)puVar1;
        in_R11[1] = (uintptr)main.absSpoolDir.str;
        sVar7.len = main.absSpoolDir.len;
        sVar7.str = puVar1;
      }
      listen.len = 5;
      listen.str = &DAT_005467fc;
      main.absSpoolDir = sVar7;
/// Her starter socket 
      mVar10 = netup/protocol.NewSocket(listen);
/// 
      piVar3 = mVar10.~r1.data;
      piVar4 = mVar10.~r1.tab;
      if (piVar4 == (internal/abi.ITab *)0x0) {
        local_50 = main.main.deferwrap1;
        local_10 = &local_50;
        local_48 = mVar10.~r0;
        while( true ) {
          mVar11 = netup/protocol.(*Socket).Accept(mVar10.~r0);
          piVar4 = mVar11.~r1.data;
          if (mVar11.~r1.tab != (internal/abi.ITab *)0x0) break;
          fn = runtime.newobject((internal/abi.Type *)&DAT_005247c0);
          fn->fn = (uintptr)main.main.gowrap2;
          if (runtime.writeBarrier._0_4_ != 0) {
            fn = (runtime.funcval *)runtime.gcWriteBarrier1();
            *in_R11 = (uintptr)mVar11.~r0;
          }
                    /* Unresolved local var: netup/protocol.Connection * conn@[???]
                       Unresolved local var: error err@[???] */
          fn[1].fn = (uintptr)mVar11.~r0;
          runtime.newproc(fn);
        }
        e.data = piVar4;
        e._type = (mVar11.~r1.tab)->Type;
        piVar3 = piVar4;
        runtime.gopanic(e);
      }
      e_00.data = piVar3;
      e_00._type = piVar4->Type;
      runtime.gopanic(e_00);
    }
    local_20._8_8_ = pvVar2;
    local_20._0_8_ = piVar3->Type;
    format.len = 0x2b;
    format.str = &DAT_0054f594;
    a.len = 1;
    a.array = (interface_{} *)local_20;
    a.cap = 1;
    sVar8 = fmt.Sprintf(format,a);
    pvVar2 = runtime.convTstring(sVar8);
    e_01.data = pvVar2;
    e_01._type = (internal/abi.Type *)&DAT_005139a0;
    runtime.gopanic(e_01);
    eVar5.data = pvVar2;
    eVar5.tab = extraout_RAX;
  }
  local_20._8_8_ = eVar5.data;
  local_20._0_8_ = (eVar5.tab)->Type;
  format_00.len = 0x24;
  format_00.str = &DAT_0054dd00;
  a_00.len = 1;
  a_00.array = (interface_{} *)local_20;
  a_00.cap = 1;
  sVar8 = fmt.Sprintf(format_00,a_00);
  pvVar2 = runtime.convTstring(sVar8);
  e_02.data = pvVar2;
  e_02._type = (internal/abi.Type *)&DAT_005139a0;
  runtime.gopanic(e_02);
  runtime.deferreturn();
  return;
}
```

### Hurtig strings search 

Det at jeg kan se netup/protocol i koden overfor for mig til at tænke
at jeg kan måske søge strings i binarien og sortere på netup. 

Bingo - det er et go project

```
/builds/campaigns/2026-admission/challenge/netup/protocol/Message.go
/builds/campaigns/2026-admission/challenge/netup/protocol/Connection.go
/builds/campaigns/2026-admission/challenge/netup/protocol/Packet.go
/builds/campaigns/2026-admission/challenge/netup/protocol/Socket.go
/builds/campaigns/2026-admission/challenge/netup/messages/AllocateRequest.go
/builds/campaigns/2026-admission/challenge/netup/messages/AllocateResponse.go
/builds/campaigns/2026-admission/challenge/netup/messages/CloseRequest.go
/builds/campaigns/2026-admission/challenge/netup/messages/CloseResponse.go
/builds/campaigns/2026-admission/challenge/netup/messages/Error.go
/builds/campaigns/2026-admission/challenge/netup/messages/ExecRequest.go
/builds/campaigns/2026-admission/challenge/netup/messages/ExecResponse.go
/builds/campaigns/2026-admission/challenge/netup/messages/OpenRequest.go
/builds/campaigns/2026-admission/challenge/netup/messages/OpenResponse.go
/builds/campaigns/2026-admission/challenge/netup/messages/PingRequest.go
/builds/campaigns/2026-admission/challenge/netup/messages/PingResponse.go
/builds/campaigns/2026-admission/challenge/netup/messages/ReadRequest.go
/builds/campaigns/2026-admission/challenge/netup/messages/ReadResponse.go
/builds/campaigns/2026-admission/challenge/netup/messages/UnlinkRequest.go
/builds/campaigns/2026-admission/challenge/netup/messages/UnlinkResponse.go
/builds/campaigns/2026-admission/challenge/netup/messages/WriteRequest.go
/builds/campaigns/2026-admission/challenge/netup/messages/WriteResponse.go
/builds/campaigns/2026-admission/challenge/netup/main.go
```

Og vi har disse message types:

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

### netup funktioner 

I Ghirda lykkedes det er finde en mappe med funktioner for netup. 
Det ville være nice starte en client med en simpelt ping function hvorefter man 
kunne expanderer der fra. 

Hvis vi kigger på de funktioner der konverter en ping request til og fra bytes:

netup/messages.(*PingReuqest).Marshal:

```

/* WARNING: Unknown calling convention */
/* DWARF original prototype: void netup/messages.(*PingRequest).Marshal(netup/messages.PingRequest *
   p, []uint8 payload, int ~r0, error ~r1) */

multireturn{int;error}.conflict
netup/messages.(*PingRequest).Marshal(netup/messages.PingRequest *p,[]uint8 payload)

{
  long unaff_R14;
  multireturn{int;error}.conflict mVar1;
  []interface_{} values;
  netup/messages.PingRequest *p_spill;
  []uint8 payload_spill;
  undefined *local_18;
  void *pvStack_10;
  
  while (&stack0x00000000 <= *(undefined1 **)(unaff_R14 + 0x10)) {
    runtime.morestack_noctxt();
  }
  pvStack_10 = runtime.convT32(p->Sequence);
  local_18 = &DAT_00513b20;
  values.len = 1;
  values.array = (interface_{} *)&local_18;
  values.cap = 1;
  mVar1 = netup/protocol.Encode(payload,values);
  return mVar1;
}

Og UnMarshal:

```

V/* WARNING: Unknown calling convention */
/* DWARF original prototype: void netup/messages.(*PingRequest).Unmarshal(netup/messages.PingRequest
   * p, []uint8 payload, error ~r0) */

error netup/messages.(*PingRequest).Unmarshal(netup/messages.PingRequest *p,[]uint8 payload)

{
  long unaff_R14;
  error eVar1;
  []interface_{} values;
  netup/messages.PingRequest *p_spill;
  []uint8 payload_spill;
  undefined *local_18;
  netup/messages.PingRequest *local_10;
  
  while (&stack0x00000000 <= *(undefined1 **)(unaff_R14 + 0x10)) {
    runtime.morestack_noctxt();
  }
  local_18 = &DAT_0050ea00;
  values.len = 1;
  values.array = (interface_{} *)&local_18;
  values.cap = 1;
  local_10 = p;
  eVar1 = netup/protocol.Decode(payload,values);
  return eVar1;
}


```

Så kan vi se kald til netup/protocol.Decode(payload, values) og en tilsvarende Encode funktion.

Så hvis vi skal lave en PingRequest fra vores client kunne det være nice at se hvordan Decode fungerer.

Her finder vi en kæmpe switch statement: 

```

/* WARNING: Unknown calling convention */
/* DWARF original prototype: void netup/protocol.Decode([]uint8 source, []interface_{} values, error
   ~r0) */

error netup/protocol.Decode([]uint8 source,[]interface_{} values)

{

/// funcktionen start med at definere en masse variables .. 

/// Herefter laves en switch statement på piVar3
/// Jeg gætter på dette er typen af object.. 

  piVar3 = __dest->_type;
  puVar4 = __dest->data;
  if (piVar3 == (internal/abi.Type *)0x0) goto switchD_004fd0fb_caseD_1;
  local_10 = __dest;
  switch(piVar3->Hash >> 0x19 & 7) {
  case 0:
    /// Type 0 

    case 2:
    /// Type 2 
    case 3:
    /// Type 3 
     case 4:
     /// Type 4
   case 5:
   /// Type 5
    case 6:
    /// Type 6 
    case 7:
    /// Type 7
}

```

Så er spørgsmålet hvilken type der svarer til hvilken request? 

Hvis vi følger piVar3 kan vi se den kommer fra values som er et array af interfaces (?):

```
error netup/protocol.Decode([]uint8 source,[]interface_{} values)
..
__dest = values.array;
..
piVar3 = __dest->_type;
```

hmmm. Nu er gode råd dyre. Så Dette kunne jo tyde på at når Decode() bliver kaldt, så ved
man allerede der hvilken type det er?
