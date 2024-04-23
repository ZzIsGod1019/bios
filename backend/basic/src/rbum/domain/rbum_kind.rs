use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::sea_query::{ColumnDef, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;
use tardis::TardisCreateIndex;

/// Resource kind model
///
/// 资源类型模型
///
/// A resource kind is a set of common resources.
/// E.g. `/tenant/**` , `/app/**` these are all APIs, and these are all API-kind resources; `/tenant/list` ,
/// `/tenant/detail#more` these are all menus, and these are all  menu-kind resources.
///
/// 资源类型是一组共同的资源。
/// 例如 `/tenant/**` , `/app/**` 这些都是API，这些都是API类型的资源； `/tenant/list` , `/tenant/detail#more` 这些都是菜单，这些都是菜单类型的资源。
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateIndex)]
#[sea_orm(table_name = "rbum_kind")]
pub struct Model {
    /// Resource kind id
    ///
    /// 资源类型id
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
        /// Resource kind module
    ///
    /// 资源类型模块
    /// 
    /// Used to further divide the resource  kind. For example, there are multiple resource  kinds under the ``cmdb compute`` module, such as ``ecs, ec2, k8s``.
    /// 
    /// 用于对资源类型做简单的分类。比如 ``cmdb计算`` 模块下可以有 ``ecs、ec2、k8s`` 等多个资源类型。
    pub module: String,
    /// Resource kind code
    ///
    /// 资源类型编码
    ///
    /// Resource kind code, which is required to conform to the scheme specification in the uri, matching the regular: ``^[a-z0-9-.]+$`` .
    ///
    /// 资源类型编码，需要符合uri中的scheme规范，匹配正则：``^[a-z0-9-.]+$`` 。
    #[index(unique)]
    pub code: String,
    /// Resource kind name
    ///
    /// 资源类型名称
    pub name: String,
    /// Resource kind note
    ///
    /// 资源类型备注
    pub note: String,
    /// Resource kind icon
    ///
    /// 资源类型图标
    pub icon: String,
    /// Resource kind sort
    ///
    /// 资源类型排序
    pub sort: i64,
    /// Extension table name
    /// 
    /// 扩展表名
    /// 
    /// Each resource kind can specify an extension table for storing customized data.
    /// 
    /// 每个资源类型可以指定一个扩展表用于存储自定义数据。
    pub ext_table_name: String,

    pub scope_level: i16,

    #[index]
    pub own_paths: String,
    pub owner: String,
    pub create_time: chrono::DateTime<Utc>,
    pub update_time: chrono::DateTime<Utc>,
    pub create_by: String,
    pub update_by: String,
}

impl TardisActiveModel for ActiveModel {
    fn fill_ctx(&mut self, ctx: &TardisContext, is_insert: bool) {
        if is_insert {
            self.own_paths = Set(ctx.own_paths.to_string());
            self.owner = Set(ctx.owner.to_string());
            self.create_by = Set(ctx.owner.to_string());
        }
        self.update_by = Set(ctx.owner.to_string());
    }

    fn create_table_statement(db: DbBackend) -> TableCreateStatement {
        let mut builder = Table::create();
        builder
            .table(Entity.table_ref())
            .if_not_exists()
            .col(ColumnDef::new(Column::Id).not_null().string().primary_key())
            // Specific
            .col(ColumnDef::new(Column::Code).not_null().string())
            .col(ColumnDef::new(Column::Module).not_null().string())
            .col(ColumnDef::new(Column::Name).not_null().string())
            .col(ColumnDef::new(Column::Note).not_null().string())
            .col(ColumnDef::new(Column::Icon).not_null().string())
            .col(ColumnDef::new(Column::Sort).not_null().big_integer())
            .col(ColumnDef::new(Column::ExtTableName).not_null().string())
            // Basic
            .col(ColumnDef::new(Column::OwnPaths).not_null().string())
            .col(ColumnDef::new(Column::Owner).not_null().string())
            // With Scope
            .col(ColumnDef::new(Column::ScopeLevel).not_null().small_integer())
            .col(ColumnDef::new(Column::CreateBy).not_null().string())
            .col(ColumnDef::new(Column::UpdateBy).not_null().string());
        if db == DatabaseBackend::Postgres {
            builder
                .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp_with_time_zone())
                .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp_with_time_zone());
        } else {
            builder
                .engine("InnoDB")
                .character_set("utf8mb4")
                .collate("utf8mb4_0900_as_cs")
                .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp())
                .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string()).timestamp());
        }
        builder.to_owned()
    }

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        tardis_create_index_statement()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
