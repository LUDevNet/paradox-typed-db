use std::{
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use assembly_core::buffer::CastError;
use assembly_data::{
    fdb::{
        common::{Latin1Str, Value},
        mem::Tables,
    },
    xml::localization::LocaleNode,
};

pub mod typed_ext;
pub mod typed_rows;
pub mod typed_tables;

use typed_tables::{
    BehaviorParameterTable, BehaviorTemplateTable, ComponentsRegistryTable, IconsTable,
    ItemSetSkillsTable, ItemSetsTable, MissionTasksTable, MissionsTable, ObjectSkillsTable,
    ObjectsTable, RebuildComponentTable, RenderComponentTable, SkillBehaviorTable, TypedTable,
};

use self::typed_ext::{Components, Mission, MissionKind, MissionTask};

#[derive(Clone)]
pub struct TypedDatabase<'db> {
    pub locale: Arc<LocaleNode>,
    /// LU-Res Prefix
    pub lu_res_prefix: &'db str,
    /// BehaviorParameter
    pub behavior_parameters: BehaviorParameterTable<'db>,
    /// BehaviorTemplate
    pub behavior_templates: BehaviorTemplateTable<'db>,
    /// ComponentRegistry
    pub comp_reg: ComponentsRegistryTable<'db>,
    /// Icons
    pub icons: IconsTable<'db>,
    /// ItemSets
    pub item_sets: ItemSetsTable<'db>,
    /// ItemSetSkills
    pub item_set_skills: ItemSetSkillsTable<'db>,
    /// Missions
    pub missions: MissionsTable<'db>,
    /// MissionTasks
    pub mission_tasks: MissionTasksTable<'db>,
    /// Objects
    pub objects: ObjectsTable<'db>,
    /// Objects
    pub object_skills: ObjectSkillsTable<'db>,
    /// RebuildComponent
    pub rebuild_component: RebuildComponentTable<'db>,
    /// RenderComponent
    pub render_comp: RenderComponentTable<'db>,
    /// SkillBehavior
    pub skills: SkillBehaviorTable<'db>,
}

fn is_not_empty(s: &&Latin1Str) -> bool {
    !s.is_empty()
}

fn cleanup_path(url: &Latin1Str) -> Option<PathBuf> {
    let url = url.decode().replace('\\', "/").to_ascii_lowercase();
    let p = Path::new(&url);

    let mut path = Path::new("/textures/ui").to_owned();
    for comp in p.components() {
        match comp {
            Component::ParentDir => {
                path.pop();
            }
            Component::CurDir => {}
            Component::Normal(seg) => path.push(seg),
            Component::RootDir => return None,
            Component::Prefix(_) => return None,
        }
    }
    path.set_extension("png");
    Some(path)
}

impl<'a> TypedDatabase<'a> {
    pub fn new(
        locale: Arc<LocaleNode>,
        lu_res_prefix: &'a str,
        tables: Tables<'a>,
    ) -> Result<Self, CastError> {
        let behavior_parameter_inner = tables.by_name("BehaviorParameter").unwrap()?;
        let behavior_template_inner = tables.by_name("BehaviorTemplate").unwrap()?;
        let components_registry_inner = tables.by_name("ComponentsRegistry").unwrap()?;
        let icons_inner = tables.by_name("Icons").unwrap()?;
        let item_sets_inner = tables.by_name("ItemSets").unwrap()?;
        let item_set_skills_inner = tables.by_name("ItemSetSkills").unwrap()?;
        let missions_inner = tables.by_name("Missions").unwrap()?;
        let mission_tasks_inner = tables.by_name("MissionTasks").unwrap()?;
        let objects_inner = tables.by_name("Objects").unwrap()?;
        let object_skills_inner = tables.by_name("ObjectSkills").unwrap()?;
        let rebuild_component_inner = tables.by_name("RebuildComponent").unwrap()?;
        let render_component_inner = tables.by_name("RenderComponent").unwrap()?;
        let skill_behavior_inner = tables.by_name("SkillBehavior").unwrap()?;
        Ok(TypedDatabase {
            locale,
            lu_res_prefix,
            behavior_parameters: BehaviorParameterTable::new(behavior_parameter_inner),
            behavior_templates: BehaviorTemplateTable::new(behavior_template_inner),
            comp_reg: ComponentsRegistryTable::new(components_registry_inner),
            icons: IconsTable::new(icons_inner),
            item_sets: ItemSetsTable::new(item_sets_inner),
            item_set_skills: ItemSetSkillsTable::new(item_set_skills_inner),
            missions: MissionsTable::new(missions_inner),
            mission_tasks: MissionTasksTable::new(mission_tasks_inner),
            objects: ObjectsTable::new(objects_inner),
            object_skills: ObjectSkillsTable::new(object_skills_inner),
            rebuild_component: RebuildComponentTable::new(rebuild_component_inner),
            render_comp: RenderComponentTable::new(render_component_inner),
            skills: SkillBehaviorTable::new(skill_behavior_inner),
        })
    }

