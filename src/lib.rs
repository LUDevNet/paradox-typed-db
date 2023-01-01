#![warn(missing_docs)]

//! # Typed bindings to `CDClient.fdb`
//!
//! This crate provides typed bindings to an FDB file that follows the structure of
//! `CDClient.fdb` from the 1.10.64 client. The design goals are:
//!
//! - Make writing code that uses this API as easy as possible
//! - Enable serialization with the [`serde`](https://serde.rs) crate
//! - Accept FDBs that may have additional columns and tables

use assembly_core::buffer::CastError;
use assembly_fdb::{
    mem::{Field, Row, Table, Tables},
    value::Value,
};
use latin1str::Latin1Str;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

pub mod ext;

use columns::{IconsColumn, MissionTasksColumn, MissionsColumn};
use tables::{
    ActivitiesTable, ActivityTextTable, BehaviorParameterTable, BehaviorTemplateTable,
    CollectibleComponentTable, ComponentsRegistryTable, CurrencyDenominationsTable,
    DeletionRestrictionsTable, DestructibleComponentTable, EmotesTable, IconsTable,
    InventoryComponentTable, ItemComponentTable, ItemSetSkillsTable, ItemSetsTable,
    JetPackPadComponentTable, LootMatrixTable, LootTableTable, MissionEmailTable,
    MissionNpcComponentTable, MissionTasksTable, MissionTextTable, MissionsTable, NpcIconsTable,
    ObjectSkillsTable, ObjectsTable, PlayerStatisticsTable, PreconditionsTable,
    PropertyTemplateTable, RebuildComponentTable, RebuildSectionsTable, RenderComponentTable,
    RewardCodesTable, RewardsTable, SkillBehaviorTable, SpeedchatMenuTable,
    TamingBuildPuzzlesTable, UgBehaviorSoundsTable, WhatsCoolItemSpotlightTable,
    WhatsCoolNewsAndTipsTable, ZoneLoadingTipsTable, ZoneTableTable,
};

use self::ext::{Components, Mission, MissionTask};

/// ## A "typed" database row
///
/// A typed table is the combination of a "raw" table from the `assembly_fdb` crate with
/// some metadata. Examples for this metadata are:
///
/// - Mapping from a well-known column name (e.g. `MissionID`) to the "real" column index within the FDB
pub trait TypedTable<'de>: Sized {
    /// The type representing one well-known column
    type Column: Copy + Clone + Eq;
    /// The literal name of the table
    const NAME: &'static str;

    /// Return the contained "raw" table
    fn as_raw(&self) -> Table<'de>;
    /// Create a typed table from a raw table.
    ///
    /// This function constructs the necessary metadata.
    fn new(inner: Table<'de>) -> Self;

    /// Get an instance from a database
    fn of(tables: Tables<'de>) -> Option<Result<Self, CastError>> {
        let table = tables.by_name(Self::NAME)?;
        Some(table.map(Self::new))
    }
}

/// ## A "typed" database row
///
/// A typed row is the combination of a "raw" row from the `assembly_fdb crate with the typing information
/// given in [`TypedRow::Table`].
pub trait TypedRow<'a, 'b>
where
    'a: 'b,
{
    /// The table this row belongs to
    type Table: TypedTable<'a> + 'a;

    /// Creates a new "typed" row from a "typed" table and a "raw" row
    fn new(inner: Row<'a>, table: &'b Self::Table) -> Self;

    /// Get a specific entry from the row by unique ID
    ///
    /// The `index_key` is the value of the first column, the `key` is the value of the unique ID column
    /// and the `id_col` must be the "real" index of that unique ID column.
    fn get(table: &'b Self::Table, index_key: i32, key: i32, id_col: usize) -> Option<Self>
    where
        Self: Sized,
    {
        let hash = index_key as usize % table.as_raw().bucket_count();
        if let Some(b) = table.as_raw().bucket_at(hash) {
            for r in b.row_iter() {
                if r.field_at(id_col).and_then(|x| x.into_opt_integer()) == Some(key) {
                    return Some(Self::new(r, table));
                }
            }
        }
        None
    }
}

/// # Iterator over [`TypedRow`]s
///
/// This class is used to iterate over typed rows
pub struct RowIter<'a, 'b, R>
where
    R: TypedRow<'a, 'b>,
{
    inner: assembly_fdb::mem::iter::TableRowIter<'a>,
    table: &'b R::Table,
}

impl<'a, 'b, R> RowIter<'a, 'b, R>
where
    R: TypedRow<'a, 'b>,
{
    /// Create a new row iter from a typed table
    pub fn new(table: &'b R::Table) -> Self {
        Self {
            inner: table.as_raw().row_iter(),
            table,
        }
    }
}

impl<'a, 'b, R> Iterator for RowIter<'a, 'b, R>
where
    R: TypedRow<'a, 'b>,
{
    type Item = R;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|row| R::new(row, self.table))
    }
}

