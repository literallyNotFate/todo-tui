use crate::{
    Application,
    app::{FolderService, OperationResult},
    cli::types::EntitySelector,
};

/// `remove` command implementation for folder subcommand
pub fn run(app: &mut Application, id_query: String) -> color_eyre::Result<()> {
    EntitySelector::find(&app.data.folders, &id_query).execute(&id_query, |folder| {
        let result = FolderService::remove_folder(&mut app.data.folders, &folder.id)?;
        app.save_all()?;

        if let OperationResult::FolderRemoved { folder } = result {
            println!(
                "Folder '{}' has been permanently removed with all its tasks!",
                folder.name,
            );
        }

        Ok(())
    })
}
