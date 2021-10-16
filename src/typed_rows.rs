use super::typed_tables::{
    BehaviorParameterTable, BehaviorTemplateTable, ComponentsRegistryTable,
    DestructibleComponentTable, ItemSetSkillsTable, LootTable, MissionTasksTable, MissionsTable,
    ObjectSkillsTable, ObjectsTable, RebuildComponentTable, SkillBehaviorTable, TypedTable,
};
use assembly_data::fdb::{
    common::{Latin1Str, Latin1String},
    mem::{Field, Row},
};
use serde::{ser::SerializeStruct, Serialize};

pub trait TypedRow<'a, 'b: 'a> {
    type Table: TypedTable<'a> + 'a;

    fn new(inner: Row<'a>, table: &'b Self::Table) -> Self;

    fn get(table: &'b Self::Table, index_key: i32, key: i32, id_col: usize) -> Option<Self>
    where
        Self: Sized,
    {
        let hash = index_key as usize % table.as_table().bucket_count();
        if let Some(b) = table.as_table().bucket_at(hash) {
            for r in b.row_iter() {
                if r.field_at(id_col).and_then(|x| x.into_opt_integer()) == Some(key) {
                    return Some(Self::new(r, table));
                }
            }
        }
        None
    }
}

mod sealed {
    use assembly_data::fdb::mem::Field;

    pub trait Extract<'a> {
        type V: Sized;
        fn from_field(f: Field<'a>) -> Self::V;
    }
}

impl<'a> sealed::Extract<'a> for i32 {
    type V = i32;
    fn from_field(f: Field<'a>) -> Self::V {
        f.into_opt_integer().unwrap()
    }
}

impl<'a> sealed::Extract<'a> for f32 {
    type V = f32;
    fn from_field(f: Field<'a>) -> Self::V {
        f.into_opt_float().unwrap()
    }
}

impl<'a> sealed::Extract<'a> for Option<f32> {
    type V = Option<f32>;
    fn from_field(f: Field<'a>) -> Self::V {
        f.into_opt_float()
    }
}

impl<'a> sealed::Extract<'a> for Option<i32> {
    type V = Option<i32>;
    fn from_field(f: Field<'a>) -> Self::V {
        f.into_opt_integer()
    }
}

impl<'a> sealed::Extract<'a> for bool {
    type V = bool;
    fn from_field(f: Field<'a>) -> Self::V {
        f.into_opt_boolean().unwrap()
    }
}

impl<'a> sealed::Extract<'a> for Option<bool> {
    type V = Option<bool>;
    fn from_field(f: Field<'a>) -> Self::V {
        f.into_opt_boolean()
    }
}

impl<'a> sealed::Extract<'a> for Option<Latin1String> {
    type V = Option<&'a Latin1Str>;
    fn from_field(f: Field<'a>) -> Self::V {
        f.into_opt_text()
    }
}

impl<'a> sealed::Extract<'a> for Latin1String {
    type V = &'a Latin1Str;
    fn from_field(f: Field<'a>) -> Self::V {
        f.into_opt_text().unwrap()
    }
}

macro_rules! extract {
    ($name:ident $col:ident $ty:ty) => {
        pub fn $name(&self) -> <$ty as sealed::Extract<'a>>::V {
            <$ty as sealed::Extract<'a>>::from_field(self.inner.field_at(self.table.$col).unwrap())
        }
    };
}

