use assembly_data::fdb::mem::Table;
use serde::Serialize;

pub trait TypedTable<'de> {
    fn as_table(&self) -> Table<'de>;
    fn new(inner: Table<'de>) -> Self;
}

macro_rules! make_typed {
    ($name:ident { $($(#[$meta:meta])*$col:ident $lit:literal),+ $(,)?}) => {
        #[derive(Copy, Clone)]
        #[allow(dead_code)]
        pub struct $name<'db> {
            inner: Table<'db>,
            $(pub $col: usize),+
        }

        impl<'db> TypedTable<'db> for $name<'db> {
            fn as_table(&self) -> Table<'db> {
                self.inner
            }

            fn new(inner: Table<'db>) -> Self {
                $(let mut $col = None;)+

                for (index, col) in inner.column_iter().enumerate() {
                    match col.name_raw().as_bytes() {
                        $($lit => $col = Some(index),)+
                        _ => continue,
                    }
                }

                Self {
                    inner,
                    $($col: $col.unwrap(),)+
                }
            }
        }
    };
}

make_typed!(BehaviorParameterTable {
    col_behavior_id b"behaviorID",
    col_parameter_id b"parameterID",
    col_value b"value",
});

make_typed!(BehaviorTemplateTable {
    col_behavior_id b"behaviorID",
    col_template_id b"templateID",
    col_effect_id b"effectID",
    col_effect_handle b"effectHandle",
});

make_typed!(ComponentsRegistryTable {
    col_id b"id",
    col_component_type b"component_type",
    col_component_id b"component_id",
});

make_typed!(DestructibleComponentTable {
    col_id b"id", // INTEGER
    col_faction b"faction", // INTEGER
    col_faction_list b"factionList", // TEXT
    col_life b"life", // INTEGER
    col_imagination b"imagination", // INTEGER
    col_loot_matrix_index b"LootMatrixIndex", // INTEGER
    col_currency_index b"CurrencyIndex", // INTEGER
    col_level b"level", // INTEGER
    col_armor b"armor", // FLOAT
    col_death_behavior b"death_behavior", // INTEGER
    col_isnpc b"isnpc", // BOOLEAN
    col_attack_priority b"attack_priority", // INTEGER
    col_is_smashable b"isSmashable", // BOOLEAN
    col_difficulty_level b"difficultyLevel", // INTEGER
});

make_typed!(IconsTable {
    col_icon_path b"IconPath",
    col_icon_name b"IconName",
});

make_typed!(ItemSetSkillsTable {
    col_skill_set_id b"SkillSetID",
    col_skill_id b"SkillID",
    col_skill_cast_type b"SkillCastType",
});

make_typed!(ItemSetsTable {
    /// itemIDs: ", " separated LOTs
    col_item_ids b"itemIDs",
    /// kitType i.e. faction
    col_kit_type b"kitType",
    /// kitRank
    col_kit_rank b"kitRank",
    /// kitImage
    col_kit_image b"kitImage",
});

make_typed!(LootTable {
    /// itemid
    col_itemid b"itemid",
    /// LootTableIndex
    col_loot_table_index b"LootTableIndex",
    /// id
    col_id b"id",
    /// MissionDrop
    col_mission_drop b"MissionDrop",
    /// sortPriority
    col_sort_priority b"sortPriority",
});

make_typed!(MissionsTable {
    col_id b"id",
    col_defined_type b"defined_type",
    col_defined_subtype b"defined_subtype",
    col_ui_sort_order b"UISortOrder",
    col_is_mission b"isMission",
    col_mission_icon_id b"missionIconID",
});

make_typed!(MissionTasksTable {
    col_id b"id",
    col_loc_status b"locStatus",
    col_task_type b"taskType",
    col_target b"target",
    col_target_group b"targetGroup",
    col_target_value b"targetValue",
    col_task_param1 b"taskParam1",
    col_large_task_icon b"largeTaskIcon",
    col_icon_id b"IconID",
    col_uid b"uid",
    col_large_task_icon_id b"largeTaskIconID",
    col_localize b"localize",
    col_gate_version b"gate_version"
});

#[derive(Serialize)]
pub struct MissionTaskIcon {
    uid: i32,
    #[serde(rename = "largeTaskIconID")]
    large_task_icon_id: Option<i32>,
}

impl<'a> MissionTasksTable<'a> {
    pub fn as_task_icon_iter(&self, key: i32) -> impl Iterator<Item = MissionTaskIcon> + '_ {
        self.key_iter(key).map(|x| MissionTaskIcon {
            uid: x.uid(),
            large_task_icon_id: x.large_task_icon_id(),
        })
    }
}

make_typed!(ObjectsTable {
    col_id b"id",
    col_name b"name",
    col_placeable b"placeable",
    col_type b"type",
    col_description b"description",
    col_localize b"localize",
    col_npc_template_id b"npcTemplateID",
    col_display_name b"displayName",
    col_interaction_distance b"interactionDistance",
    col_nametag b"nametag",
    col_internal_notes b"_internalNotes",
    col_loc_status b"locStatus",
    col_gate_version b"gate_version",
    col_hq_valid b"HQ_valid",
});

make_typed!(ObjectSkillsTable {
    col_object_template b"objectTemplate",
    col_skill_id b"skillID",
    col_cast_on_type b"castOnType",
    col_ai_combat_weight b"AICombatWeight",
});

make_typed!(RebuildComponentTable {
    col_id b"id", // 	INTEGER
    col_reset_time b"reset_time", // 	FLOAT
    col_complete_time b"complete_time", // 	FLOAT
    col_take_imagination b"take_imagination", // 	INTEGER
    col_interruptible b"interruptible", // 	BOOLEAN
    col_self_activator b"self_activator", // 	BOOLEAN
    col_custom_modules b"custom_modules", // 	TEXT
    col_activity_id b"activityID", // 	INTEGER
    col_post_imagination_cost b"post_imagination_cost", // 	INTEGER
    col_time_before_smash b"time_before_smash", // 	FLOAT
});

make_typed!(SkillBehaviorTable {
    col_skill_id b"skillID",
    col_loc_status b"locStatus",
    col_behavior_id b"behaviorID",
    col_imaginationcost b"imaginationcost",
    col_cooldowngroup b"cooldowngroup",
    col_cooldown b"cooldown",
    col_in_npc_editor b"inNpcEditor",
    col_skill_icon b"skillIcon",
    col_oom_skill_id b"oomSkillID",
    col_oom_behavior_effect_id b"oomBehaviorEffectID",
    col_cast_type_desc b"castTypeDesc",
    col_im_bonus_ui b"imBonusUI",
    col_life_bonus_ui b"lifeBonusUI",
    col_armor_bonus_ui b"armorBonusUI",
    col_damage_ui b"damageUI",
    col_hide_icon b"hideIcon",
    col_localize b"localize",
    col_gate_version b"gate_version",
    col_cancel_type b"cancelType"
});
