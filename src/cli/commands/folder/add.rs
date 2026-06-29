use crate::{
    Application,
    app::FolderService,
    models::{Folder, FolderColor},
};

/// `add` command implementation for folder subcommand
pub fn run(app: &mut Application, name: String, color: FolderColor) -> color_eyre::Result<()> {
    let new_folder: Folder = Folder::new(name, color);
    let result = FolderService::append_folder(&mut app.data.folders, new_folder)?;

    app.save_all()?;
    let added: Folder = result.unwrap_folder_created();
    println!(
        "Folder `{}` with color {} was added and saved successfully!",
        added.name, added.color
    );

    Ok(())
}
