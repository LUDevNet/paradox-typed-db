use assembly_data::fdb::common::Value;

use super::typed_tables::{ItemSetsTable, SkillBehaviorTable, TypedTable};

#[derive(Debug, Copy, Clone, Default)]
pub struct Components {
    pub render: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct ItemSet {
    pub item_ids: Vec<i32>,
    pub kit_type: i32,
    pub kit_rank: i32,
    pub kit_image: Option<i32>,
}

impl<'db> ItemSetsTable<'db> {
    pub fn get_data(&self, id: i32) -> Option<ItemSet> {
        let hash = u32::from_ne_bytes(id.to_ne_bytes());
        let bucket = self.as_table().bucket_for_hash(hash);

        for row in bucket.row_iter() {
            let id_field = row.field_at(0).unwrap();

            if id_field == Value::Integer(id) {
                let kit_type = row
                    .field_at(self.col_kit_type)
                    .unwrap()
                    .into_opt_integer()
                    .unwrap();
                let kit_rank = row
                    .field_at(self.col_kit_rank)
                    .unwrap()
                    .into_opt_integer()
                    .unwrap_or(0);
                let kit_image = row.field_at(self.col_kit_image).unwrap().into_opt_integer();
                let item_ids = row
                    .field_at(self.col_item_ids)
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
pub struct Mission {
    pub mission_icon_id: Option<i32>,
    pub is_mission: bool,
}

#[derive(Default)]
pub struct MissionTask {
    pub icon_id: Option<i32>,
    pub uid: i32,
}

#[derive(Debug, Copy, Clone)]
pub enum MissionKind {
    Achievement,
    Mission,
}

#[derive(Copy, Clone)]
pub struct SkillBehavior {
    pub skill_icon: Option<i32>,
}

impl<'db> SkillBehaviorTable<'db> {
    pub fn get_data(&self, id: i32) -> Option<SkillBehavior> {
        let hash = u32::from_ne_bytes(id.to_ne_bytes());
        let bucket = self.as_table().bucket_for_hash(hash);

        for row in bucket.row_iter() {
            let id_field = row.field_at(0).unwrap();

            if id_field == Value::Integer(id) {
                let skill_icon = row
                    .field_at(self.col_skill_icon)
                    .unwrap()
                    .into_opt_integer();

                return Some(SkillBehavior { skill_icon });
            }
        }
        None
    }
}
