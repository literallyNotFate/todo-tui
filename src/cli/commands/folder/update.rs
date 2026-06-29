use crate::{
    Application,
    app::{FolderService, OperationResult},
    cli::types::EntitySelector,
    models::{FolderColor, FolderEditor},
};

/// `update` command implementation for folder subcommand
pub fn run(
    app: &mut Application,
    id_query: String,
    name: Option<String>,
    color: Option<FolderColor>,
) -> color_eyre::Result<()> {
    EntitySelector::find(&app.data.folders, &id_query).execute(&id_query, |folder| {
        let new_name: String = name.clone().unwrap_or_else(|| folder.name);
        let new_color: FolderColor = color.unwrap_or_else(|| folder.color);
        let editor: FolderEditor = FolderEditor::new(new_name, new_color);

        let result = FolderService::update_folder(&mut app.data.folders, &folder.id, editor)?;
        app.save_all()?;

        if let OperationResult::FolderUpdated { old, new } = result {
            println!("Folder updated:");
            if old.name != new.name {
                println!("   Name: '{}' -> '{}'", old.name, new.name);
            }
            if old.color != new.color {
                println!("   Color:  '{}' -> '{}'", old.color, new.color);
            }
        }

        Ok(())
    })
}