#[derive(Clone)]
/// A selection of relevant database tables
pub struct TypedDatabase<'db> {
    /// Activities
    pub activities: ActivitiesTable<'db>,
    /// ActivityText
    pub activity_text: ActivityTextTable<'db>,
    /// BehaviorParameter
    pub behavior_parameters: BehaviorParameterTable<'db>,
    /// BehaviorTemplate
    pub behavior_templates: BehaviorTemplateTable<'db>,
    /// CollectibleComponent
    pub collectible_component: CollectibleComponentTable<'db>,
    /// ComponentRegistry
    pub comp_reg: ComponentsRegistryTable<'db>,
    /// CurrencyDenominations
    pub currency_denominations: CurrencyDenominationsTable<'db>,
    /// DeletionRestrictions
    pub deletion_restrictions: DeletionRestrictionsTable<'db>,
    /// DestructibleComponent
    pub destructible_component: DestructibleComponentTable<'db>,
    /// Emotes
    pub emotes: EmotesTable<'db>,
    /// Icons
    pub icons: IconsTable<'db>,
    /// InventoryComponent
    pub inventory_component: InventoryComponentTable<'db>,
    /// ItemComponent
    pub item_component: ItemComponentTable<'db>,
    /// ItemSets
    pub item_sets: ItemSetsTable<'db>,
    /// ItemSetSkills
    pub item_set_skills: ItemSetSkillsTable<'db>,
    /// JetPackPadComponent
    pub jet_pack_pad_component: JetPackPadComponentTable<'db>,
    /// LootTable
    pub loot_table: LootTableTable<'db>,
    /// LootMatrix
    pub loot_matrix: LootMatrixTable<'db>,
    /// MissionEmail
    pub mission_email: MissionEmailTable<'db>,
    /// MissionNPCComponent
    pub mission_npc_component: MissionNpcComponentTable<'db>,
    /// MissionTasks
    pub mission_tasks: MissionTasksTable<'db>,
    /// MissionText
    pub mission_text: MissionTextTable<'db>,
    /// Missions
    pub missions: MissionsTable<'db>,
    /// NpcIcons
    pub npc_icons: NpcIconsTable<'db>,
    /// Objects
    pub objects: ObjectsTable<'db>,
    /// Objects
    pub object_skills: ObjectSkillsTable<'db>,
    /// PlayerStatistics
    pub player_statistics: PlayerStatisticsTable<'db>,
    /// Preconditions
    pub preconditions: PreconditionsTable<'db>,
    /// PropertyTemplate
    pub property_template: PropertyTemplateTable<'db>,
    /// RebuildComponent
    pub rebuild_component: RebuildComponentTable<'db>,
    /// RebuildSections
    pub rebuild_sections: Option<RebuildSectionsTable<'db>>,
    /// Rewards
    pub rewards: RewardsTable<'db>,
    /// RewardCodes
    pub reward_codes: RewardCodesTable<'db>,
    /// RenderComponent
    pub render_comp: RenderComponentTable<'db>,
    /// SkillBehavior
    pub skills: SkillBehaviorTable<'db>,
    /// SpeedchatMenu
    pub speedchat_menu: SpeedchatMenuTable<'db>,
    /// TamingBuildPuzzles
    pub taming_build_puzzles: TamingBuildPuzzlesTable<'db>,
    /// UGBehaviorSounds
    pub ug_behavior_sounds: UgBehaviorSoundsTable<'db>,
    /// WhatsCoolItemSpotlight
    pub whats_cool_item_spotlight: WhatsCoolItemSpotlightTable<'db>,
    /// WhatsCoolNewsAndTips
    pub whats_cool_news_and_tips: WhatsCoolNewsAndTipsTable<'db>,
    /// ZoneLoadingTips
    pub zone_loading_tips: ZoneLoadingTipsTable<'db>,
    /// ZoneTable
    pub zone_table: ZoneTableTable<'db>,
}

