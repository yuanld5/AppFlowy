use crate::{
    errors::WorkspaceError,
    event::WorkspaceEvent,
    handlers::*,
    services::{server::construct_workspace_server, AppController, ViewController, WorkspaceController},
};

use crate::services::TrashController;
use flowy_database::DBConnection;
use flowy_dispatch::prelude::*;
use flowy_document::module::FlowyDocument;
use flowy_net::config::ServerConfig;
use flowy_sqlite::ConnectionPool;
use std::sync::Arc;

pub trait WorkspaceDeps: WorkspaceUser + WorkspaceDatabase {}

pub trait WorkspaceUser: Send + Sync {
    fn user_id(&self) -> Result<String, WorkspaceError>;
    fn token(&self) -> Result<String, WorkspaceError>;
}

pub trait WorkspaceDatabase: Send + Sync {
    fn db_pool(&self) -> Result<Arc<ConnectionPool>, WorkspaceError>;

    fn db_connection(&self) -> Result<DBConnection, WorkspaceError> {
        let pool = self.db_pool()?;
        let conn = pool.get().map_err(|e| WorkspaceError::internal().context(e))?;
        Ok(conn)
    }
}

pub fn mk_workspace(
    user: Arc<dyn WorkspaceUser>,
    database: Arc<dyn WorkspaceDatabase>,
    flowy_document: Arc<FlowyDocument>,
    server_config: &ServerConfig,
) -> Arc<WorkspaceController> {
    let server = construct_workspace_server(server_config);

    let trash_controller = Arc::new(TrashController::new());

    let view_controller = Arc::new(ViewController::new(
        user.clone(),
        database.clone(),
        server.clone(),
        flowy_document,
    ));

    let app_controller = Arc::new(AppController::new(user.clone(), database.clone(), server.clone()));

    let workspace_controller = Arc::new(WorkspaceController::new(
        user.clone(),
        database.clone(),
        app_controller.clone(),
        view_controller.clone(),
        trash_controller.clone(),
        server.clone(),
    ));
    workspace_controller
}

pub fn create(workspace: Arc<WorkspaceController>) -> Module {
    let mut module = Module::new()
        .name("Flowy-Workspace")
        .data(workspace.clone())
        .data(workspace.app_controller.clone())
        .data(workspace.view_controller.clone())
        .data(workspace.trash_controller.clone())
        .event(WorkspaceEvent::InitWorkspace, init_workspace_handler);

    module = module
        .event(WorkspaceEvent::CreateWorkspace, create_workspace_handler)
        .event(WorkspaceEvent::ReadCurWorkspace, read_cur_workspace_handler)
        .event(WorkspaceEvent::ReadWorkspaces, read_workspaces_handler)
        .event(WorkspaceEvent::OpenWorkspace, open_workspace_handler)
        .event(WorkspaceEvent::ReadWorkspaceApps, read_workspace_apps_handler);

    module = module
        .event(WorkspaceEvent::CreateApp, create_app_handler)
        .event(WorkspaceEvent::ReadApp, read_app_handler)
        .event(WorkspaceEvent::UpdateApp, update_app_handler)
        .event(WorkspaceEvent::DeleteApp, delete_app_handler);

    module = module
        .event(WorkspaceEvent::CreateView, create_view_handler)
        .event(WorkspaceEvent::ReadView, read_view_handler)
        .event(WorkspaceEvent::UpdateView, update_view_handler)
        .event(WorkspaceEvent::DeleteView, delete_view_handler)
        .event(WorkspaceEvent::OpenView, open_view_handler)
        .event(WorkspaceEvent::ApplyDocDelta, apply_doc_delta_handler);

    module = module.event(WorkspaceEvent::ReadTrash, read_trash_handler);

    module
}
