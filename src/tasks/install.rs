//! Overall hierarchy for installing a installation of the application.

use std::collections::HashMap;

use crate::installer::InstallerFramework;

use crate::sources::types::VersionTarget;
use crate::tasks::ensure_only_instance::EnsureOnlyInstanceTask;
use crate::tasks::install_dir::VerifyInstallDirTask;
use crate::tasks::install_global_shortcut::InstallGlobalShortcutsTask;
use crate::tasks::install_pkg::InstallPackageTask;
use crate::tasks::launch_installed_on_exit::LaunchOnExitTask;
use crate::tasks::remove_target_dir::RemoveTargetDirTask;
use crate::tasks::save_executable::SaveExecutableTask;
use crate::tasks::uninstall_pkg::UninstallPackageTask;

use crate::tasks::Task;
use crate::tasks::TaskDependency;
use crate::tasks::TaskMessage;
use crate::tasks::TaskOrdering;
use crate::tasks::TaskParamType;

pub struct InstallTask {
    pub items: HashMap<String, VersionTarget>,
    pub uninstall_items: Vec<String>,
    pub fresh_install: bool,
    pub create_desktop_shortcuts: bool,
    // force_install: remove the target directory before installing
    pub force_install: bool,
}

impl Task for InstallTask {
    fn execute(
        &mut self,
        _: Vec<TaskParamType>,
        _: &mut InstallerFramework,
        messenger: &dyn Fn(&TaskMessage),
    ) -> Result<TaskParamType, String> {
        messenger(&TaskMessage::DisplayMessage("Wrapping up...", 0.0));
        Ok(TaskParamType::None)
    }

    fn dependencies(&self) -> Vec<TaskDependency> {
        let mut elements = Vec::new();

        elements.push(TaskDependency::build(
            TaskOrdering::Pre,
            Box::new(EnsureOnlyInstanceTask {}),
        ));

        if self.force_install {
            elements.push(TaskDependency::build(
                TaskOrdering::Pre,
                Box::new(RemoveTargetDirTask {}),
            ));
        }

        elements.push(TaskDependency::build(
            TaskOrdering::Pre,
            Box::new(VerifyInstallDirTask {
                clean_install: self.fresh_install,
            }),
        ));

        for item in &self.uninstall_items {
            elements.push(TaskDependency::build(
                TaskOrdering::Pre,
                Box::new(UninstallPackageTask {
                    name: item.clone(),
                    optional: false,
                }),
            ));
        }

        for item in &self.items {
            elements.push(TaskDependency::build(
                TaskOrdering::Pre,
                Box::new(InstallPackageTask {
                    name: item.0.to_string(),
                    version_target: item.1.clone(),
                    create_desktop_shortcuts: self.create_desktop_shortcuts,
                }),
            ));
        }

        if self.fresh_install {
            elements.push(TaskDependency::build(
                TaskOrdering::Pre,
                Box::new(SaveExecutableTask {}),
            ));

            elements.push(TaskDependency::build(
                TaskOrdering::Pre,
                Box::new(InstallGlobalShortcutsTask {}),
            ));

            elements.push(TaskDependency::build(
                TaskOrdering::Post,
                Box::new(LaunchOnExitTask {}),
            ))
        }

        elements
    }

    fn name(&self) -> String {
        "InstallTask".to_string()
    }
}
