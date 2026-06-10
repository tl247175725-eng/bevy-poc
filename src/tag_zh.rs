use std::collections::HashMap;
use std::sync::LazyLock;

static TAG_ZH: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("actor", "角色"),
        ("anchor", "锚点"),
        ("animal", "动物"),
        ("animalHome", "兽穴"),
        ("aquatic", "水生"),
        ("autonomous", "自主"),
        ("barren", "荒地"),
        ("basic", "基础"),
        ("being", "生物"),
        ("berry.source", "浆果源"),
        ("blunt", "钝器"),
        ("body.large", "大体型"),
        ("body.medium", "中体型"),
        ("body.small", "小体型"),
        ("body.tiny", "微型"),
        ("burrower", "掘穴"),
        ("bush", "灌木"),
        ("businessUnit", "经营单元"),
        ("camp.anchor", "营地锚"),
        ("camp.fire_bond", "篝火绑定"),
        ("camp.storable", "可储存"),
        ("carnivore", "食肉"),
        ("cell.overlay", "地表覆盖"),
        ("commodity", "商品"),
        ("cone_producer", "松塔产出"),
        ("container", "容器"),
        ("container.water", "水容器"),
        ("cooked", "熟食"),
        ("copper", "铜"),
        ("corpse", "尸体"),
        ("cover.small", "小型掩护"),
        ("cover_user", "利用掩护"),
        ("craft", "制作"),
        ("currency", "货币"),
        ("customer", "顾客"),
        ("den", "窝"),
        ("den_resident", "穴居"),
        ("den.candidate.fox", "狐窝候选"),
        ("dry", "干枯"),
        ("elder", "长者"),
        ("environment", "环境"),
        ("fertile", "肥沃"),
        ("fiber", "纤维"),
        ("filter_feeder", "滤食"),
        ("fire_bond", "火绑定"),
        ("floating", "浮水"),
        ("flocking", "集群"),
        ("food.edible", "可食"),
        ("foodSource", "食物源"),
        ("forager", "采集者"),
        ("forest", "树林"),
        ("fuel", "燃料"),
        ("fuel.fire", "可燃"),
        ("grass", "草"),
        ("grazer", "食草"),
        ("hard", "坚硬"),
        ("heat", "热源"),
        ("herbivore", "食草"),
        ("home", "居所"),
        ("human", "人类"),
        ("juvenile", "幼体"),
        ("largePrey", "大猎物"),
        ("material", "材料"),
        ("material.lumber", "木材"),
        ("material.shard", "碎石"),
        ("material.stone", "石料"),
        ("material.tool_head", "工具头"),
        ("meat_diet", "肉食"),
        ("mesopredator", "中型捕食者"),
        ("migratory", "迁徙"),
        ("mushroomFarm", "菇场"),
        ("nest", "巢"),
        ("nut_producer", "橡子产出"),
        ("observer", "观察者"),
        ("omnivore", "杂食"),
        ("omnivore.small", "小型杂食"),
        ("opportunistic", "机会主义"),
        ("organize.locked", "固定布局"),
        ("pack_hunter", "群猎"),
        ("perishable", "易腐"),
        ("predator", "捕食者"),
        ("primary_producer", "生产者"),
        ("prolific", "高产"),
        ("rooted", "扎根"),
        ("scavenger", "清腐"),
        ("sessile", "固着"),
        ("sharp", "锋利"),
        ("shelter", "庇护"),
        ("small", "小型"),
        ("smallHerbivore", "小型食草"),
        ("smallPrey", "小猎物"),
        ("source.lumber", "木材源"),
        ("source.stone", "石源"),
        ("source.twig", "树枝源"),
        ("structure", "结构"),
        ("terrain", "地形"),
        ("taoyuan", "桃源"),
        ("tiny", "微小"),
        ("tool", "工具"),
        ("tool_dependent", "依赖工具"),
        ("tough", "坚韧"),
        ("tuber", "块茎"),
        ("underground", "地下"),
        ("volant", "飞行"),
        ("water", "水"),
        ("weapon", "武器"),
        ("wildPrey", "野味"),
        ("wood", "木质"),
        ("worker", "劳力"),
        ("youth", "青年"),
    ])
});