    pub fn get_mission_name(&self, kind: MissionKind, id: i32) -> Option<String> {
        let missions = self.locale.str_children.get("Missions").unwrap();
        if id > 0 {
            if let Some(mission) = missions.int_children.get(&(id as u32)) {
                if let Some(name_node) = mission.str_children.get("name") {
                    let name = name_node.value.as_ref().unwrap();
                    return Some(format!("{} | {:?} #{}", name, kind, id));
                }
            }
        }
        None
    }

    pub fn get_item_set_name(&self, rank: i32, id: i32) -> Option<String> {
        let missions = self.locale.str_children.get("ItemSets").unwrap();
        if id > 0 {
            if let Some(mission) = missions.int_children.get(&(id as u32)) {
                if let Some(name_node) = mission.str_children.get("kitName") {
                    let name = name_node.value.as_ref().unwrap();
                    return Some(if rank > 0 {
                        format!("{} (Rank {}) | Item Set #{}", name, rank, id)
                    } else {
                        format!("{} | Item Set #{}", name, id)
                    });
                }
            }
        }
        None
    }

    pub fn get_skill_name_desc(&self, id: i32) -> (Option<String>, Option<String>) {
        let skills = self.locale.str_children.get("SkillBehavior").unwrap();
        let mut the_name = None;
        let mut the_desc = None;
        if id > 0 {
            if let Some(skill) = skills.int_children.get(&(id as u32)) {
                if let Some(name_node) = skill.str_children.get("name") {
                    let name = name_node.value.as_ref().unwrap();
                    the_name = Some(format!("{} | Item Set #{}", name, id));
                }
                if let Some(desc_node) = skill.str_children.get("descriptionUI") {
                    let desc = desc_node.value.as_ref().unwrap();
                    the_desc = Some(desc.clone());
                }
            }
        }
        (the_name, the_desc)
    }

    pub fn get_icon_path(&self, id: i32) -> Option<PathBuf> {
        let hash = u32::from_ne_bytes(id.to_ne_bytes());
        let bucket = self.icons.as_table().bucket_for_hash(hash);

        for row in bucket.row_iter() {
            let id_field = row.field_at(0).unwrap();

            if id_field == Value::Integer(id) {
                if let Some(url) = row
                    .field_at(self.icons.col_icon_path)
                    .unwrap()
                    .into_opt_text()
                {
                    return cleanup_path(url);
                }
            }
        }
        None
    }

    pub fn get_mission_data(&self, id: i32) -> Option<Mission> {
        let hash = u32::from_ne_bytes(id.to_ne_bytes());
        let bucket = self.missions.as_table().bucket_for_hash(hash);

        for row in bucket.row_iter() {
            let id_field = row.field_at(0).unwrap();

            if id_field == Value::Integer(id) {
                let mission_icon_id = row
                    .field_at(self.missions.col_mission_icon_id)
                    .unwrap()
                    .into_opt_integer();
                let is_mission = row
                    .field_at(self.missions.col_is_mission)
                    .unwrap()
                    .into_opt_boolean()
                    .unwrap_or(true);

                return Some(Mission {
                    mission_icon_id,
                    is_mission,
                });
            }
        }
        None
    }

