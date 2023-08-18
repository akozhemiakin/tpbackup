use chrono::{Datelike, Local, Timelike};
use flate2::{write::GzEncoder, Compression};
use std::fs;
use std::path::Path;
use tar::Builder;

pub fn compress_dir_to_tar_gz<P1, P2>(
    dir_path: &P1,
    output_path: &P2,
) -> Result<(), Box<dyn std::error::Error>>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    // Open the directory.
    let dir = fs::read_dir(dir_path)?;

    // Create a GzEncoder which compresses data and writes to the specified file.
    let output_file = fs::File::create(output_path)?;
    let encoder = GzEncoder::new(output_file, Compression::default());

    // Create a tar builder to package files.
    let mut tar_builder = Builder::new(encoder);

    // Iterate over the directory and add files to the tar archive.
    for entry in dir {
        let entry = entry?;
        let path = entry.path();

        // Calculate the relative path.
        let rel_path = path.strip_prefix(dir_path).unwrap_or(&path);

        if path.is_file() {
            tar_builder.append_path_with_name(&path, rel_path)?;
        }
    }

    // Finish the tar archive to flush any remaining data.
    tar_builder.into_inner()?;

    Ok(())
}

pub fn yyyymmddhhmm() -> String {
    let now = Local::now();

    format!(
        "{:04}{:02}{:02}{:02}{:02}",
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute()
    )
}

/// List of valid TargetProcess resources
// TODO: Convert to enum and allow to select subset with Cli?
pub const RESOURCES: [&str; 67] = [
    // The following one exists in API but seems to be missed in actual APi
    //"AgileReleaseTrains",
    "Assignables",
    "AssignedEfforts",
    "Assignments",
    "Attachments",
    "Bugs",
    "Builds",
    "Comments",
    "Companies",
    // Context is a special resource that should not be backed up
    //"Context",
    "CustomActivities",
    "CustomFields",
    "CustomRules",
    "EntityPermissions",
    "EntityStates",
    "EntityTypes",
    "Epics",
    "Features",
    "Generals",
    "GeneralFollowers",
    "GeneralUsers",
    "GlobalSettings",
    "Impediments",
    "InboundAssignables",
    "Iterations",
    "Messages",
    "MessageUids",
    "Milestones",
    "MilestoneProjects",
    "OutboundAssignables",
    "PortfolioEpics",
    "Priorities",
    "Processes",
    "Programs",
    "Projects",
    "ProjectMembers",
    "Relations",
    "RelationTypes",
    "Releases",
    "ReleaseProjects",
    "Requests",
    "Requesters",
    "RequestTypes",
    "Revisions",
    "RevisionFiles",
    "Roles",
    "RoleEfforts",
    "RoleEntityTypes",
    "RoleEntityTypeProcessSettings",
    "Severities",
    "Tags",
    "Tasks",
    "Teams",
    "TeamAssignments",
    "TeamIterations",
    "TeamMembers",
    "TeamProjects",
    "Terms",
    "TestCases",
    "TestCaseRuns",
    "TestPlans",
    "TestPlanRuns",
    "TestRunItemHierarchyLinks",
    "TestSteps",
    "TestStepRuns",
    "Times",
    "Users",
    "UserStories",
    "Workflows",
];
