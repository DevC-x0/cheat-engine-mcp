# 📝 TaskbarHero Reverse Engineering Offsets & Structures

Dokumen ini merangkum seluruh hasil pemindaian offset RVA (Relative Virtual Address) pada pustaka `GameAssembly.dll` dan struktur kelas Unity/IL2CPP yang digunakan dalam pembuatan cheat.

---

## 🎯 RVA Offsets (GameAssembly.dll)

Semua alamat di bawah ini adalah offset relatif (RVA) dari base address modul `GameAssembly.dll` saat runtime.

| No | RVA Offset | Name | Description | Register / Return Value |
|---|---|---|---|---|
| 1 | `0xC3B810` | `godmode_ph` | Entry point fungsi penerima damage fisik hero | Intercept damage value di `%xmm1` |
| 2 | `0xC3A860` | `godmode_pd` | Entry point fungsi penerima damage proyektil hero | Intercept damage value di `%xmm1` |
| 3 | `0x958ADC` | `attack_damage` | Fungsi kalkulasi damage serangan hero | Mengalikan `%xmm0` dengan `1000.0` |
| 4 | `0x8B25E0` | `cube_exp` | Fungsi reward EXP kubus | Mengalikan `%xmm0` dengan `1000.0` |
| 5 | `0x920760` | `Unit.gtw` | Awal fungsi pengambil durasi interval serangan | Pengali attack speed dasar |
| 6 | `0x920786` | `unit_gtw_exit` | Exit point fungsi interval serangan hero | Membagi `%xmm0` dengan `2.0` (Attack Speed x2) |
| 7 | `0x9F88C0` | `yz.kpm` | Fungsi lookup statistik (StatType) dari unit | Dipakai untuk detour Area of Effect (StatType 8) |
| 8 | `0xC23110` | `Hero.gqd` | Awal fungsi pengambil Movement Speed hero | Override kecepatan berjalan |
| 9 | `0xC2314B` | `hero_speed_exit1` | Exit point 1 fungsi Movement Speed hero | Menyetel `%xmm0` ke `15.95` (2500 flat UI) |
| 10| `0xC23191` | `hero_speed_exit2` | Exit point 2 fungsi Movement Speed hero | Menyetel `%xmm0` ke `15.95` (2500 flat UI) |

---

## 🧬 Class & Struct Layouts (IL2CPP)

### 1. `Unit` Class (MonoBehaviour)
* **TypeDefIndex**: 2476
* **Size**: `0x35E`

| Field Name | Type | Offset | Description |
|---|---|---|---|
| `b_isHero` | `bool` | `0x100` | Menandakan apakah unit merupakan Hero (`1`) atau Monster (`0`) |
| `bcyp` | `ObscuredFloat` | `0x104` | Stats float terenkripsi |
| `bcyq` | `ObscuredFloat` | `0x118` | **Movement Speed** |
| `bcyr` | `ObscuredFloat` | `0x12C` | **Attack Speed** (Interval serangan) |

---

### 2. `Hero` Class (Inherits `Unit`)
* **TypeDefIndex**: 2467

| Field Name | Type | Offset | Description |
|---|---|---|---|
| `cache` | `vd` | `0x3A8` | Pointer ke objek data runtime hero |

---

### 3. `vd` Class (Inherits `vk`)
* **TypeDefIndex**: 2807

| Field Name | Type | Offset | Description |
|---|---|---|---|
| `bfdc` | `HeroInfoData` | `0x30` | Informasi profil hero |

---

### 4. `HeroInfoData` Class
| Field Name | Type | Offset | Description |
|---|---|---|---|
| `bfdm` | `int` | `0x30` | **Hero ID** (Sorcerer: `201`, Ranger: `301`, Priest: `401`) |

---

### 5. `ObscuredFloat` Struct (Anti-Cheat Toolkit)
* **TypeDefIndex**: 19571
* **Size**: `0x14` bytes