    pub fn get_mission_tasks(&self, id: i32) -> Vec<MissionTask> {
        let hash = u32::from_ne_bytes(id.to_ne_bytes());
        let bucket = self.mission_tasks.as_table().bucket_for_hash(hash);
        let mut tasks = Vec::with_capacity(4);

        for row in bucket.row_iter() {
            let id_field = row.field_at(0).unwrap();

            if id_field == Value::Integer(id) {
                let icon_id = row
                    .field_at(self.mission_tasks.col_icon_id)
                    .unwrap()
                    .into_opt_integer();
                let uid = row
                    .field_at(self.mission_tasks.col_uid)
                    .unwrap()
                    .into_opt_integer()
                    .unwrap();

                tasks.push(MissionTask { icon_id, uid })
            }
        }
        tasks
    }

    pub fn get_object_name_desc(&self, id: i32) -> Option<(String, String)> {
        let hash = u32::from_ne_bytes(id.to_ne_bytes());

        let table = self.objects.as_table();
        let bucket = table
            .bucket_at(hash as usize % table.bucket_count())
            .unwrap();

        for row in bucket.row_iter() {
            let mut fields = row.field_iter();
            let id_field = fields.next().unwrap();
            if id_field == Value::Integer(id) {
                let name = fields.next().unwrap(); // 1: name
                let description = fields.nth(2).unwrap(); // 4: description
                let display_name = fields.nth(2).unwrap(); // 7: displayName
                let internal_notes = fields.nth(2).unwrap(); // 10: internalNotes

                let title = match (
                    name.into_opt_text().filter(is_not_empty),
                    display_name.into_opt_text().filter(is_not_empty),
                ) {
                    (Some(name), Some(display)) if display != name => {
                        format!("{} ({}) | Object #{}", display.decode(), name.decode(), id)
                    }
                    (Some(name), _) => {
                        format!("{} | Object #{}", name.decode(), id)
                    }
                    (None, Some(display)) => {
                        format!("{} | Object #{}", display.decode(), id)
                    }
                    (None, None) => {
                        format!("Object #{}", id)
                    }
                };
                let desc = match (
                    description.into_opt_text().filter(is_not_empty),
                    internal_notes.into_opt_text().filter(is_not_empty),
                ) {
                    (Some(description), Some(internal_notes)) if description != internal_notes => {
                        format!("{} ({})", description.decode(), internal_notes.decode(),)
                    }
                    (Some(description), _) => {
                        format!("{}", description.decode())
                    }
                    (None, Some(internal_notes)) => {
                        format!("{}", internal_notes.decode())
                    }
                    (None, None) => String::new(),
                };
                return Some((title, desc));
            }
        }
        None
    }

    pub fn get_render_image(&self, id: i32) -> Option<String> {
        let hash = u32::from_ne_bytes(id.to_ne_bytes());
        let table = self.render_comp.as_table();
        let bucket = table
            .bucket_at(hash as usize % table.bucket_count())
            .unwrap();

        for row in bucket.row_iter() {
            let mut fields = row.field_iter();
            let id_field = fields.next().unwrap();
            if id_field == Value::Integer(id) {
                let _render_asset = fields.next().unwrap();
                let icon_asset = fields.next().unwrap();

                if let Value::Text(url) = icon_asset {
                    let path = cleanup_path(url)?;
                    return Some(self.to_res_href(&path));
                }
            }
        }
        None
    }

    pub fn to_res_href(&self, path: &Path) -> String {
        format!("{}{}", self.lu_res_prefix, path.display())
    }

    pub fn get_components(&self, id: i32) -> Components {
        let hash = u32::from_ne_bytes(id.to_ne_bytes());
        let table = self.comp_reg.as_table();
        let bucket = table
            .bucket_at(hash as usize % table.bucket_count())
            .unwrap();

        let mut comp = Components::default();

        for row in bucket.row_iter() {
            let mut fields = row.field_iter();
            let id_field = fields.next().unwrap();
            if id_field == Value::Integer(id) {
                let component_type = fields.next().unwrap();
                let component_id = fields.next().unwrap();

                if let Value::Integer(2) = component_type {
                    comp.render = component_id.into_opt_integer();
                }
            }
        }
        comp
    }
}