fn is_not_empty(s: &&Latin1Str) -> bool {
    !s.is_empty()
}

impl<'a> TypedDatabase<'a> {
    /// Construct a new typed database
    pub fn new(tables: Tables<'a>) -> Result<Self, CastError> {
        Ok(TypedDatabase {
            activities: ActivitiesTable::of(tables).unwrap()?,
            activity_text: ActivityTextTable::of(tables).unwrap()?,
            behavior_parameters: BehaviorParameterTable::of(tables).unwrap()?,
            behavior_templates: BehaviorTemplateTable::of(tables).unwrap()?,
            collectible_component: CollectibleComponentTable::of(tables).unwrap()?,
            comp_reg: ComponentsRegistryTable::of(tables).unwrap()?,
            currency_denominations: CurrencyDenominationsTable::of(tables).unwrap()?,
            deletion_restrictions: DeletionRestrictionsTable::of(tables).unwrap()?,
            destructible_component: DestructibleComponentTable::of(tables).unwrap()?,
            emotes: EmotesTable::of(tables).unwrap()?,
            icons: IconsTable::of(tables).unwrap()?,
            inventory_component: InventoryComponentTable::of(tables).unwrap()?,
            item_component: ItemComponentTable::of(tables).unwrap()?,
            item_sets: ItemSetsTable::of(tables).unwrap()?,
            item_set_skills: ItemSetSkillsTable::of(tables).unwrap()?,
            jet_pack_pad_component: JetPackPadComponentTable::of(tables).transpose()?,
            loot_matrix: LootMatrixTable::of(tables).unwrap()?,
            loot_table: LootTableTable::of(tables).unwrap()?,
            mission_email: MissionEmailTable::of(tables).unwrap()?,
            mission_npc_component: MissionNpcComponentTable::of(tables).unwrap()?,
            mission_tasks: MissionTasksTable::of(tables).unwrap()?,
            mission_text: MissionTextTable::of(tables).unwrap()?,
            missions: MissionsTable::of(tables).unwrap()?,
            npc_icons: NpcIconsTable::of(tables).unwrap()?,
            objects: ObjectsTable::of(tables).unwrap()?,
            object_skills: ObjectSkillsTable::of(tables).unwrap()?,
            player_statistics: PlayerStatisticsTable::of(tables).unwrap()?,
            preconditions: PreconditionsTable::of(tables).unwrap()?,
            property_template: PropertyTemplateTable::of(tables).unwrap()?,
            rewards: RewardsTable::of(tables).unwrap()?,
            reward_codes: RewardCodesTable::of(tables).unwrap()?,
            rebuild_component: RebuildComponentTable::of(tables).unwrap()?,
            rebuild_sections: RebuildSectionsTable::of(tables).transpose()?,
            render_comp: RenderComponentTable::of(tables).unwrap()?,
            skills: SkillBehaviorTable::of(tables).unwrap()?,
            speedchat_menu: SpeedchatMenuTable::of(tables).unwrap()?,
            taming_build_puzzles: TamingBuildPuzzlesTable::of(tables).unwrap()?,
            ug_behavior_sounds: UgBehaviorSoundsTable::of(tables).unwrap()?,
            whats_cool_item_spotlight: WhatsCoolItemSpotlightTable::of(tables).unwrap()?,
            whats_cool_news_and_tips: WhatsCoolNewsAndTipsTable::of(tables).unwrap()?,
            zone_loading_tips: ZoneLoadingTipsTable::of(tables).unwrap()?,
            zone_table: ZoneTableTable::of(tables).unwrap()?,
        })
    }