macro_rules! row_type {
    ($row:ident $table:ident $row_iter_name:ident $key_iter_name:ident) => {
        #[derive(Copy, Clone)]
        pub struct $row<'a, 'b> {
            inner: Row<'a>,
            table: &'b $table<'a>,
        }

        impl<'a, 'b: 'a> TypedRow<'a, 'b> for $row<'a, 'b> {
            type Table = $table<'a>;
            fn new(inner: Row<'a>, table: &'b $table<'a>) -> Self {
                Self { inner, table }
            }
        }

        impl<'a> $table<'a> {
            #[allow(dead_code)]
            pub fn $row_iter_name<'b: 'a>(&'b self) -> impl Iterator<Item = $row<'a, 'b>> {
                self.as_table()
                    .row_iter()
                    .map(move |inner| $row::new(inner, self))
            }
        }

        impl<'a> $table<'a> {
            #[allow(dead_code)]
            pub fn $key_iter_name<'b: 'a>(&'b self, key: i32) -> impl Iterator<Item = $row<'a, 'b>> {
                let hash = key as usize % self.as_table().bucket_count();
                self.as_table()
                    .bucket_at(hash)
                    .unwrap()
                    .row_iter()
                    .map(move |inner| $row::new(inner, self))
            }
        }
    };
    ($row:ident $table:ident) => {
        row_type!($row $table row_iter key_iter);
    }
}

macro_rules! count {
    ($t1:tt $t2:tt $t3:tt $($tr:tt)*) => {
        3 + count!($($tr)*);
    };
    ($t1:tt $t2:tt $($tr:tt)*) => {
        2 + count!($($tr)*);
    };
    ($t1:tt $($tr:tt)*) => {
        1 + count!($($tr)*);
    };
    () => { 0 };
}

macro_rules! ser_impl {
    ($name:ident $str:literal {
        $(
            #[name = $lit:literal, col = $col:ident]
            $fn:ident: $ty:ty
        ),* $(,)?
    }) => {
        impl<'a, 'b> $name<'a, 'b> {
            $(
            extract!($fn $col $ty);
            )*
        }

        impl<'a, 'b> Serialize for $name<'a, 'b> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer {
                let mut s = serializer.serialize_struct($str, count!($($fn)*))?;
                $(
                s.serialize_field($lit, &self.$fn())?;
                )*
                s.end()
            }
        }
    };
}

row_type!(BehaviorParameterRow BehaviorParameterTable);
ser_impl!(BehaviorParameterRow "BehaviorParameter" {
    #[name = "behaviorID", col = col_behavior_id]
    behavior_id: i32,
    #[name = "parameterID", col = col_parameter_id]
    parameter_id: Latin1String,
    #[name = "value", col = col_value]
    value: f32,
});

row_type!(BehaviorTemplateRow BehaviorTemplateTable);
ser_impl!(BehaviorTemplateRow "BehaviorTemplate" {
    #[name = "behaviorID", col = col_behavior_id]
    behavior_id: i32,
    #[name = "templateID", col = col_template_id]
    template_id: i32,
    #[name = "effectID", col = col_effect_id]
    effect_id: Option<i32>,
    #[name = "effectHandle", col = col_effect_handle]
    effect_handle: Option<Latin1String>,
});

row_type!(ComponentsRegistryRow ComponentsRegistryTable);
ser_impl!(ComponentsRegistryRow "ComponentsRegistry" {
    #[name = "id", col = col_id]
    id: i32,
    #[name = "component_type", col = col_component_type]
    component_type: i32,
    #[name = "component_id", col = col_component_id]
    component_id: i32,
});

row_type!(DestructibleComponentRow DestructibleComponentTable);
ser_impl!(DestructibleComponentRow "DestructibleComponent" {
    #[name = "id", col = col_id]
    id: i32,
    #[name = "faction", col = col_faction]
    faction: Option<i32>,
    #[name = "factionList", col = col_faction_list]
    faction_list: Latin1String,
    #[name = "life", col = col_life]
    life: Option<i32>, // only one NULL
    #[name = "imagination", col = col_imagination]
    imagination: i32,
    #[name = "LootMatrixIndex", col = col_loot_matrix_index]
    loot_matrix_index: Option<i32>,
    #[name = "CurrencyIndex", col = col_currency_index]
    currency_index: Option<i32>,
    #[name = "level", col = col_level]
    level: Option<i32>,
    #[name = "armor", col = col_armor]
    armor: Option<f32>,
    #[name = "death_behavior", col = col_death_behavior]
    death_behavior: i32, // ENUM
    #[name = "isnpc", col = col_isnpc]
    isnpc: Option<bool>,
    #[name = "attack_priority", col = col_attack_priority]
    attack_priority: i32,
    #[name = "isSmashable", col = col_is_smashable]
    is_smashable: bool,
    #[name = "difficultyLevel", col = col_difficulty_level]
    difficulty_level: Option<i32>,
});

