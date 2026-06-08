# Card Rule Facts

> 自动导出自 `CardDB`。本文件只记录代码事实，不承载设计判断。

## Summary

- cards: 53
- relations: 22
- impact recipes: 9

## Cards

| type | name | layer | tags | rooted | autonomous | hp | value | weapon_range | craft_depth | script |
|---|---|---|---|---|---|---|---|---|---|---|
| axe | 斧头 | entity | sharp, tool, weapon, commodity, camp.storable | false | false | 0 | 0 | 0 | 3 |  |
| berry | 浆果 | entity | consumable, food, food.edible, basic, camp.storable | false | false | 0 | 0 | 0 | 0 |  |
| bucket | 空桶 | entity | container, container.water, camp.storable | false | false | 0 | 0 | 0 | 0 |  |
| bush | 灌木丛 | cover | bush, natural, shelter, organize.locked | false | false | 0 | 0 | 0 | 0 |  |
| charcoal | 木炭 | entity | fuel | false | false | 0 | 0 | 0 | 0 |  |
| coin | 铜钱 | entity | currency, anchor, copper, camp.storable | false | false | 0 | 0 | 0 | 0 |  |
| cookmeat | 熟肉 | entity | consumable, food, food.edible, cooked, commodity, camp.storable | false | false | 0 | 1 | 0 | 2 |  |
| copperBlock | 铜块 | entity | copper, material, camp.storable | false | false | 0 | 0 | 0 | 1 |  |
| copperCraft | 铜饰 | entity | consumable, craft, copper, novelty, commodity, camp.storable | false | false | 0 | 5 | 0 | 4 |  |
| deer | 鹿 | entity | being, animal, herbivore, wildPrey, largePrey | false | false | 3 | 0 | 0 | 0 |  |
| deerCorpse | 鹿尸体 | entity | corpse, organic, organize.locked | false | false | 0 | 0 | 0 | 0 |  |
| deerMeat | 鹿肉 | entity | consumable, food, food.raw, raw, camp.storable | false | false | 0 | 0 | 0 | 0 |  |
| dryGrass | 干草 | cover | dry, fiber, material | false | false | 0 | 0 | 0 | 0 |  |
| fire | 篝火 | entity | heat, environment, camp.anchor, organize.locked | false | false | 0 | 0 | 0 | 0 |  |
| grass | 草皮 | cover | grass, foodSource, organize.locked | false | false | 0 | 0 | 0 | 0 |  |
| grassRope | 草绳 | entity | fiber, craftPart, material | false | false | 0 | 0 | 0 | 2 |  |
| halfbucket | 半桶水 | entity | water, container | false | false | 0 | 0 | 0 | 0 |  |
| hammer | 锤子 | entity | hard, blunt, tool, commodity | false | false | 0 | 0 | 0 | 3 |  |
| hoe | 锄头 | entity | tool, commodity | false | false | 0 | 0 | 0 | 3 |  |
| humanMeat | 人肉 | entity | consumable, food, food.raw, raw, camp.storable | false | false | 0 | 0 | 0 | 0 |  |
| hut | 草棚 | entity | shelter, structure, home, camp.fire_bond, organize.locked | false | false | 0 | 0 | 0 | 3 |  |
| knife | 石制小刀 | entity | sharp, tool, weapon, commodity | false | false | 0 | 1 | 2 | 2 |  |
| lamb | 羊羔 | entity | being, animal, juvenile | false | false | 1 | 0 | 0 | 0 |  |
| mountain | 山 | entity | rooted, environment, stoneSource, source.stone, organize.locked | true | false | 0 | 0 | 0 | 0 |  |
| mushroom | 蘑菇 | entity | consumable, food, food.edible, basic, camp.storable | false | false | 0 | 0 | 0 | 0 |  |
| mushroomFarmer | 蘑菇农 | entity | being, worker, autonomous | false | true | 6 | 0 | 0 | 0 | res://scripts/cards/mushroom_farmer_card.gd |
| mushroomGreenhouse | 蘑菇棚 | entity | shelter, structure, mushroomFarm, organize.locked | false | false | 0 | 0 | 0 | 4 |  |
| mushroomWood | 长蘑菇的木头 | entity | wood, mushroomSource, material | false | false | 0 | 0 | 0 | 0 |  |
| player | 玩家 | entity | being, actor | false | false | 6 | 0 | 0 | 0 | res://scripts/cards/player_card.gd |
| playerCorpse | 遗体 | entity | corpse, organic, organize.locked | false | false | 0 | 0 | 0 | 0 |  |
| rabbit | 野兔 | entity | being, animal, smallHerbivore | false | false | 1 | 0 | 0 | 0 |  |
| rabbitMeat | 兔肉 | entity | consumable, food, food.raw, raw, camp.storable | false | false | 0 | 0 | 0 | 0 |  |
| shard | 碎石 | entity | hard, material, material.shard, camp.storable | false | false | 0 | 0 | 0 | 0 |  |
| sheep | 羊 | entity | being, animal | false | false | 2 | 0 | 0 | 0 |  |
| sheepCorpse | 羊尸体 | entity | corpse, organic, organize.locked | false | false | 0 | 0 | 0 | 0 |  |
| sheepMeat | 羊肉 | entity | consumable, food, food.raw, raw, camp.storable | false | false | 0 | 0 | 0 | 0 |  |
| spear | 长矛 | entity | sharp, tool, weapon, commodity | false | false | 0 | 2 | 2 | 2 |  |
| square | 正方碎石 | entity | hard, toolHead | false | false | 0 | 0 | 0 | 0 |  |
| stone | 石头 | entity | hard, blunt, material.stone, camp.storable | false | false | 0 | 0 | 0 | 0 |  |
| table | 桌子 | entity | businessUnit, structure, camp.fire_bond, organize.locked | false | false | 0 | 0 | 0 | 3 |  |
| traveler | 旅人 | entity | being, customer, autonomous | false | true | 4 | 0 | 0 | 0 | res://scripts/cards/traveler_card.gd |
| tree | 树林 | entity | woodSource, rooted, forest, source.lumber, source.twig, organize.locked | true | false | 0 | 0 | 0 | 0 |  |
| tri | 三角碎石 | entity | hard, sharp, toolHead, material.tool_head | false | false | 0 | 0 | 0 | 0 |  |
| twig | 树枝 | entity | wood, fuel, fuel.fire, material, material.lumber, naturalDrop, camp.storable | false | false | 0 | 0 | 0 | 0 |  |
| waterbucket | 一桶水 | entity | water, container, container.water, camp.storable | false | false | 0 | 0 | 0 | 0 |  |
| wetWood | 湿木头 | entity | wood, wet, material | false | false | 0 | 0 | 0 | 0 |  |
| wolf | 狼 | entity | being, animal, predator, autonomous | false | true | 4 | 0 | 0 | 0 | res://scripts/cards/wolf_card.gd |
| wolfCorpse | 狼尸体 | entity | corpse, organic, organize.locked | false | false | 0 | 0 | 0 | 0 |  |
| wolfCub | 幼狼 | entity | being, animal, predator, juvenile, autonomous | false | true | 1 | 0 | 0 | 0 | res://scripts/cards/wolf_card.gd |
| wolfDen | 狼窝 | entity | den, shelter, animalHome | true | false | 0 | 0 | 0 | 0 |  |
| wolfMeat | 狼肉 | entity | consumable, food, food.raw, raw, camp.storable | false | false | 0 | 0 | 0 | 0 |  |
| wood | 木头 | entity | wood, fuel, fuel.fire, material, material.lumber, camp.storable | false | false | 0 | 0 | 0 | 0 |  |
| woodStruct | 木构件 | entity | structure, material | false | false | 0 | 0 | 0 | 2 |  |