| Field Name | Type | Offset | Description |
|---|---|---|---|
| `hash` | `int` | `0x0` | Hash verifikasi |
| `hiddenValue` | `int` | `0x4` | Nilai float terenkripsi (XORed dengan key) |
| `currentCryptoKey` | `int` | `0x8` | Kunci dekripsi XOR |
| `fakeValue` | `float` | `0xC` | Nilai bayangan tidak terenkripsi (untuk editor) |

> [!NOTE]
> Rumus dekripsi manual untuk `ObscuredFloat`:
> `decrypted_float = (float)(hiddenValue ^ currentCryptoKey)`

---

## 🔄 Otomasi Pemindaian Offset Saat Game Update (v0.4.2)

Ketika game *Taskbar Heroes* melakukan pembaruan (update), nama kelas dan fungsi IL2CPP akan diacak ulang (*obfuscated*). Jangan melakukan pemindaian manual dari awal, gunakan tool otomasi berikut:

### 1. Ekstraksi Metadata Otomatis (`il2cpp_run_dumper`)
Mengekstrak file `dump.cs` dan metadata langsung dari proses game yang sedang berjalan (PID) atau path file:
```json
{
  "name": "il2cpp_run_dumper",
  "arguments": {
    "pid": 23171
  }
}
```

### 2. Penelusuran Anchor Heuristik Otomatis (`il2cpp_scan_taskbarhero_offsets`)
Membaca `dump.cs` dan secara deterministik menemukan seluruh RVA baru:
```json
{
  "name": "il2cpp_scan_taskbarhero_offsets",
  "arguments": {
    "dump_path": "reverse/taskbarhero/tools/dump.cs",
    "pid": 23171
  }
}
```
Tool ini secara otomatis melacak:
* **Base Health Class** via `SpriteSlider HpBar` & RVA Base Damage (`pj.gsi` / `ph.gsf`)
* **Hero Subclass** & RVA Godmode Hero (`pf.gsi` / `pd.gsf`)
* **Stat Multiplier Extension** via `StatType` parameter (`zo.haz` / `pn.hal`)
* **AoE Physical Radius & Hitbox** via `DamageDetectRadiusRawValue` (`bec.nax` / `bdl.muj`)
* Menghitung **Runtime Address** (`Base + RVA`) untuk langsung dipatch via `memory_write_bytes`.

### 3. Alokasi Code Cave & Detour Hooking (`memory_allocate` & `memory_write_bytes`)
Untuk memasang detour hook (godmode hero, damage multiplier, AoE radius) langsung dari MCP tanpa membutuhkan injector eksternal di Windows & Linux:

1. **Alokasikan RWX Code Cave**:
```json
{
  "name": "memory_allocate",
  "arguments": {
    "pid": 23171,
    "size": 4096,
    "protection": "rwx"
  }
}
```
2. **Tulis Detour Hook Payload & Konstanta ke Cave Memory**:
```json
{
  "name": "memory_write_bytes",
  "arguments": {
    "pid": 23171,
    "address": "0x1b40080",
    "bytes_hex": "50 48 8B 41 30 48 85 C0 74 0C 80 B8 00 01 00 00 00 74 03 0F 57 C9 F3 0F 59 0D 0C 00 00 00 58 48 89 5C 24 08 57 E9 ...",
    "confirm_write": true
  }
}
```
3. **Patch Prologue Target Method dengan Jump ke Cave**:
```json
{
  "name": "memory_write_bytes",
  "arguments": {
    "pid": 23171,
    "address": "0x7ffb12345678",
    "bytes_hex": "E9 4B 44 E7 FF 90",
    "confirm_write": true
  }
}
```
4. *(Opsional)* Jika perlu mengubah proteksi memori kode game sebelum/sesudah patching:
```json
{
  "name": "memory_protect",
  "arguments": {
    "pid": 23171,
    "address": "0x7ffb12345678",
    "size": 16,
    "protection": "rwx"
  }
}
```