row_type!(LootTableRow LootTable);
ser_impl!(LootTableRow "LootTable" {
    #[name = "itemid", col = col_itemid]
    itemid: i32,
    #[name = "LootTableIndex", col = col_loot_table_index]
    loot_table_index: i32,
    #[name = "id", col = col_id]
    id: i32,
    #[name = "MissionDrop", col = col_mission_drop]
    mission_drop: bool,
    #[name = "sortPriority", col = col_sort_priority]
    sort_priority: i32,
});

row_type!(MissionsRow MissionsTable);
ser_impl!(MissionsRow "Mission" {
    #[name = "id", col = col_id]
    id: i32,
    #[name = "defined_type", col = col_defined_type]
    defined_type: Option<Latin1String>,
    #[name = "defined_subtype", col = col_defined_subtype]
    defined_subtype: Option<Latin1String>,
    #[name = "isMission", col = col_is_mission]
    is_mission: bool,
    #[name = "UISortOrder", col = col_ui_sort_order]
    ui_sort_order: Option<i32>,
    #[name = "missionIconID", col = col_mission_icon_id]
    mission_icon_id: Option<i32>,
});

row_type!(MissionTaskRow MissionTasksTable);
ser_impl!(MissionTaskRow "MissionTask" {
    #[name = "id", col = col_id]
    id: i32,
    #[name = "locStatus", col = col_loc_status]
    loc_status: i32,
    #[name = "taskType", col = col_task_type]
    task_type: i32,
    #[name = "target", col = col_target]
    target: Option<i32>,
    #[name = "targetGroup", col = col_target_group]
    target_group: Option<Latin1String>,
    #[name = "targetValue", col = col_target_value]
    target_value: Option<i32>,
    #[name = "taskParam1", col = col_task_param1]
    task_param1: Option<Latin1String>,
    #[name = "largeTaskIcon", col = col_large_task_icon]
    large_task_icon: Option<Latin1String>,
    #[name = "IconID", col = col_icon_id]
    icon_id: Option<i32>,
    #[name = "uid", col = col_uid]
    uid: i32,
    #[name = "largeTaskIconID", col = col_large_task_icon_id]
    large_task_icon_id: Option<i32>,
    #[name = "localize", col = col_localize]
    localize: bool,
    #[name = "gate_version", col = col_gate_version]
    gate_version: Option<Latin1String>,
});

row_type!(ObjectsRef ObjectsTable ref_iter ref_key_iter);
ser_impl!(ObjectsRef "ObjectRef" {
    // #[name = "id", col=col_id]
    // id: i32, // 	INTEGER
    #[name = "name", col=col_name]
    name: Latin1String, // 	TEXT
});

row_type!(ObjectsRow ObjectsTable);
ser_impl!(ObjectsRow "Object" {
    #[name = "id", col=col_id]
    id: i32, // 	INTEGER
    #[name = "name", col=col_name]
    name: Latin1String, // 	TEXT
    #[name = "placeable", col=col_placeable]
    placeable: bool, // 	BOOLEAN
    #[name = "type", col=col_type]
    r#type: Latin1String, // 	TEXT
    #[name = "description", col=col_description]
    description: Option<Latin1String>, // 	TEXT
    #[name = "localize", col=col_localize]
    localize: bool, // 	BOOLEAN
    #[name = "npcTemplateID", col=col_npc_template_id]
    npc_template_id: Option<i32>, // 	INTEGER
    #[name = "displayName", col=col_display_name]
    display_name: Option<Latin1String>, // 	TEXT
    #[name = "interactionDistance", col=col_interaction_distance]
    interaction_distance: Option<f32>, // 	FLOAT
    #[name = "nametag", col=col_nametag]
    nametag: bool, // 	BOOLEAN
    #[name = "_internalNotes", col=col_internal_notes]
    internal_notes: Option<Latin1String>, // 	TEXT
    #[name = "locStatus", col=col_loc_status]
    loc_status: i32, // 	INTEGER
    #[name = "gate_version", col=col_gate_version]
    gate_version: Option<Latin1String>, // 	TEXT
    #[name = "HQ_valid", col=col_hq_valid]
    hq_valid: bool, // 	BOOLEAN
});

