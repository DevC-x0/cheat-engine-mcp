use serde::{Deserialize, Serialize};
use std::io::BufRead;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MethodFinding {
    pub class_name: String,
    pub method_name: String,
    pub signature: String,
    pub rva: String,
    pub rva_int: u64,
    pub slot: Option<u32>,
    pub line_number: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskbarHeroOffsets {
    pub base_health_class: Option<String>,
    pub hero_health_class: Option<String>,
    pub godmode_hero: Option<MethodFinding>,
    pub godmode_base: Option<MethodFinding>,
    pub stat_multiplier: Option<MethodFinding>,
    pub aoe_radius: Option<MethodFinding>,
    pub unit_hero_offset: Option<String>,
}

#[derive(Default)]
struct PendingMeta {
    rva: Option<String>,
    rva_int: u64,
    slot: Option<u32>,
    is_extension: bool,
}

/// Parses dump.cs line-by-line to find TaskbarHero offsets using deterministic structural anchors.
pub fn scan_taskbarhero_dump<R: BufRead>(reader: R) -> Result<TaskbarHeroOffsets, String> {
    let mut offsets = TaskbarHeroOffsets::default();

    let mut current_class = String::new();
    let mut current_base = String::new();
    let mut current_class_has_hero_field = false;
    let mut current_class_has_hpbar_field = false;
    let mut current_class_has_radius_field = false;
    let mut current_class_methods: Vec<(MethodFinding, bool)> = Vec::new();

    struct ClassSummary {
        name: String,
        base: String,
        has_hero_field: bool,
        has_hpbar_field: bool,
        has_radius_field: bool,
        methods: Vec<(MethodFinding, bool)>,
    }

    let mut classes: Vec<ClassSummary> = Vec::new();
    let mut pending_meta = PendingMeta::default();

    for (line_idx, line_res) in reader.lines().enumerate() {
        let line = line_res.map_err(|e| format!("Failed to read line {}: {}", line_idx + 1, e))?;
        let trimmed = line.trim();

        if trimmed == "[Extension]" {
            pending_meta.is_extension = true;
            continue;
        }

        if trimmed.starts_with("// RVA:") {
            if let Some(rva_str) = parse_rva_hex(trimmed) {
                let rva_val = u64::from_str_radix(
                    rva_str.trim_start_matches("0x").trim_start_matches("0X"),
                    16,
                )
                .unwrap_or(0);
                pending_meta.rva = Some(rva_str);
                pending_meta.rva_int = rva_val;
            }
            if let Some(slot_val) = parse_slot(trimmed) {
                pending_meta.slot = Some(slot_val);
            }
            continue;
        }

        // Check for class start
        if trimmed.starts_with("public ")
            || trimmed.starts_with("private ")
            || trimmed.starts_with("protected ")
            || trimmed.starts_with("internal ")
        {
            if trimmed.contains("class ") {
                if !current_class.is_empty() {
                    classes.push(ClassSummary {
                        name: std::mem::take(&mut current_class),
                        base: std::mem::take(&mut current_base),
                        has_hero_field: current_class_has_hero_field,
                        has_hpbar_field: current_class_has_hpbar_field,
                        has_radius_field: current_class_has_radius_field,
                        methods: std::mem::take(&mut current_class_methods),
                    });
                    current_class_has_hero_field = false;
                    current_class_has_hpbar_field = false;
                    current_class_has_radius_field = false;
                }

                if let Some((c_name, b_name)) = parse_class_and_base(trimmed) {
                    current_class = c_name;
                    current_base = b_name;
                }
                pending_meta = PendingMeta::default();
                continue;
            }
        }

        // Check field anchors in current class
        if trimmed.contains("SpriteSlider HpBar;") || trimmed.contains("SpriteSlider HpBar") {
            current_class_has_hpbar_field = true;
        }
        if trimmed.contains("DamageDetectRadiusRawValue;")
            || trimmed.contains("DamageDetectRadiusRawValue")
        {
            current_class_has_radius_field = true;
        }
        if trimmed.contains("Hero ")
            && (trimmed.contains("private ")
                || trimmed.contains("public ")
                || trimmed.contains("protected "))
        {
            current_class_has_hero_field = true;
        }

        // Check if line is a method declaration
        if let Some(rva) = pending_meta.rva.take() {
            if trimmed.contains('(') && trimmed.contains(')') && !trimmed.starts_with("//") {
                let method_name = parse_method_name(trimmed);
                let finding = MethodFinding {
                    class_name: current_class.clone(),
                    method_name,
                    signature: trimmed.to_string(),
                    rva,
                    rva_int: pending_meta.rva_int,
                    slot: pending_meta.slot,
                    line_number: line_idx + 1,
                };
                let is_ext = pending_meta.is_extension;
                current_class_methods.push((finding, is_ext));
            }
            pending_meta = PendingMeta::default();
        }
    }

    if !current_class.is_empty() {
        classes.push(ClassSummary {
            name: current_class,
            base: current_base,
            has_hero_field: current_class_has_hero_field,
            has_hpbar_field: current_class_has_hpbar_field,
            has_radius_field: current_class_has_radius_field,
            methods: current_class_methods,
        });
    }

    // Now resolve the anchors across collected classes:

    // 1. Find Base Health Class (contains HpBar)
    let base_health = classes.iter().find(|c| c.has_hpbar_field);
    let base_health_name = base_health.map(|c| c.name.clone());
    offsets.base_health_class = base_health_name.clone();

    if let Some(base_cls) = base_health {
        for (m, _) in &base_cls.methods {
            if m.signature.contains("(float a, Unit b)") {
                offsets.godmode_base = Some(m.clone());
                break;
            }
        }
    }

    // 2. Find Hero Subclass (inherits from BaseHealthClass and has Hero field)
    if let Some(base_name) = &base_health_name {
        let hero_subclass = classes
            .iter()
            .find(|c| c.base == *base_name && c.has_hero_field);
        if let Some(hero_cls) = hero_subclass {
            offsets.hero_health_class = Some(hero_cls.name.clone());
            for (m, _) in &hero_cls.methods {
                if m.signature.contains("(float a, Unit b)") {
                    offsets.godmode_hero = Some(m.clone());
                    break;
                }
            }
        }
    }

    // 3. Find Stat Multiplier (Extension method returning float taking StatType)
    for c in &classes {
        for (m, is_ext) in &c.methods {
            if *is_ext
                && m.signature.contains("float ")
                && (m.signature.contains("StatType b)")
                    || m.signature.contains("StatType b,")
                    || m.signature.contains("StatType "))
                && !m.signature.contains("int a, StatType b")
                && !m.signature.contains("string ")
                && !m.signature.contains("double ")
            {
                offsets.stat_multiplier = Some(m.clone());
                break;
            }
        }
        if offsets.stat_multiplier.is_some() {
            break;
        }
    }

    // 4. Find AoE Physical Radius (in class with DamageDetectRadiusRawValue)
    let aoe_cls = classes.iter().find(|c| c.has_radius_field);
    if let Some(cls) = aoe_cls {
        for (m, _) in &cls.methods {
            if (m.signature.contains("Vector3 a, Unit b")
                || m.signature.contains("Func<DamageInfo>"))
                && (m.signature.contains("Slot: 10")
                    || m.signature.contains("Slot: 5")
                    || m.signature.contains("virtual void"))
            {
                offsets.aoe_radius = Some(m.clone());
                break;
            }
        }
    }

    offsets.unit_hero_offset = Some("0x100".to_string());
    Ok(offsets)
}

fn parse_rva_hex(line: &str) -> Option<String> {
    let after_rva = line.strip_prefix("//")?.trim().strip_prefix("RVA:")?.trim();
    let rva_token = after_rva.split_whitespace().next()?;
    if rva_token.starts_with("0x") || rva_token.starts_with("0X") {
        Some(rva_token.to_string())
    } else {
        None
    }
}

fn parse_slot(line: &str) -> Option<u32> {
    if let Some(slot_idx) = line.find("Slot:") {
        let after_slot = &line[slot_idx + 5..].trim();
        let token = after_slot.split_whitespace().next()?;
        token.parse::<u32>().ok()
    } else {
        None
    }
}

fn parse_class_and_base(line: &str) -> Option<(String, String)> {
    let clean = if let Some(idx) = line.find("//") {
        &line[..idx]
    } else {
        line
    };
    let parts: Vec<&str> = clean.split_whitespace().collect();
    let class_idx = parts.iter().position(|&p| p == "class")?;
    if class_idx + 1 >= parts.len() {
        return None;
    }
    let class_name = parts[class_idx + 1].trim().to_string();

    let mut base_name = String::new();
    if let Some(colon_idx) = parts.iter().position(|&p| p == ":") {
        if colon_idx + 1 < parts.len() {
            base_name = parts[colon_idx + 1].trim().to_string();
        }
    }

    Some((class_name, base_name))
}

fn parse_method_name(sig: &str) -> String {
    let clean = if let Some(idx) = sig.find("//") {
        &sig[..idx]
    } else {
        sig
    };
    if let Some(paren_idx) = clean.find('(') {
        let before_paren = &clean[..paren_idx].trim();
        if let Some(space_idx) = before_paren.rfind(' ') {
            return before_paren[space_idx + 1..].trim().to_string();
        }
    }
    sig.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_synthetic_taskbarhero_dump() {
        let snippet = r#"
// Namespace:
public class pj : MonoBehaviour // TypeDefIndex: 630
{
    // Fields
    public SpriteSlider HpBar; // 0x20
    public Action<float> OnHpChange; // 0x28

    // Methods
    // RVA: 0xCABC30 Offset: 0xCAA230 VA: 0x180CABC30 Slot: 9
    public virtual void gsi(float a, Unit b) { }
}

public class pf : pj // TypeDefIndex: 620
{
    // Fields
    private Hero bdeg; // 0x58

    // Methods
    // RVA: 0xCAA8B0 Offset: 0xCA8EB0 VA: 0x180CAA8B0 Slot: 9
    public override void gsi(float a, Unit b) { }
}

public static class pp
{
    [Extension]
    // RVA: 0xCB8A90 Offset: 0xCB7090 VA: 0x180CB8A90
    public static float haz(zo a, StatType b) { }
}

public class bec : MonoBehaviour
{
    // Fields
    [FormerlySerializedAs("DamageDetectRadius")]
    public int DamageDetectRadiusRawValue; // 0x68

    // Methods
    // RVA: 0xB6D950 Offset: 0xB6BF50 VA: 0x180B6D950 Slot: 10
    public virtual void nax(Vector3 a, Unit b, float c = 0, bool d = False, Func<DamageInfo> e, Action f) { }
}
"#;

        let cursor = Cursor::new(snippet);
        let offsets = scan_taskbarhero_dump(cursor).expect("Should scan successfully");

        assert_eq!(offsets.base_health_class.as_deref(), Some("pj"));
        assert_eq!(offsets.hero_health_class.as_deref(), Some("pf"));

        let base_dmg = offsets.godmode_base.expect("Should find godmode_base");
        assert_eq!(base_dmg.rva, "0xCABC30");
        assert_eq!(base_dmg.method_name, "gsi");

        let hero_dmg = offsets.godmode_hero.expect("Should find godmode_hero");
        assert_eq!(hero_dmg.rva, "0xCAA8B0");
        assert_eq!(hero_dmg.method_name, "gsi");

        let stat_calc = offsets
            .stat_multiplier
            .expect("Should find stat_multiplier");
        assert_eq!(stat_calc.rva, "0xCB8A90");
        assert_eq!(stat_calc.method_name, "haz");

        let aoe = offsets.aoe_radius.expect("Should find aoe_radius");
        assert_eq!(aoe.rva, "0xB6D950");
        assert_eq!(aoe.method_name, "nax");
    }

    #[test]
    fn test_real_taskbarhero_dump_1_01_05() {
        let path = "/home/cahya/2026/tbh-injector/reverse/TaskbarHero_1.01.05/dump.cs";
        if !std::path::Path::new(path).exists() {
            return;
        }
        let file = std::fs::File::open(path).expect("Open 1.01.05 dump.cs");
        let reader = std::io::BufReader::new(file);
        let offsets = scan_taskbarhero_dump(reader).expect("Scan real 1.01.05 dump");

        assert_eq!(offsets.base_health_class.as_deref(), Some("pj"));
        assert_eq!(offsets.hero_health_class.as_deref(), Some("pf"));

        let hero_dmg = offsets.godmode_hero.expect("Hero damage in 1.01.05");
        assert_eq!(hero_dmg.rva, "0xCAA8B0");

        let base_dmg = offsets.godmode_base.expect("Base damage in 1.01.05");
        assert_eq!(base_dmg.rva, "0xCABC30");

        let stat_calc = offsets.stat_multiplier.expect("Stat calc in 1.01.05");
        assert_eq!(stat_calc.rva, "0xCB8A90");

        let aoe = offsets.aoe_radius.expect("AoE radius in 1.01.05");
        assert_eq!(aoe.rva, "0xB6D950");
    }

    #[test]
    fn test_real_taskbarhero_dump_old_tools() {
        let path = "reverse/taskbarhero/tools/dump.cs";
        if !std::path::Path::new(path).exists() {
            return;
        }
        let file = std::fs::File::open(path).expect("Open old dump.cs");
        let reader = std::io::BufReader::new(file);
        let offsets = scan_taskbarhero_dump(reader).expect("Scan old dump");

        assert_eq!(offsets.base_health_class.as_deref(), Some("ph"));
        assert_eq!(offsets.hero_health_class.as_deref(), Some("pd"));

        let hero_dmg = offsets.godmode_hero.expect("Hero damage in old dump");
        assert_eq!(hero_dmg.rva, "0xC3A860");

        let base_dmg = offsets.godmode_base.expect("Base damage in old dump");
        assert_eq!(base_dmg.rva, "0xC3B810");

        let stat_calc = offsets.stat_multiplier.expect("Stat calc in old dump");
        assert_eq!(stat_calc.rva, "0xC443C0");

        let aoe = offsets.aoe_radius.expect("AoE radius in old dump");
        assert_eq!(aoe.rva, "0xB132B0");
    }
}