## Relations

| a | b | result | bidirectional | spawn_near_target |
|---|---|---|---|---|
| twig | shard | spear | true | false |
| woodStruct | wood | table | true | false |
| woodStruct | twig | table | true | false |
| wood | grass | hut | true | false |
| twig | grass | hut | true | false |
| wood | dryGrass | hut | true | false |
| twig | dryGrass | hut | true | false |
| tri | wood | axe | true | false |
| tri | twig | axe | true | false |
| square | wood | hoe | true | false |
| square | twig | hoe | true | false |
| stone | wood | hammer | true | false |
| stone | twig | hammer | true | false |
| copperBlock | grassRope | copperCraft | true | false |
| hut | mushroomWood | mushroomGreenhouse | true | false |
| wood | fire | charcoal | false | true |
| twig | fire | charcoal | false | true |
| sheepMeat | fire | cookmeat | false | true |
| rabbitMeat | fire | cookmeat | false | true |
| deerMeat | fire | cookmeat | false | true |
| wolfMeat | fire | cookmeat | false | true |
| humanMeat | fire | cookmeat | false | true |

## Impact Recipes

| source | target | result | extra | handler | hits | consumes_source | consumes_target |
|---|---|---|---|---|---|---|---|
| stone | stone | shard |  |  | 2 | false | true |
| wood_like | wood_like | woodStruct |  |  | 2 | false | true |
| wood_like | shard | fire |  |  | 2 | false | true |
| hard | shard |  |  | shape_shard | 2 | false | true |
| player | dryGrass | grassRope |  |  | 2 | false | true |
| axe | tree |  |  | chop_tree | 2 | false | true |
| * | wolfDen |  |  | wolf_den | 2 | false | true |
| mushroomFarmer | shard |  |  | farmer_knife | 2 | false | true |
| * | mushroomWood |  |  | harvest_mushroom | 2 | false | true |