static CAP_ZH: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("capability.move", "移动"),
        ("capability.move_normal", "常速移动"),
        ("capability.move_slow", "慢速移动"),
        ("capability.move_quick", "快速移动"),
        ("capability.move_fast", "疾行"),
        ("capability.move_juvenile", "幼体移动"),
        ("capability.hunt", "捕猎"),
        ("capability.forage", "觅食"),
        ("capability.flee", "逃跑"),
        ("capability.reproduce", "繁殖"),
        ("capability.be_hunted", "可被猎"),
        ("capability.follow", "跟随"),
        ("capability.grow", "成长"),
        ("capability.be_cared_for", "需照料"),
        ("capability.care_child", "育幼"),
        ("capability.return_home", "归巢"),
        ("capability.carry", "携带"),
        ("capability.scavenge", "清腐"),
        ("capability.use_cover", "利用掩护"),
        ("capability.escape_small", "小型逃脱"),
        ("capability.escape_fast", "快速逃脱"),
        ("capability.escape_cover", "掩护逃脱"),
        ("capability.be_carried", "可携带"),
        ("capability.be_stored", "可储存"),
        ("capability.be_used_as_material", "可作材料"),
        ("capability.transform_input", "转化输入"),
        ("capability.tool_input", "工具输入"),
        ("capability.use_tool", "使用工具"),
        ("capability.butcher", "屠宰"),
        ("capability.defend", "防御"),
        ("capability.fuel", "作燃料"),
        ("capability.structure_input", "结构输入"),
        ("capability.bond_to_domain", "绑定领域"),
        ("capability.provide_service", "提供服务"),
        ("capability.attract_actor", "吸引角色"),
        ("capability.trade", "交易"),
        ("capability.dining", "用餐"),
        ("capability.shelter", "庇护"),
        ("capability.expand_domain", "扩展领域"),
        ("capability.respond_to_resource", "响应资源"),
        ("capability.define_domain", "定义领域"),
        ("capability.incorporeal", "无形"),
        ("capability.produce_resource", "产出资源"),
        ("capability.regenerate", "再生"),
        ("capability.respond_to_tool", "响应工具"),
        ("capability.protect", "防护"),
        ("capability.accept_resource", "接受资源"),
        ("capability.container", "容器"),
        ("capability.provide_cover", "提供掩护"),
        ("capability.support_den", "支持筑巢"),
        ("capability.bond_to_actor", "绑定角色"),
        ("capability.support_reproduce", "支持繁殖"),
        ("capability.craft", "制作"),
        ("capability.store", "储存"),
        ("capability.observe", "观察"),
        ("capability.social_boundary", "社会边界"),
        ("capability.consume_service", "消费服务"),
        ("capability.be_collected", "可采集"),
        ("capability.be_butchered", "可屠宰"),
        ("capability.sanitation_target", "卫生目标"),
        ("capability.be_consumed", "可食用"),
        ("capability.be_cooked", "可烹饪"),
        ("capability.be_traded", "可交易"),
        ("capability.perishable", "易腐"),
        ("capability.migrate", "迁徙"),
        ("capability.filter_feed", "滤食"),
        ("capability.mature", "成熟"),
        ("capability.work_domain", "经营领域"),
        ("capability.currency", "货币"),
        ("capability.commodity", "商品"),
        ("capability.attracted_by_service", "被服务吸引"),
    ])
});

/// Tags omitted from identity display (structural / duplicate of type name).
pub const SKIP_TAGS: &[&str] = &[
    "being",
    "organize.locked",
    "cell.overlay",
    "camp.storable",
    "camp.fire_bond",
    "camp.anchor",
];

