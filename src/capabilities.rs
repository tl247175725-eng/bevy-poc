use std::collections::HashMap;
use std::sync::LazyLock;

static CAPABILITIES: LazyLock<HashMap<&'static str, &'static [&'static str]>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("player", &["capability.move", "capability.hunt", "capability.forage", "capability.craft", "capability.store", "capability.trade", "capability.use_tool", "capability.carry"][..]);
    m.insert("stone", &["capability.be_carried", "capability.be_stored", "capability.be_used_as_material", "capability.transform_input"][..]);
    m.insert("shard", &["capability.be_carried", "capability.be_stored", "capability.be_used_as_material", "capability.transform_input", "capability.tool_input"][..]);
    m.insert("tri", &["capability.be_carried", "capability.be_used_as_material", "capability.tool_input"][..]);
    m.insert("square", &["capability.be_carried", "capability.be_used_as_material", "capability.tool_input"][..]);
    m.insert("knife", &["capability.be_carried", "capability.use_tool", "capability.hunt", "capability.butcher"][..]);
    m.insert("spear", &["capability.be_carried", "capability.use_tool", "capability.hunt", "capability.defend"][..]);
    m.insert("wood", &["capability.be_carried", "capability.be_stored", "capability.be_used_as_material", "capability.transform_input", "capability.fuel"][..]);
    m.insert("twig", &["capability.be_carried", "capability.be_stored", "capability.be_used_as_material", "capability.transform_input", "capability.fuel"][..]);
    m.insert("woodStruct", &["capability.be_carried", "capability.be_used_as_material", "capability.structure_input"][..]);
    m.insert("table", &["capability.bond_to_domain", "capability.provide_service", "capability.attract_actor", "capability.trade", "capability.dining"][..]);
    m.insert("hut", &["capability.bond_to_domain", "capability.provide_service", "capability.shelter", "capability.expand_domain"][..]);
    m.insert("axe", &["capability.be_carried", "capability.be_stored", "capability.use_tool", "capability.hunt", "capability.respond_to_resource"][..]);
    m.insert("hoe", &["capability.be_carried", "capability.use_tool"][..]);
    m.insert("hammer", &["capability.be_carried", "capability.use_tool", "capability.transform_input"][..]);
    m.insert("barrenLand", &["capability.define_domain", "capability.incorporeal"][..]);
    m.insert("mountain", &["capability.define_domain", "capability.produce_resource"][..]);
    m.insert("tree", &["capability.define_domain", "capability.produce_resource", "capability.regenerate", "capability.respond_to_tool"][..]);
    m.insert("fire", &["capability.define_domain", "capability.protect", "capability.transform_input", "capability.provide_service"][..]);
    m.insert("charcoal", &["capability.be_carried", "capability.fuel", "capability.be_used_as_material"][..]);
    m.insert("bucket", &["capability.be_carried", "capability.be_stored", "capability.accept_resource", "capability.container"][..]);
    m.insert("waterbucket", &["capability.be_carried", "capability.be_stored", "capability.transform_input", "capability.container"][..]);
    m.insert("halfbucket", &["capability.be_carried", "capability.container"][..]);
    m.insert("sheep", &["capability.move", "capability.move_normal", "capability.forage", "capability.reproduce", "capability.flee", "capability.be_hunted"][..]);
    m.insert("lamb", &["capability.move", "capability.move_slow", "capability.follow", "capability.grow", "capability.be_cared_for", "capability.be_hunted"][..]);
    m.insert("rabbit", &["capability.move", "capability.move_quick", "capability.escape_small", "capability.forage", "capability.reproduce", "capability.flee", "capability.be_hunted"][..]);
    m.insert("pheasant", &["capability.move", "capability.forage", "capability.escape_small", "capability.flee", "capability.reproduce", "capability.be_hunted", "capability.care_child"][..]);
    m.insert("pheasantChick", &["capability.move", "capability.escape_small", "capability.follow", "capability.grow", "capability.be_cared_for", "capability.be_hunted"][..]);
    m.insert("deer", &["capability.move", "capability.move_normal", "capability.escape_fast", "capability.forage", "capability.flee", "capability.be_hunted", "capability.reproduce"][..]);
    m.insert("deerFawn", &["capability.move", "capability.move_slow", "capability.follow", "capability.grow", "capability.be_cared_for", "capability.be_hunted"][..]);
    m.insert("taoyuanElder", &["capability.move", "capability.observe", "capability.social_boundary"][..]);
    m.insert("taoyuanForager", &["capability.move", "capability.observe", "capability.social_boundary"][..]);
    m.insert("taoyuanYouth", &["capability.move", "capability.observe", "capability.social_boundary"][..]);
    m.insert("wolf", &["capability.move", "capability.move_quick", "capability.hunt", "capability.reproduce", "capability.care_child", "capability.return_home", "capability.carry"][..]);
    m.insert("wolfCub", &["capability.move", "capability.move_juvenile", "capability.follow", "capability.grow", "capability.be_cared_for", "capability.return_home"][..]);
    m.insert("wolfDen", &["capability.define_domain", "capability.bond_to_actor", "capability.support_reproduce", "capability.provide_service"][..]);
    m.insert("humus", &[][..]);
    m.insert("sheepCorpse", &["capability.be_butchered", "capability.sanitation_target"][..]);
    m.insert("deerCorpse", &["capability.be_butchered", "capability.sanitation_target"][..]);
    m.insert("wolfCorpse", &["capability.be_butchered", "capability.sanitation_target"][..]);
    m.insert("playerCorpse", &["capability.be_butchered", "capability.sanitation_target"][..]);
    m.insert("rabbitCorpse", &["capability.be_butchered", "capability.sanitation_target"][..]);
    m.insert("pheasantCorpse", &["capability.be_butchered", "capability.sanitation_target"][..]);
    m.insert("foxCorpse", &["capability.be_butchered", "capability.sanitation_target"][..]);
    m.insert("waterBuffaloCorpse", &["capability.be_butchered", "capability.sanitation_target"][..]);
    m.insert("fieldMouseCorpse", &["capability.be_butchered", "capability.sanitation_target"][..]);
    m.insert("bambooRatCorpse", &["capability.be_butchered", "capability.sanitation_target"][..]);
    m.insert("fishCorpse", &["capability.be_butchered", "capability.sanitation_target"][..]);
    m.insert("sheepMeat", &["capability.be_carried", "capability.be_stored", "capability.be_consumed", "capability.be_cooked", "capability.be_traded", "capability.perishable"][..]);
    m.insert("rabbitMeat", &["capability.be_carried", "capability.be_stored", "capability.be_consumed", "capability.be_cooked", "capability.be_traded", "capability.perishable"][..]);
    m.insert("deerMeat", &["capability.be_carried", "capability.be_stored", "capability.be_consumed", "capability.be_cooked", "capability.be_traded", "capability.perishable"][..]);
    m.insert("wolfMeat", &["capability.be_carried", "capability.be_stored", "capability.be_consumed", "capability.be_cooked", "capability.be_traded", "capability.perishable"][..]);
    m.insert("humanMeat", &["capability.be_carried", "capability.be_stored", "capability.be_consumed", "capability.be_cooked", "capability.perishable"][..]);
    m.insert("fishMeat", &["capability.be_carried", "capability.be_stored", "capability.be_consumed", "capability.be_cooked", "capability.be_traded", "capability.perishable"][..]);
    m.insert("foxMeat", &["capability.be_carried", "capability.be_stored", "capability.be_consumed", "capability.be_cooked", "capability.be_traded", "capability.perishable"][..]);
    m.insert("pheasantMeat", &["capability.be_carried", "capability.be_stored", "capability.be_consumed", "capability.be_cooked", "capability.be_traded", "capability.perishable"][..]);
    m.insert("buffaloMeat", &["capability.be_carried", "capability.be_stored", "capability.be_consumed", "capability.be_cooked", "capability.be_traded", "capability.perishable"][..]);
    m.insert("mouseMeat", &["capability.be_carried", "capability.be_stored", "capability.be_consumed", "capability.be_cooked", "capability.be_traded", "capability.perishable"][..]);
    m.insert("algae", &["capability.define_domain", "capability.produce_resource", "capability.regenerate"][..]);
    m.insert("waterBug", &["capability.move", "capability.move_fast", "capability.forage", "capability.flee", "capability.be_hunted", "capability.migrate"][..]);
    m.insert("fish", &["capability.move", "capability.move_quick", "capability.forage", "capability.flee", "capability.be_hunted", "capability.migrate"][..]);
    m.insert("shellfish", &["capability.define_domain", "capability.produce_resource", "capability.filter_feed", "capability.be_hunted"][..]);
    m.insert("oak", &["capability.be_collected"][..]);
    m.insert("pine", &["capability.be_collected"][..]);
    m.insert("waterCaltrop", &["capability.be_collected"][..]);
    m.insert("lotus", &["capability.be_collected"][..]);
    m.insert("acorn", &["capability.be_carried", "capability.be_stored", "capability.be_consumed"][..]);
    m.insert("pineCone", &["capability.be_carried", "capability.be_stored", "capability.be_consumed"][..]);
    m.insert("caltropFruit", &["capability.be_carried", "capability.be_stored", "capability.be_consumed"][..]);
    m.insert("lotusSeed", &["capability.be_carried", "capability.be_stored", "capability.be_consumed"][..]);
    m.insert("birdNest", &["capability.be_collected"][..]);
    m.insert("landBug", &["capability.move", "capability.be_hunted"][..]);
    m.insert("wildYam", &["capability.be_collected"][..]);
    m.insert("wildYamRoot", &["capability.be_carried", "capability.be_stored", "capability.be_consumed"][..]);
    m.insert("berry", &["capability.be_carried", "capability.be_stored", "capability.be_consumed"][..]);
    m.insert("mushroom", &["capability.be_carried", "capability.be_stored", "capability.be_consumed", "capability.be_traded"][..]);
    m.insert("wetWood", &["capability.be_carried", "capability.mature", "capability.be_used_as_material"][..]);
    m.insert("mushroomWood", &["capability.be_carried", "capability.produce_resource", "capability.be_used_as_material"][..]);
    m.insert("mushroomGreenhouse", &["capability.define_domain", "capability.produce_resource", "capability.attract_actor", "capability.provide_service"][..]);
    m.insert("mushroomFarmer", &["capability.move", "capability.craft", "capability.use_tool", "capability.work_domain"][..]);
    m.insert("cookmeat", &["capability.be_carried", "capability.be_stored", "capability.be_consumed", "capability.be_traded"][..]);
    m.insert("coin", &["capability.be_carried", "capability.be_stored", "capability.currency", "capability.transform_input"][..]);
    m.insert("copperBlock", &["capability.be_carried", "capability.be_stored", "capability.be_used_as_material"][..]);
    m.insert("copperCraft", &["capability.be_carried", "capability.be_stored", "capability.be_traded", "capability.commodity"][..]);
    m.insert("traveler", &["capability.move", "capability.trade", "capability.consume_service", "capability.attracted_by_service"][..]);
    m.insert("grass", &["capability.define_domain", "capability.produce_resource", "capability.be_used_as_material", "capability.shelter"][..]);
    m.insert("bush", &["capability.define_domain", "capability.produce_resource", "capability.shelter", "capability.provide_cover", "capability.regenerate", "capability.support_den"][..]);
    m.insert("fieldMouse", &["capability.move", "capability.move_fast", "capability.escape_small", "capability.escape_cover", "capability.forage", "capability.use_cover", "capability.reproduce", "capability.flee", "capability.be_hunted"][..]);
    m.insert("bambooRat", &["capability.move", "capability.forage", "capability.use_cover", "capability.escape_cover", "capability.flee", "capability.reproduce", "capability.be_hunted"][..]);
    m.insert("fieldMousePup", &["capability.move", "capability.move_slow", "capability.escape_cover", "capability.follow", "capability.grow", "capability.be_cared_for", "capability.use_cover", "capability.be_hunted"][..]);
    m.insert("fox", &["capability.move", "capability.move_quick", "capability.escape_fast", "capability.escape_cover", "capability.hunt", "capability.use_cover", "capability.scavenge", "capability.return_home", "capability.care_child", "capability.reproduce"][..]);
    m.insert("foxCub", &["capability.move", "capability.move_juvenile", "capability.follow", "capability.grow", "capability.be_cared_for", "capability.return_home"][..]);
    m.insert("foxDen", &["capability.define_domain", "capability.bond_to_actor", "capability.support_reproduce", "capability.provide_service"][..]);
    m.insert("waterBuffalo", &["capability.move", "capability.move_slow", "capability.forage", "capability.flee", "capability.be_hunted"][..]);
    m.insert("waterBuffaloCalf", &["capability.move", "capability.move_juvenile", "capability.forage", "capability.flee", "capability.be_hunted", "capability.grow"][..]);
    m.insert("dryGrass", &["capability.be_carried", "capability.be_used_as_material", "capability.transform_input"][..]);
    m.insert("grassRope", &["capability.be_carried", "capability.be_used_as_material"][..]);
    m
});

pub fn card_capabilities(type_name: &str) -> &'static [&'static str] {
    CAPABILITIES.get(type_name).copied().unwrap_or(&[])
}

pub fn all_capability_cards() -> impl Iterator<Item = &'static str> {
    CAPABILITIES.keys().copied()
}

pub fn capability_count() -> usize {
    CAPABILITIES.len()
}

