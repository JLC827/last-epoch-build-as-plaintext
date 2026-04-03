use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use serde_json::Value;
use anyhow::Result;

pub struct Resolver {
    translations: HashMap<String, String>,
    affix_map: HashMap<String, AffixData>,
    ability_map: HashMap<String, AbilityData>,
    item_map: HashMap<String, String>, // ID -> NameKey
    item_data_map: HashMap<String, ItemData>, // ID -> Implicits
    base_item_map: HashMap<(u32, u32), ItemData>, // (BaseType, SubType) -> Implicits
    unique_map: HashMap<String, String>,
    unique_data_map: HashMap<String, UniqueData>,
    property_map: HashMap<u32, String>,
    player_property_map: HashMap<u32, String>,
    base_type_name_map: HashMap<u32, String>,
}

#[derive(Debug)]
struct UniqueData {
    base_type_id: u32,
    sub_type_id: u32,
    mods: Vec<UniqueMod>,
    tooltip_descriptions: Vec<String>,
}

#[derive(Debug)]
struct UniqueMod {
    property_id: u32,
    value: f32,
    max_value: f32,
    roll_id: usize,
    can_roll: bool,
}

#[derive(Debug)]
struct AffixData {
    display_name_key: String,
    properties: Vec<String>,
    tiers: Vec<TierData>,
}

#[derive(Debug)]
struct TierData {
    rolls: Vec<RollData>,
}

#[derive(Debug)]
struct RollData {
    min: f32,
    max: f32,
}

#[derive(Debug)]
struct AbilityData {
    name: Option<String>,
    name_key: Option<String>,
    description_key: Option<String>,
}

#[derive(Debug, Clone)]
struct ItemData {
    base_type_id: u32,
    implicits: Vec<ImplicitData>,
}

#[derive(Debug, Clone)]
struct ImplicitData {
    property_id: u32,    tags: u32,    value: f32,
    max_value: f32,
}

impl Resolver {
    pub fn new() -> Result<Self> {
        let mut resolver = Resolver {
            translations: HashMap::new(),
            affix_map: HashMap::new(),
            ability_map: HashMap::new(),
            item_map: HashMap::new(),
            item_data_map: HashMap::new(),
            base_item_map: HashMap::new(),
            unique_map: HashMap::new(),
            unique_data_map: HashMap::new(),
            property_map: Self::init_property_map(),
            player_property_map: HashMap::new(),
            base_type_name_map: HashMap::new(),
        };

        resolver.load_translations("translations.json")?;
        resolver.load_properties("core_db.json")?;
        resolver.load_affixes("item_db.json")?;
        resolver.load_items("item_db.json")?;
        resolver.load_uniques("item_db.json")?;
        resolver.load_abilities("le_abilities.json")?;

        Ok(resolver)
    }