pub fn tag_has_zh_mapping(token: &str) -> bool {
    if TAG_ZH.contains_key(token) {
        return true;
    }
    if let Some(base) = token.split('.').next() {
        if TAG_ZH.contains_key(base) {
            return true;
        }
    }
    token.starts_with("body.")
        || token.starts_with("material.")
        || token.starts_with("fuel.")
        || token.starts_with("den.")
        || token.starts_with("container.")
        || token.starts_with("camp.")
        || token.starts_with("berry.")
        || token.starts_with("cover.")
        || token.starts_with("source.")
        || token.starts_with("organize.")
        || token.starts_with("food.")
        || token.starts_with("drive:")
        || token.starts_with("move_speed:")
        || token.starts_with("sprint:")
        || token.starts_with("social_structure:")
        || token.starts_with("flock_")
        || token.starts_with("repro_")
        || token.starts_with("corpse_type:")
        || token.starts_with("meat_yield:")
        || token.starts_with("meat_product:")
        || token.starts_with("max_starve:")
        || token.starts_with("forages:")
        || token.starts_with("harvest_product:")
        || token.starts_with("perception:")
        || token.starts_with("bridge:")
        || token == "player"
        || token == "meat_diet"
        || token == "den_resident"
        || token == "underground_crop"
        || (token.starts_with("capability.") && cap_has_zh_mapping(token))
}

pub fn cap_has_zh_mapping(cap: &str) -> bool {
    CAP_ZH.contains_key(cap)
}

pub fn tag_zh(token: &str) -> String {
    if let Some(zh) = TAG_ZH.get(token) {
        return (*zh).to_string();
    }
    if let Some(base) = token.split('.').next() {
        if let Some(zh) = TAG_ZH.get(base) {
            return zh.to_string();
        }
    }
    if token.starts_with("body.") {
        return "体型".to_string();
    }
    if token.starts_with("material.") {
        return "材料".to_string();
    }
    if token.starts_with("fuel.") {
        return "燃料".to_string();
    }
    if token.starts_with("den.") {
        return "窝".to_string();
    }
    if token.starts_with("drive:") {
        return "行为驱动".to_string();
    }
    if token.starts_with("move_speed:") {
        return "移动速度".to_string();
    }
    if token.starts_with("sprint:") {
        return "冲刺速度".to_string();
    }
    if token.starts_with("social_structure:") {
        return "社会结构".to_string();
    }
    if token.starts_with("flock_") || token.starts_with("repro_") {
        return "繁殖参数".to_string();
    }
    if token.starts_with("corpse_type:")
        || token.starts_with("meat_yield:")
        || token.starts_with("meat_product:")
        || token.starts_with("max_starve:")
        || token.starts_with("forages:")
        || token.starts_with("harvest_product:")
        || token.starts_with("perception:")
        || token.starts_with("bridge:")
    {
        return "规则参数".to_string();
    }
    if token.starts_with("capability.") {
        return cap_zh(token);
    }
    token_to_fallback_zh(token)
}

pub fn cap_zh(cap: &str) -> String {
    if let Some(zh) = CAP_ZH.get(cap) {
        return (*zh).to_string();
    }
    let key = cap.trim_start_matches("capability.");
    if let Some(zh) = CAP_ZH.get(key) {
        return (*zh).to_string();
    }
    capability_fallback_zh(key)
}

fn capability_fallback_zh(key: &str) -> String {
    key.split('_')
        .map(|part| TAG_ZH.get(part).copied().unwrap_or(part).to_string())
        .collect::<Vec<_>>()
        .join("")
}

fn token_to_fallback_zh(token: &str) -> String {
    let mut out = String::new();
    for (i, part) in token.split('.').enumerate() {
        if i > 0 {
            out.push('·');
        }
        let word: String = part
            .chars()
            .enumerate()
            .map(|(j, c)| {
                if j == 0 {
                    c
                } else if c.is_uppercase() {
                    c
                } else {
                    c
                }
            })
            .collect();
        if let Some(zh) = TAG_ZH.get(word.as_str()) {
            out.push_str(zh);
        } else {
            out.push_str(&camel_to_words(part));
        }
    }
    out
}

fn camel_to_words(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push(' ');
        }
        out.push(c);
    }
    out
}

pub fn contains_english_tag(text: &str) -> bool {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("HP：") || trimmed.starts_with('(') {
            continue;
        }
        if trimmed.chars().any(|c| c.is_ascii_alphabetic()) {
            return true;
        }
    }
    false
}