row_type!(ObjectSkillsRow ObjectSkillsTable);

impl<'a> ObjectSkillsRow<'a, '_> {
    extract!(skill_id col_skill_id i32);
    extract!(object_template col_object_template i32);
}

row_type!(ItemSetSkillsRow ItemSetSkillsTable);

impl<'a> ItemSetSkillsRow<'a, '_> {
    extract!(skill_set_id col_skill_set_id i32);
    extract!(skill_id col_skill_id i32);
    //extract!(skill_cast_type col_skill_cast_type i32);
}

row_type!(RebuildComponentRow RebuildComponentTable);
ser_impl!(RebuildComponentRow "RebuildComponent" {
    #[name = "id", col = col_id]
    id: i32, //
    #[name = "reset_time", col = col_reset_time]
    reset_time: f32, //
    #[name = "complete_time", col = col_complete_time]
    complete_time: Option<f32>, //
    #[name = "take_imagination", col = col_take_imagination]
    take_imagination: i32, //
    #[name = "interruptible", col = col_interruptible]
    interruptible: bool, //
    #[name = "self_activator", col = col_self_activator]
    self_activator: bool, //
    #[name = "custom_modules", col = col_custom_modules]
    custom_modules: Option<Latin1String>, //
    #[name = "activityID", col = col_activity_id]
    activity_id: Option<i32>, //
    #[name = "post_imagination_cost", col = col_post_imagination_cost]
    post_imagination_cost: Option<i32>, //
    #[name = "time_before_smash", col = col_time_before_smash]
    time_before_smash: f32, //
});

row_type!(SkillBehaviorRow SkillBehaviorTable);
ser_impl!(SkillBehaviorRow "SkillBehavior" {
    #[name = "skillID", col = col_skill_id]
    skill_id: i32,
    #[name = "locStatus", col = col_loc_status]
    loc_status: i32,
    #[name = "behaviorID", col = col_behavior_id]
    behavior_id: i32,
    #[name = "imaginationcost", col = col_imaginationcost]
    imaginationcost: i32,
    #[name = "cooldowngroup", col = col_cooldowngroup]
    cooldowngroup: i32,
    #[name = "cooldown", col = col_cooldown]
    cooldown: f32,
    #[name = "inNpcEditor", col = col_in_npc_editor]
    in_npc_editor: bool,
    #[name = "skillIcon", col = col_skill_icon]
    skill_icon: i32,
    #[name = "oomSkillID", col = col_oom_skill_id]
    oom_skill_id: Latin1String,
    #[name = "oomBehaviorEffectID", col = col_oom_behavior_effect_id]
    oom_behavior_effect_id: i32,
    #[name = "castTypeDesc", col = col_cast_type_desc]
    cast_type_desc: i32,
    #[name = "imBonusUI", col = col_im_bonus_ui]
    im_bonus_ui: i32,
    #[name = "lifeBonusUI", col = col_life_bonus_ui]
    life_bonus_ui: i32,
    #[name = "armorBonusUI", col = col_armor_bonus_ui]
    armor_bonus_ui: i32,
    #[name = "damageUI", col = col_damage_ui]
    damage_ui: i32,
    #[name = "hideIcon", col = col_hide_icon]
    hide_icon: bool,
    #[name = "localize", col = col_localize]
    localize: bool,
    #[name = "gate_version", col = col_gate_version]
    gate_version: Latin1String,
    #[name = "cancelType", col = col_cancel_type]
    cancel_type: i32,
});