    /// Get the path of an icon ID
    pub fn get_icon_path(&self, id: i32) -> Option<&Latin1Str> {
        let hash = u32::from_ne_bytes(id.to_ne_bytes());
        let bucket = self.icons.as_raw().bucket_for_hash(hash);

        let col_icon_path = self
            .icons
            .get_col(IconsColumn::IconPath)
            .expect("Missing column 'Icons::IconPath'");

        for row in bucket.row_iter() {
            let id_field = row.field_at(0).unwrap();

            if id_field == Field::Integer(id) {
                return row.field_at(col_icon_path).unwrap().into_opt_text();
            }
        }
        None
    }

    /// Get data for the specified mission ID
    pub fn get_mission_data(&self, id: i32) -> Option<Mission> {
        let hash = u32::from_ne_bytes(id.to_ne_bytes());
        let bucket = self.missions.as_raw().bucket_for_hash(hash);

        let col_mission_icon_id = self
            .missions
            .get_col(MissionsColumn::MissionIconId)
            .expect("Missing column 'Missions::mission_icon_id'");
        let col_is_mission = self
            .missions
            .get_col(MissionsColumn::IsMission)
            .expect("Missing column 'Missions::is_mission'");

        for row in bucket.row_iter() {
            let id_field = row.field_at(0).unwrap();

            if id_field == Field::Integer(id) {
                let mission_icon_id = row
                    .field_at(col_mission_icon_id)
                    .unwrap()
                    .into_opt_integer();
                let is_mission = row
                    .field_at(col_is_mission)
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

    /// Get a list of mission tasks for the specified mission ID
    pub fn get_mission_tasks(&self, id: i32) -> Vec<MissionTask> {
        let hash = u32::from_ne_bytes(id.to_ne_bytes());
        let bucket = self.mission_tasks.as_raw().bucket_for_hash(hash);
        let mut tasks = Vec::with_capacity(4);

        let col_icon_id = self
            .mission_tasks
            .get_col(MissionTasksColumn::IconId)
            .expect("Missing column 'MissionTasks::icon_id'");
        let col_uid = self
            .mission_tasks
            .get_col(MissionTasksColumn::Uid)
            .expect("Missing column 'MissionTasks::uid'");

        for row in bucket.row_iter() {
            let id_field = row.field_at(0).unwrap();

            if id_field == Field::Integer(id) {
                let icon_id = row.field_at(col_icon_id).unwrap().into_opt_integer();
                let uid = row.field_at(col_uid).unwrap().into_opt_integer().unwrap();

                tasks.push(MissionTask { icon_id, uid })
            }
        }
        tasks
    }

    /// Get the name and description for the specified LOT
    pub fn get_object_name_desc(&self, id: i32) -> Option<(String, String)> {
        let hash = u32::from_ne_bytes(id.to_ne_bytes());

        let table = self.objects.as_raw();
        let bucket = table
            .bucket_at(hash as usize % table.bucket_count())
            .unwrap();

        for row in bucket.row_iter() {
            let mut fields = row.field_iter();
            let id_field = fields.next().unwrap();
            if id_field == Field::Integer(id) {
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

    /// Get the path of the icon asset of the specified render component
    pub fn get_render_image(&self, id: i32) -> Option<&Latin1Str> {
        let hash = u32::from_ne_bytes(id.to_ne_bytes());
        let table = self.render_comp.as_raw();
        let bucket = table
            .bucket_at(hash as usize % table.bucket_count())
            .unwrap();

        for row in bucket.row_iter() {
            let mut fields = row.field_iter();
            let id_field = fields.next().unwrap();
            if id_field == Field::Integer(id) {
                let _render_asset = fields.next().unwrap();
                let icon_asset = fields.next().unwrap();

                if let Field::Text(url) = icon_asset {
                    return Some(url);
                }
            }
        }
        None
    }

    /// Get all components for the specified LOT
    pub fn get_components(&self, id: i32) -> Components {
        let hash = u32::from_ne_bytes(id.to_ne_bytes());
        let table = self.comp_reg.as_raw();
        let bucket = table
            .bucket_at(hash as usize % table.bucket_count())
            .unwrap();

        let mut comp = Components::default();

        for row in bucket.row_iter() {
            let mut fields = row.field_iter();
            let id_field = fields.next().unwrap();
            if id_field == Field::Integer(id) {
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
