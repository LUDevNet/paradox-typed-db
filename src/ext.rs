//! # Extensions to the auto-generated queries

use assembly_fdb::{
    common::{Latin1Str, Value},
    mem::Row,
};

use crate::{
    columns::{ItemSetsColumn, SkillBehaviorColumn},
    tables::{ItemSetsTable, MissionTasksTable, ObjectsTable, SkillBehaviorTable},
    TypedTable,
};
use serde::Serialize;

/// Well-known components of an object
#[derive(Debug, Copy, Clone, Default)]
pub struct Components {
    /// The render component of the object
    pub render: Option<i32>,
}

/// Data for an item set
#[derive(Debug, Clone)]
pub struct ItemSet {
    /// The object IDs that make up this set
    pub item_ids: Vec<i32>,
    /// The kit faction of this set
    pub kit_type: i32,
    /// The rank of the set
    pub kit_rank: i32,
    /// The ID of the Image used to present the set
    pub kit_image: Option<i32>,
}

impl<'db> ItemSetsTable<'db> {
    /// Get data for a specific item set
    pub fn get_data(&self, id: i32) -> Option<ItemSet> {
        let hash = u32::from_ne_bytes(id.to_ne_bytes());
        let bucket = self.as_raw().bucket_for_hash(hash);

        let col_item_ids = self
            .get_col(ItemSetsColumn::ItemIDs)
            .expect("Missing column 'ItemSets::itemIDs'");
        let col_kit_image = self
            .get_col(ItemSetsColumn::KitImage)
            .expect("Missing column 'ItemSets::kitImage'");
        let col_kit_type = self
            .get_col(ItemSetsColumn::KitType)
            .expect("Missing column 'ItemSets::kitType'");
        let col_kit_rank = self
            .get_col(ItemSetsColumn::KitRank)
            .expect("Missing column 'ItemSets::kitRank'");

        for row in bucket.row_iter() {
            let id_field = row.field_at(0).unwrap();

            if id_field == Value::Integer(id) {
                let kit_type = row
                    .field_at(col_kit_type)
                    .unwrap()
                    .into_opt_integer()
                    .unwrap();
                let kit_rank = row
                    .field_at(col_kit_rank)
                    .unwrap()
                    .into_opt_integer()
                    .unwrap_or(0);
                let kit_image = row.field_at(col_kit_image).unwrap().into_opt_integer();
                let item_ids = row
                    .field_at(col_item_ids)
                    .unwrap()
                    .into_opt_text()
                    .unwrap()
                    .decode()
                    .split(',')
                    .map(str::trim)
                    .filter_map(|idstr| idstr.parse::<i32>().ok())
                    .collect();

                return Some(ItemSet {
                    kit_type,
                    kit_rank,
                    kit_image,
                    item_ids,
                });
            }
        }
        None
    }
}

#[derive(Default)]
/// Metadata for a mission
pub struct Mission {
    /// The icon ID of the mission
    pub mission_icon_id: Option<i32>,
    /// true for missions, false for achievements
    pub is_mission: bool,
}

#[derive(Default)]
/// Data for a mission task
pub struct MissionTask {
    /// The icon ID for the task
    pub icon_id: Option<i32>,
    /// The unique ID of the task
    pub uid: i32,
}

#[derive(Debug, Copy, Clone)]
/// The kind of an entry in the `Missions` table
pub enum MissionKind {
    /// The entry is an achievement (i.e. is active by default)
    Achievement,
    /// The entry is a mission (i.e. has offer & target NPCs)
    Mission,
}

#[derive(Debug, Copy, Clone, Serialize)]
/// Metadata for an object
pub struct ObjectRef<'a> {
    /// The id of the object
    pub id: i32,
    /// The name of the object
    pub name: &'a Latin1Str,
}

impl<'a> ObjectsTable<'a> {
    /// Iterate over all references
    pub fn ref_iter(&self) -> impl Iterator<Item = ObjectRef<'a>> + '_ {
        self.row_iter().map(|row| ObjectRef {
            id: row.id(),
            name: row.name(),
        })
    }
}

#[derive(Copy, Clone)]
/// Data for a skill
pub struct SkillBehavior {
    /// The icon of the skill
    pub skill_icon: Option<i32>,
}

impl<'db> SkillBehaviorTable<'db> {
    /// Get the data for a skill
    pub fn get_data(&self, id: i32) -> Option<SkillBehavior> {
        let hash = u32::from_ne_bytes(id.to_ne_bytes());
        let bucket = self.as_raw().bucket_for_hash(hash);

        let col_skill_icon = self
            .get_col(SkillBehaviorColumn::SkillIcon)
            .expect("Missing 'SkillBehavior::skillIcon'");

        for row in bucket.row_iter() {
            let id_field = row.field_at(0).unwrap();

            if id_field == Value::Integer(id) {
                let skill_icon = row.field_at(col_skill_icon).unwrap().into_opt_integer();

                return Some(SkillBehavior { skill_icon });
            }
        }
        None
    }
}

#[derive(Serialize)]
/// Data for mission tasks
pub struct MissionTaskIcon {
    /// The uid of the task
    uid: i32,
    /// The icon of the task
    #[serde(rename = "largeTaskIconID")]
    large_task_icon_id: Option<i32>,
}

impl<'a> MissionTasksTable<'a> {
    /// Get metadata for all tasks associated with a mission
    pub fn as_task_icon_iter(&self, key: i32) -> impl Iterator<Item = MissionTaskIcon> + '_ {
        self.key_iter(key).map(|x| MissionTaskIcon {
            uid: x.uid(),
            large_task_icon_id: x.large_task_icon_id(),
        })
    }
}