    fn load_properties(&mut self, path: &str) -> Result<()> {
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            let json: Value = serde_json::from_reader(reader)?;
            
            if let Some(list) = json.get("propertyList").and_then(|v| v.as_array()) {
                for item in list {
                    if let Some(id) = item.get("property").and_then(|v| v.as_u64()) {
                        if let Some(key) = item.get("propertyNameKey").and_then(|v| v.as_str()) {
                            if let Some(trans) = self.translations.get(key) {
                                self.property_map.insert(id as u32, trans.clone());
                            } else {
                                self.property_map.insert(id as u32, key.to_string());
                            }
                        }
                    }
                }
            }

            if let Some(list) = json.get("playerPropertyList").and_then(|v| v.as_array()) {
                for (i, item) in list.iter().enumerate() {
                     if let Some(key) = item.get("propertyNameKey").and_then(|v| v.as_str()) {
                        let id = i as u32;
                        if let Some(trans) = self.translations.get(key) {
                            self.player_property_map.insert(id, trans.clone());
                        } else {
                            self.player_property_map.insert(id, key.to_string());
                        }
                     }
                }
            }
        }
        Ok(())
    }

    fn init_property_map() -> HashMap<u32, String> {
        let mut m = HashMap::new();
        m.insert(0, "Damage".to_string());
        m.insert(1, "Ailment Chance".to_string());
        m.insert(2, "Attack Speed".to_string());
        m.insert(3, "Cast Speed".to_string());
        m.insert(4, "Critical Chance".to_string());
        m.insert(5, "Critical Multiplier".to_string());
        m.insert(6, "Damage Taken".to_string());
        m.insert(7, "Health".to_string());
        m.insert(8, "Mana".to_string());
        m.insert(9, "Movespeed".to_string());
        m.insert(10, "Armor".to_string());
        m.insert(11, "Dodge Rating".to_string());
        m.insert(12, "Stun Avoidance".to_string());
        m.insert(13, "Fire Resistance".to_string());
        m.insert(14, "Cold Resistance".to_string());
        m.insert(15, "Lightning Resistance".to_string());
        m.insert(16, "Ward Retention".to_string());
        m.insert(17, "Health Regen".to_string());
        m.insert(18, "Mana Regen".to_string());
        m.insert(19, "Strength".to_string());
        m.insert(20, "Vitality".to_string());
        m.insert(21, "Intelligence".to_string());
        m.insert(22, "Dexterity".to_string());
        m.insert(23, "Attunement".to_string());
        m.insert(24, "Mana Before Health Percent".to_string());
        m.insert(25, "Channel Cost".to_string());
        m.insert(26, "Void Resistance".to_string());
        m.insert(27, "Necrotic Resistance".to_string());
        m.insert(28, "Poison Resistance".to_string());
        m.insert(29, "Block Chance".to_string());
        m.insert(30, "All Resistances".to_string());
        m.insert(31, "Damage Taken As Physical".to_string());
        m.insert(32, "Damage Taken As Fire".to_string());
        m.insert(33, "Damage Taken As Cold".to_string());
        m.insert(34, "Damage Taken As Lightning".to_string());
        m.insert(35, "Damage Taken As Necrotic".to_string());
        m.insert(36, "Damage Taken As Void".to_string());
        m.insert(37, "Damage Taken As Poison".to_string());
        m.insert(38, "Health Gain".to_string());
        m.insert(39, "Ward Gain".to_string());
        m.insert(40, "Mana Gain".to_string());
        m.insert(41, "Adaptive Spell Damage".to_string());
        m.insert(42, "Increased Ailment Duration".to_string());
        m.insert(43, "Increased Ailment Effect".to_string());
        m.insert(44, "Increased Healing".to_string());
        m.insert(45, "Increased Stun Chance".to_string());
        m.insert(46, "All Attributes".to_string());
        m.insert(47, "Increased Potion Drop Rate".to_string());
        m.insert(48, "Potion Health".to_string());
        m.insert(49, "Potion Slots".to_string());
        m.insert(50, "Haste On Hit Chance".to_string());
        m.insert(51, "Health Leech".to_string());
        m.insert(52, "Elemental Resistance".to_string());
        m.insert(53, "Block Effectiveness".to_string());
        m.insert(54, "None".to_string());
        m.insert(55, "Increased Stun Immunity Duration".to_string());
        m.insert(56, "Stun Immunity".to_string());
        m.insert(57, "Mana Drain".to_string());
        m.insert(58, "Ability Property".to_string());
        m.insert(59, "Penetration".to_string());
        m.insert(60, "Current Health Drain".to_string());
        m.insert(61, "Maximum Companions".to_string());
        m.insert(62, "Glancing Blow Chance".to_string());
        m.insert(63, "Cull Percent From Passives".to_string());
        m.insert(64, "Physical Resistance".to_string());
        m.insert(65, "Cull Percent From Weapon".to_string());
        m.insert(66, "Mana Cost".to_string());
        m.insert(67, "Freeze Rate Multiplier".to_string());
        m.insert(68, "Increased Chance To Be Frozen".to_string());
        m.insert(69, "Mana Efficiency".to_string());
        m.insert(70, "Increased Cooldown Recovery Speed".to_string());
        m.insert(71, "Received Stun Duration".to_string());
        m.insert(72, "Negative Physical Resistance".to_string());
        m.insert(73, "Chill Retaliation Chance".to_string());
        m.insert(74, "Slow Retaliation Chance".to_string());
        m.insert(75, "Endurance".to_string());
        m.insert(76, "Endurance Threshold".to_string());
        m.insert(77, "Negative Armour".to_string());
        m.insert(78, "Negative Fire Resistance".to_string());
        m.insert(79, "Negative Cold Resistance".to_string());
        m.insert(80, "Negative Lightning Resistance".to_string());
        m.insert(81, "Negative Void Resistance".to_string());
        m.insert(82, "Negative Necrotic Resistance".to_string());
        m.insert(83, "Negative Poison Resistance".to_string());
        m.insert(84, "Negative Elemental Resistance".to_string());
        m.insert(85, "Thorns".to_string());
        m.insert(86, "Percent Reflect".to_string());
        m.insert(87, "Shock Retaliation Chance".to_string());
        m.insert(88, "Level Of Skills".to_string());
        m.insert(89, "Crit Avoidance".to_string());
        m.insert(90, "Potion Health Converted To Ward".to_string());
        m.insert(91, "Ward On Potion Use".to_string());
        m.insert(92, "Ward Regen".to_string());
        m.insert(93, "Overkill Leech".to_string());
        m.insert(94, "Mana Before Ward Percent".to_string());
        m.insert(95, "Increased Stun Duration".to_string());
        m.insert(96, "Maximum Health Gained As Endurance Threshold".to_string());
        m.insert(97, "Chance To Gain 30 Ward When Hit".to_string());
        m.insert(98, "Player Property".to_string());
        m.insert(99, "Mana Spent Gained As Ward".to_string());
        m.insert(100, "Ailment Conversion".to_string());        m.insert(101, "PerceivedUnimportanceModifier".to_string());
        m.insert(102, "IncreasedLeechRate".to_string());
        m.insert(103, "MoreFreezeRatePerStackOfChill".to_string());
        m.insert(104, "IncreasedDropRate".to_string());
        m.insert(105, "IncreasedExperience".to_string());
        m.insert(106, "PhysicalAndVoidResistance".to_string());
        m.insert(107, "NecroticAndPoisonResistance".to_string());
        m.insert(108, "DamageTakenBuff".to_string());
        m.insert(109, "IncreasedChanceToBeStunned".to_string());
        m.insert(110, "DamageTakenFromNearbyEnemies".to_string());
        m.insert(111, "BlockChanceAgainstDistantEnemies".to_string());
        m.insert(112, "ChanceToBeCrit".to_string());
        m.insert(113, "DamageTakenWhileMoving".to_string());
        m.insert(114, "ReducedBonusDamageTakenFromCrits".to_string());
        m.insert(115, "DamagePerStackOfAilment".to_string());
        m.insert(116, "IncreasedAreaForAreaSkills".to_string());
        m.insert(117, "GlobalConditionalDamage".to_string());
        m.insert(118, "ArmourMitigationAppliesToDamageOverTime".to_string());
        m.insert(119, "WardDecayThreshold".to_string());
        m.insert(120, "EffectOfAilmentOnYou".to_string());
        m.insert(121, "ParryChance".to_string());
        m.insert(122, "CircleOfFortuneLensEffect".to_string());
        m.insert(123, "TrackerProperty".to_string());
        m.insert(124, "UnimportanceModifier".to_string());        m
    }

    fn load_translations(&mut self, path: &str) -> Result<()> {
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            let json: Value = serde_json::from_reader(reader)?;
            
            if let Some(obj) = json.as_object() {
                if let Some(full) = obj.get("full").and_then(|v| v.as_object()) {
                    for (k, v) in full {
                        if let Some(s) = v.as_str() {
                            self.translations.insert(k.clone(), s.to_string());
                        }
                    }
                }
                for (k, v) in obj {
                    if let Some(s) = v.as_str() {
                        self.translations.insert(k.clone(), s.to_string());
                    }
                }
            }
        }
        Ok(())
    }

    fn load_items(&mut self, path: &str) -> Result<()> {
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            let json: Value = serde_json::from_reader(reader)?;
            
            if let Some(item_list) = json.get("itemList").and_then(|v| v.as_object()) {
                let categories = ["equippable", "nonEquippable"];
                for cat in categories {
                    if let Some(cat_obj) = item_list.get(cat).and_then(|v| v.as_object()) {
                        for (base_type_key, base_type_val) in cat_obj {
                            let base_type_id = base_type_key.parse::<u32>().unwrap_or(0);
                            
                            // Extract base type name map
                            if let Some(display_name_key) = base_type_val.get("displayNameKey").and_then(|v| v.as_str()) {
                                if let Some(translated_name) = self.translations.get(display_name_key) {
                                    self.base_type_name_map.insert(base_type_id, translated_name.clone());
                                }
                            }
                            
                            if let Some(sub_items) = base_type_val.get("subItems").and_then(|v| v.as_array()) {
                                for sub_item_val in sub_items {
                                    if let Some(id) = sub_item_val.get("id").and_then(|v| v.as_str()) {
                                        if let Some(key) = sub_item_val.get("displayNameKey").and_then(|v| v.as_str()) {
                                            self.item_map.insert(id.to_string(), key.to_string());
                                        }
                                        let mut implicits = Vec::new();
                                        if let Some(impl_arr) = sub_item_val.get("implicits").and_then(|v| v.as_array()) {
                                            for imp in impl_arr {
                                                let prop = imp.get("property").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                                let val = imp.get("implicitValue").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                                                let max_val = imp.get("implicitMaxValue").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                                                let tag_val = imp.get("tags").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                                implicits.push(ImplicitData { property_id: prop, tags: tag_val, value: val, max_value: max_val });
                                            }
                                        }
                                        let item_data = ItemData { implicits, base_type_id };
                                        self.item_data_map.insert(id.to_string(), item_data.clone());
                                        
                                        let sub_type_id = id.parse::<u32>().unwrap_or(0);
                                        self.base_item_map.insert((base_type_id, sub_type_id), item_data);
                                    }
                                }
                            }
                            if let Some(sub_items) = base_type_val.get("subItems").and_then(|v| v.as_object()) {
                                for (_sub_id, sub_item_val) in sub_items {
                                    if let Some(id) = sub_item_val.get("id").and_then(|v| v.as_str()) {
                                        if let Some(key) = sub_item_val.get("displayNameKey").and_then(|v| v.as_str()) {
                                            self.item_map.insert(id.to_string(), key.to_string());
                                        }
                                        let mut implicits = Vec::new();
                                        if let Some(impl_arr) = sub_item_val.get("implicits").and_then(|v| v.as_array()) {
                                            for imp in impl_arr {
                                                let prop = imp.get("property").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                                let val = imp.get("implicitValue").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                                                let max_val = imp.get("implicitMaxValue").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                                                let tag_val = imp.get("tags").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                                implicits.push(ImplicitData { property_id: prop, tags: tag_val, value: val, max_value: max_val });
                                            }
                                        }
                                        let item_data = ItemData { implicits, base_type_id };
                                        self.item_data_map.insert(id.to_string(), item_data.clone());
                                        
                                        let sub_type_id = id.parse::<u32>().unwrap_or(0);
                                        self.base_item_map.insert((base_type_id, sub_type_id), item_data);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn load_uniques(&mut self, path: &str) -> Result<()> {
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            let json: Value = serde_json::from_reader(reader)?;
            
            if let Some(unique_list) = json.get("uniqueList").and_then(|v| v.get("uniques")).and_then(|v| v.as_object()) {
                for (_key, value) in unique_list {
                    if let Some(id) = value.get("id").and_then(|v| v.as_str()) {
                        if let Some(key) = value.get("displayNameKey").and_then(|v| v.as_str()) {
                            self.unique_map.insert(id.to_string(), key.to_string());
                        }
                        
                        let base_type_id = value.get("baseTypeId").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                        let sub_type_id = value.get("subTypeId").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                        
                        let mut mods = Vec::new();
                        if let Some(mod_arr) = value.get("mods").and_then(|v| v.as_array()) {
                            for m in mod_arr {
                                let prop = m.get("property").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                let val = m.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                                let max_val = m.get("maxValue").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                                let roll_id = m.get("rollId").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                                let can_roll = m.get("canRoll").and_then(|v| v.as_u64()).unwrap_or(0) == 1;
                                
                                mods.push(UniqueMod {
                                    property_id: prop,
                                    value: val,
                                    max_value: max_val,
                                    roll_id,
                                    can_roll,
                                });
                            }
                        }

                        let mut tooltip_descriptions = Vec::new();
                        if let Some(desc_arr) = value.get("tooltipDescriptions").and_then(|v| v.as_array()) {
                            for d in desc_arr {
                                if let Some(key) = d.get("descriptionKey").and_then(|v| v.as_str()) {
                                    tooltip_descriptions.push(key.to_string());
                                }
                            }
                        }
                        
                        self.unique_data_map.insert(id.to_string(), UniqueData {
                            base_type_id,
                            sub_type_id,
                            mods,
                            tooltip_descriptions,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn load_affixes(&mut self, path: &str) -> Result<()> {
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            let json: Value = serde_json::from_reader(reader)?;
            
            if let Some(affix_list) = json.get("affixList").and_then(|v| v.as_object()) {
                let mut process_affixes = |affixes: &serde_json::Map<String, Value>| {
                    for (_key, value) in affixes {
                        if let Some(obj) = value.as_object() {
                            if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                                let display_name_key = obj.get("affixDisplayNameKey")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                
                                let mut properties = Vec::new();
                                if let Some(props) = obj.get("affixProperties").and_then(|v| v.as_array()) {
                                    for p in props {
                                        let key = p.get("modDisplayNameKey")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();
                                        properties.push(key);
                                    }
                                }

                                let mut tiers = Vec::new();
                                if let Some(tier_arr) = obj.get("tiers").and_then(|v| v.as_array()) {
                                    for t in tier_arr {
                                        let mut rolls = Vec::new();
                                        if let Some(roll_arr) = t.get("rolls").and_then(|v| v.as_array()) {
                                            for r in roll_arr {
                                                let min = r.get("min").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                                                let max = r.get("max").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                                                rolls.push(RollData { min, max });
                                            }
                                        }
                                        tiers.push(TierData { rolls });
                                    }
                                }
                                
                                self.affix_map.insert(id.to_string(), AffixData {
                                    display_name_key,
                                    properties,
                                    tiers,
                                });
                            }
                        }
                    }
                };

                if let Some(single) = affix_list.get("singleAffixes").and_then(|v| v.as_object()) {
                    process_affixes(single);
                }
                if let Some(multi) = affix_list.get("multiAffixes").and_then(|v| v.as_object()) {
                    process_affixes(multi);
                }
            }
        }
        Ok(())
    }

    fn load_abilities(&mut self, path: &str) -> Result<()> {
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            let json: Value = serde_json::from_reader(reader)?;
            
            if let Some(obj) = json.as_object() {
                for (id, value) in obj {
                    if let Some(ability_obj) = value.as_object() {
                        let name = ability_obj.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let name_key = ability_obj.get("nameKey").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let description_key = ability_obj.get("descriptionKey").and_then(|v| v.as_str()).map(|s| s.to_string());
                        
                        self.ability_map.insert(id.clone(), AbilityData {
                            name,
                            name_key,
                            description_key,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get_affix_name(&self, id: &str) -> String {
        if let Some(data) = self.affix_map.get(id) {
            if let Some(trans) = self.translations.get(&data.display_name_key) {
                return trans.clone();
            }
            return data.display_name_key.clone();
        }
        id.to_string()
    }

    pub fn get_affix_detail(&self, id: &str, tier: usize, roll: f32) -> String {
        if let Some(data) = self.affix_map.get(id) {
            let tier_idx = if tier > 0 { tier - 1 } else { 0 };
            
            if let Some(tier_data) = data.tiers.get(tier_idx) {
                let mut parts = Vec::new();
                for (i, roll_data) in tier_data.rolls.iter().enumerate() {
                    let default_key = String::new();
                    let prop_key = data.properties.get(i).unwrap_or(&default_key);
                    let prop_name = self.translations.get(prop_key).map(|s| s.as_str()).unwrap_or("Unknown Property");
                    
                    let val = roll_data.min + (roll_data.max - roll_data.min) * (roll / 255.0);
                    
                    let (val_s, min_s, max_s) = if roll_data.min.abs() < 2.0 && roll_data.min != 0.0 {
                        (
                            format!("{:.0}%", val * 100.0),
                            format!("{:.0}%", roll_data.min * 100.0),
                            format!("{:.0}%", roll_data.max * 100.0)
                        )
                    } else {
                        (
                            format!("{:.0}", val),
                            format!("{:.0}", roll_data.min),
                            format!("{:.0}", roll_data.max)
                        )
                    };

                    if roll_data.min == roll_data.max {
                         parts.push(format!("[T{}] +{} {}", tier, val_s, prop_name));
                    } else {
                         parts.push(format!("[T{}] +{} ({}-{}) {}", tier, val_s, min_s, max_s, prop_name));
                    }
                }
                return parts.join(", ");
            }
        }
        "".to_string()
    }

    pub fn get_unique_detail(&self, id: &str, ir: &[u8]) -> Vec<String> {
        let mut details = Vec::new();
        
        if let Some(data) = self.unique_data_map.get(id) {
            // Base Implicits
            if let Some(base_data) = self.base_item_map.get(&(data.base_type_id, data.sub_type_id)) {
                for (i, imp) in base_data.implicits.iter().enumerate() {
                    let prop_name = if imp.property_id == 98 {
                        self.player_property_map.get(&imp.tags).map(|s| s.as_str()).unwrap_or("Unknown Player Property")
                    } else {
                        self.property_map.get(&imp.property_id).map(|s| s.as_str()).unwrap_or("Unknown Property")
                    };
                    let RollData { min, max } = RollData { min: imp.value, max: imp.max_value };
                    
                    let (min_s, max_s) = if min.abs() < 2.0 && min != 0.0 {
                        (format!("{:.0}%", min * 100.0), format!("{:.0}%", max * 100.0))
                    } else {
                        (format!("{:.0}", min), format!("{:.0}", max))
                    };

                    if min == max {
                        details.push(format!("{} {}", min_s, prop_name));
                    } else {
                        details.push(format!("{}-{} {}", min_s, max_s, prop_name));
                    }
                }
            }

            // Unique Mods
            for m in &data.mods {
                let prop_name = self.property_map.get(&m.property_id).map(|s| s.as_str()).unwrap_or("Unknown");
                
                if !m.can_roll {
                    let val = m.value;
                    let val_s = if val.abs() < 2.0 && val != 0.0 {
                        format!("{:.0}%", val * 100.0)
                    } else {
                        format!("{:.0}", val)
                    };
                    details.push(format!("+{} {}", val_s, prop_name));
                } else {
                    let roll_val = if m.roll_id < ir.len() {
                        ir[m.roll_id] as f32
                    } else {
                        0.0 // Default to min if missing
                    };
                    
                    let val = m.value + (m.max_value - m.value) * (roll_val / 255.0);
                    
                    let (val_s, min_s, max_s) = if m.value.abs() < 2.0 && m.value != 0.0 {
                        (
                            format!("{:.0}%", val * 100.0),
                            format!("{:.0}%", m.value * 100.0),
                            format!("{:.0}%", m.max_value * 100.0)
                        )
                    } else {
                        (
                            format!("{:.0}", val),
                            format!("{:.0}", m.value),
                            format!("{:.0}", m.max_value)
                        )
                    };

                    if m.value == m.max_value {
                         details.push(format!("+{} {}", val_s, prop_name));
                    } else {
                         details.push(format!("+{} ({}-{}) {}", val_s, min_s, max_s, prop_name));
                    }
                }
            }

            // Tooltip Descriptions
            for key in &data.tooltip_descriptions {
                if let Some(trans) = self.translations.get(key) {
                    details.push(self.clean_html(trans));
                }
            }
        }
        
        details
    }

    pub fn get_item_implicits(&self, id: &str) -> String {
        if let Some(data) = self.item_data_map.get(id) {
            let mut parts = Vec::new();
            for imp in &data.implicits {
                let prop_name = if imp.property_id == 98 {
                    self.player_property_map.get(&imp.tags).map(|s| s.as_str()).unwrap_or("Unknown Player Property")
                } else {
                    self.property_map.get(&imp.property_id).map(|s| s.as_str()).unwrap_or("Unknown Property")
                };
                
                let min = imp.value;
                let max = imp.max_value;
                
                let (min_s, max_s) = if min.abs() < 2.0 && min != 0.0 {
                    (format!("{:.0}%", min * 100.0), format!("{:.0}%", max * 100.0))
                } else {
                    (format!("{:.0}", min), format!("{:.0}", max))
                };

                if min == max {
                    parts.push(format!("{} {}", min_s, prop_name));
                } else {
                    parts.push(format!("{}-{} {}", min_s, max_s, prop_name));
                }
            }
            return parts.join(", ");
        }
        "".to_string()
    }

    pub fn get_skill_name(&self, id: &str) -> String {
        let direct_key = format!("Skills.Skill_{}_0_Name", id);
        if let Some(trans) = self.translations.get(&direct_key) {
            return trans.clone();
        }

        if let Some(data) = self.ability_map.get(id) {
            if let Some(name) = &data.name {
                return name.clone();
            }
            if let Some(key) = &data.name_key {
                if let Some(trans) = self.translations.get(key) {
                    return trans.clone();
                }
                return key.clone();
            }
        }
        id.to_string()
    }

    pub fn get_skill_description(&self, id: &str) -> String {
        if let Some(data) = self.ability_map.get(id) {
            if let Some(key) = &data.description_key {
                if let Some(trans) = self.translations.get(key) {
                    return self.clean_html(trans);
                }
            }
        }
        let key = format!("Skills.Skill_{}_0_Description", id);
        if let Some(trans) = self.translations.get(&key) {
            return self.clean_html(trans);
        }
        "".to_string()
    }

    fn clean_html(&self, input: &str) -> String {
        let mut output = String::new();
        let mut in_tag = false;
        for c in input.chars() {
            if c == '<' {
                in_tag = true;
            } else if c == '>' {
                in_tag = false;
            } else if !in_tag {
                output.push(c);
            }
        }
        output
    }

    pub fn get_skill_node_name(&self, skill_id: &str, node_id: &str) -> String {
        let key = format!("Skills.Skill_{}_{}_Name", skill_id, node_id);
        if let Some(trans) = self.translations.get(&key) {
            return trans.clone();
        }
        format!("Node {}", node_id)
    }

    pub fn get_skill_node_description(&self, skill_id: &str, node_id: &str) -> String {
        let key = format!("Skills.Skill_{}_{}_Description", skill_id, node_id);
        if let Some(trans) = self.translations.get(&key) {
            return self.clean_html(trans);
        }
        "".to_string()
    }

    pub fn get_passive_name(&self, class_id: u8, node_id: u8) -> String {
        let prefix = match class_id {
            0 => "pr",
            1 => "mg",
            2 => "se",
            3 => "ac",
            4 => "rg",
            _ => return format!("Unknown Class {}", class_id),
        };
        
        let key = format!("Skills.Skill_{}-1_{}_Name", prefix, node_id);
        
        if let Some(trans) = self.translations.get(&key) {
            return trans.clone();
        }
        format!("Passive {}", node_id)
    }

    pub fn get_passive_description(&self, class_id: u8, node_id: u8) -> String {
        let prefix = match class_id {
            0 => "pr",
            1 => "mg",
            2 => "se",
            3 => "ac",
            4 => "rg",
            _ => return "".to_string(),
        };
        
        let key = format!("Skills.Skill_{}-1_{}_Description", prefix, node_id);
        
        if let Some(trans) = self.translations.get(&key) {
            return self.clean_html(trans);
        }
        "".to_string()
    }

    pub fn get_item_name(&self, id: &str) -> String {
        if let Some(key) = self.unique_map.get(id) {
             if let Some(trans) = self.translations.get(key) {
                return trans.clone();
            }
            return key.clone();
        }
        if let Some(key) = self.item_map.get(id) {
            if let Some(trans) = self.translations.get(key) {
                return trans.clone();
            }
            return key.clone();
        }
        id.to_string()
    }

    pub fn get_base_type_name(&self, base_type_id: u32) -> Option<&String> {
        self.base_type_name_map.get(&base_type_id)
    }

    pub fn get_item_type_name(&self, id: &str) -> Option<String> {
        if let Some(unique_data) = self.unique_data_map.get(id) {
            return self.get_base_type_name(unique_data.base_type_id).cloned();
        }
        if let Some(item_data) = self.item_data_map.get(id) {
            return self.get_base_type_name(item_data.base_type_id).cloned();
        }
        None
    }
}

