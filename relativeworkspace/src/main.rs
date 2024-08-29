use std::process;

use hyprland::{
    data::{Monitor, Workspace, Workspaces},
    dispatch,
    dispatch::{Dispatch, DispatchType},
    shared::{HyprData, HyprDataActive, HyprError},
};

fn parse_argument() -> i32 {
    let arguments = &*std::env::args().skip(1).collect::<Vec<_>>().join(" ");

    match arguments {
        "previous" => -1,
        "next" => 1,
        _ => {
            eprintln!("Invalid argument: {}", arguments);
            process::exit(1);
        }
    }
}

fn workspace_ids() -> Result<Vec<i32>, HyprError> {
    let active_monitor = Monitor::get_active()?.name;
    let workspaces = Workspaces::get()?;

    let functional_workspaces = workspaces
        .iter()
        .filter(|workspace| workspace.monitor == active_monitor && workspace.id >= 0)
        .collect::<Vec<_>>();

    let ids = functional_workspaces
        .iter()
        .map(|workspace| workspace.id)
        .collect::<Vec<_>>();

    Ok(ids)
}

fn dispatch(identifier: i32) -> Result<(), HyprError> {
    let current_workspace = Workspace::get_active()?;

    let is_first = current_workspace.id == 1;
    let is_max = current_workspace.id == *workspace_ids()?.iter().max().unwrap();
    let is_min = current_workspace.id == *workspace_ids()?.iter().min().unwrap();
    let is_empty = current_workspace.windows == 0;
    let multiple_workspaces = workspace_ids()?.len() > 1;

    let relative_monitor_including_empty = |id| {
        dispatch!(
            Workspace,
            dispatch::WorkspaceIdentifierWithSpecial::RelativeMonitorIncludingEmpty(id)
        )
    };

    let relative_monitor = |id| {
        dispatch!(
            Workspace,
            dispatch::WorkspaceIdentifierWithSpecial::RelativeMonitor(id)
        )
    };

    match (is_first, identifier) {
        // If the workspace is the first, we never go previous.
        (true, 1) => {
            // We are on the only workspace, so we can go to the next empty one.
            if !multiple_workspaces {
                relative_monitor_including_empty(1)
            // There's another workspace to jump to.
            } else {
                relative_monitor(1)
            }
        }
        // We can go back to the previous workspace if we are not on the first one.
        (false, -1) => {
            // We are only the only workspace workspace and it is not the first,
            // so we can go to the previous empty one.
            if !multiple_workspaces {
                relative_monitor_including_empty(-1)
            // We jump to the previous workspace because we know there are multiple
            // workspaces and we are not on the first one.
            } else if !is_min {
                relative_monitor(-1)
            // Otherwise, we are still not on the first workspace, but there
            // are no previous workspaces to jump to, so we jump to the previous empty one.
            } else {
                relative_monitor_including_empty(-1)
            }
        }
        // We can also go to the next workspace if we are not on the last one.
        (false, 1) => {
            // We know there is no other workspaces, so we can go to the next empty one.
            if !multiple_workspaces {
                relative_monitor_including_empty(1)
            // If there are multiple workspaces and we are not on the last one,
            // we can jump to the next open workspace.
            } else if !is_max {
                relative_monitor(1)
            // We only want to go to the next empty workspace if we are already on an empty one.
            } else if is_empty {
                relative_monitor_including_empty(1)
            // Otherwise, we know there are multiple workspaces and we are on a non-empty one,
            // so we don't want to make any changes.
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }?;

    Ok(())
}

fn main() -> Result<(), HyprError> {
    let identifier = parse_argument();

    dispatch(identifier)?;

    Ok(())
}
