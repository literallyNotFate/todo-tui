use crate::{
    app::OperationResult,
    core::FolderError,
    models::{Folder, FolderEditor},
    state::ApplicationResult,
};
use uuid::Uuid;

/// Main service methods (only for folders)
pub struct FolderService;

impl FolderService {
    /// Append new folder to the end of list
    pub fn append_folder(
        folders: &mut Vec<Folder>,
        folder: Folder,
    ) -> ApplicationResult<OperationResult> {
        if folder.name.trim().is_empty() {
            log::debug!("Validation error on append: Folder name is empty");
            return Err(FolderError::EmptyName.into());
        }

        if folders.iter().any(|f| f.name.trim() == folder.name.trim()) {
            log::debug!(
                "Validation error on append: Folder name '{}' already exists",
                folder.name
            );
            return Err(FolderError::DuplicateName.into());
        }

        let name = folder.name.clone();
        log::info!("Adding new folder: '{}' (ID: {})", name, folder.id);

        folders.push(folder.clone());
        let index = folders.len() - 1;

        Ok(OperationResult::FolderCreated { index, folder })
    }

    /// Update folder by id using FolderEditor model
    pub fn update_folder(
        folders: &mut [Folder],
        id: &Uuid,
        editor: FolderEditor,
    ) -> ApplicationResult<OperationResult> {
        if editor.name.trim().is_empty() {
            log::debug!("Validation error on update: Folder name is empty");
            return Err(FolderError::EmptyName.into());
        }

        let original_index = folders.iter().position(|f| f.id == *id).ok_or_else(|| {
            log::warn!("Update failed: Folder with ID {} not found", id);
            FolderError::FolderNotFound
        })?;

        if folders
            .iter()
            .enumerate()
            .any(|(i, f)| i != original_index && f.name.trim() == editor.name.trim())
        {
            log::debug!(
                "Validation error on update: Folder name '{}' already exists",
                editor.name
            );
            return Err(FolderError::DuplicateName.into());
        }

        let old = folders[original_index].clone();
        folders[original_index].update_from_editor(editor);
        let new = folders[original_index].clone();

        log::info!(
            "Folder updated successfully: '{}' (ID: {}). Changes: [Name: '{}' -> '{}', Color: {:?} -> {:?}]",
            new.name,
            id,
            old.name,
            new.name,
            old.color,
            new.color
        );

        Ok(OperationResult::FolderUpdated {
            index: original_index,
            old,
            new,
        })
    }

    /// Remove folder by id
    pub fn remove_folder(
        folders: &mut Vec<Folder>,
        id: &Uuid,
    ) -> ApplicationResult<OperationResult> {
        let index = folders.iter().position(|f| f.id == *id).ok_or_else(|| {
            log::error!(
                "Remove failed: Attempted to remove non-existent folder ID {}",
                id
            );
            FolderError::FolderNotFound
        })?;

        let folder = folders.remove(index);
        log::info!("Folder removed: '{}' (ID: {})", folder.name, id);

        Ok(OperationResult::FolderRemoved { folder })
    }
}

/// Unit-tests for folder service
#[cfg(test)]
mod folder_tests {
    use super::*;
    use crate::{core::ApplicationError, models::FolderColor};

    #[test]
    fn should_append_folder_service() {
        let mut folders: Vec<Folder> = Vec::new();
        let folder_to_add = Folder::new("Work", &FolderColor::Red.to_string());
        let new_title = folder_to_add.name.clone();

        let result = FolderService::append_folder(&mut folders, folder_to_add);
        assert!(result.is_ok());
        assert_eq!(folders.len(), 1);
        assert_eq!(result.unwrap().entity_title(), new_title);
    }

    #[test]
    fn should_fail_append_folder_on_empty_name() {
        let mut folders: Vec<Folder> = Vec::new();
        let folder_to_add = Folder::new("   ", "Blue");

        let result = FolderService::append_folder(&mut folders, folder_to_add);

        assert!(matches!(
            result,
            Err(ApplicationError::Folder(FolderError::EmptyName))
        ));
        assert!(folders.is_empty());
    }

    #[test]
    fn should_fail_append_folder_on_duplicate_name() {
        let mut folders: Vec<Folder> = vec![Folder::new("Personal", "Red")];
        let duplicate = Folder::new("Personal", "Red");

        let result = FolderService::append_folder(&mut folders, duplicate);

        assert!(matches!(
            result,
            Err(ApplicationError::Folder(FolderError::DuplicateName))
        ));
        assert_eq!(folders.len(), 1);
    }

    #[test]
    fn should_update_folder_service() {
        let mut folders = vec![Folder::new("Old Name", "Blue")];
        let id = folders[0].id;

        let editor = FolderEditor {
            name: "New Name".into(),
            color: FolderColor::Green,
        };

        let result = FolderService::update_folder(&mut folders, &id, editor);
        assert!(result.is_ok());

        let (index, old, new) = result.unwrap().unwrap_folder_updated();

        assert_eq!(index, 0);
        assert_eq!(old.name, "Old Name");
        assert_eq!(new.name, "New Name");
        assert_eq!(new.color, FolderColor::Green.to_string());
        assert_eq!(folders[0].name, "New Name");
    }

    #[test]
    fn should_fail_update_folder_on_duplicate() {
        let mut folders = vec![
            Folder::new("First", "Red"),
            Folder::new("Second", "Lavender"),
        ];
        let id_to_update = folders[1].id;

        let editor = FolderEditor {
            name: "First".into(),
            color: FolderColor::Blue,
        };

        let result = FolderService::update_folder(&mut folders, &id_to_update, editor);

        assert!(matches!(
            result,
            Err(ApplicationError::Folder(FolderError::DuplicateName))
        ));
        assert_eq!(folders[1].name, "Second");
    }

    #[test]
    fn should_remove_folder_service() {
        let mut folders = vec![
            Folder::new("Trash", "Red"),
            Folder::new("Archive", "Magenta"),
        ];
        let id_to_remove = folders[0].id;

        let result = FolderService::remove_folder(&mut folders, &id_to_remove);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().entity_title(), "Trash");
        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0].name, "Archive");
    }
}
