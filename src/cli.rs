#[cfg(feature = "application")]
use crate::annotations::AnnotationMaker;
use crate::annotations::SearchQuery;
use crate::groups::{Expand, GroupFilters};
use crate::{AnnotationID, GroupID, Hypothesis};
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
name = "hypothesis",
about = "Call the Hypothesis API from the comfort of your terminal",
rename_all = "kebab-case",
global_settings = & [AppSettings::DeriveDisplayOrder]
)]
pub enum HypothesisCLI {
    /// Manage annotations
    Annotations {
        #[structopt(subcommand)]
        cmd: AnnotationsCommand,
    },
    /// Manage groups
    Groups {
        #[structopt(subcommand)]
        cmd: GroupsCommand,
    },

    /// Manage user profile
    Profile {
        #[structopt(subcommand)]
        cmd: ProfileCommand,
    },
}

#[derive(StructOpt, Debug)]
pub enum AnnotationsCommand {
    /// Create a new annotation (TODO: add Target somehow)
    Create {
        #[structopt(flatten)]
        annotation: AnnotationMaker,
    },

    /// Update an existing annotation
    Update {
        /// unique ID of the annotation to update
        id: AnnotationID,
        #[structopt(flatten)]
        annotation: AnnotationMaker,
    },

    /// Search for annotations with optional filters
    Search {
        #[structopt(flatten)]
        query: SearchQuery,
    },
    /// Fetch annotation by ID
    Fetch {
        /// unique ID of the annotation to fetch
        id: AnnotationID,
    },
    /// Delete annotation by ID
    Delete {
        /// unique ID of the annotation to delete
        id: AnnotationID,
    },
    /// Flag an annotation
    ///
    /// Flag an annotation for review (moderation). The moderator of the group containing the
    /// annotation will be notified of the flag and can decide whether or not to hide the
    /// annotation. Note that flags persist and cannot be removed once they are set.
    Flag {
        /// unique ID of the annotation to flag
        id: AnnotationID,
    },
    /// Hide an annotation
    ///
    /// Hide an annotation. The authenticated user needs to have the moderate permission for the
    /// group that contains the annotation — this permission is granted to the user who created the group.
    Hide {
        /// unique ID of the annotation to hide
        id: AnnotationID,
    },
    /// Show an annotation
    ///
    /// Show/"un-hide" an annotation. The authenticated user needs to have the moderate permission
    /// for the group that contains the annotation—this permission is granted to the user who created the group.
    Show {
        /// unique ID of the annotation to show
        id: AnnotationID,
    },
}

#[derive(StructOpt, Debug)]
pub enum GroupsCommand {
    /// Retrieve a list of applicable Groups, filtered by authority and target document (document_uri).
    /// Also retrieve user's private Groups.
    List {
        #[structopt(flatten)]
        filters: GroupFilters,
    },
    /// Create a new, private group for the currently-authenticated user.
    Create {
        /// group name
        name: String,
        /// group description
        description: Option<String>,
    },
    /// Fetch a single Group resource.
    Fetch {
        /// unique Group ID
        id: GroupID,
        /// Expand the organization, scope, or both
        #[structopt(long, short)]
        expand: Vec<Expand>,
    },
    /// Update a Group resource.
    Update {
        /// unique Group ID
        id: GroupID,
        /// new group name
        #[structopt(long, short)]
        name: Option<String>,
        /// new group description
        #[structopt(long, short)]
        description: Option<String>,
    },
    /// Fetch a list of all members (users) in a group.
    ///
    /// Returned user resource only contains public-facing user data.
    /// Authenticated user must have read access to the group. Does not require authentication for reading members of
    /// public groups. Returned members are unsorted.
    Members {
        /// unique Group ID
        id: GroupID,
    },
    /// Remove yourself from a group.
    Leave { id: GroupID },
}

#[derive(StructOpt, Debug)]
pub enum ProfileCommand {
    /// Fetch profile information for the currently-authenticated user.
    User,
    /// Fetch the groups for which the currently-authenticated user is a member.
    Groups,
}

impl HypothesisCLI {
    pub fn run(self, client: &Hypothesis) -> color_eyre::Result<()> {
        match self {
            HypothesisCLI::Annotations { cmd } => match cmd {
                AnnotationsCommand::Create { annotation } => {
                    let annotation = client.create_annotation(&annotation)?;
                    println!("Annotation {} created", annotation.id);
                }
                AnnotationsCommand::Update { id, annotation } => {
                    let annotation = client.update_annotation(&id, &annotation)?;
                    println!("Annotation {} updated", annotation.id);
                }
                AnnotationsCommand::Search { query } => {
                    let annotations = client.search_annotations(&query)?;
                }
                AnnotationsCommand::Fetch { id } => {
                    let annotation = client.fetch_annotation(&id)?;
                }
                AnnotationsCommand::Delete { id } => {
                    let deleted = client.delete_annotation(&id)?;
                    if deleted {
                        println!("Annotation {} deleted", id);
                    } else {
                        println!("Couldn't delete annotation {}", id);
                    }
                }
                AnnotationsCommand::Flag { id } => {
                    client.flag_annotation(&id)?;
                    println!("Annotation {} flagged", id);
                }
                AnnotationsCommand::Hide { id } => {
                    client.hide_annotation(&id)?;
                    println!("Annotation {} hidden", id);
                }
                AnnotationsCommand::Show { id } => {
                    client.show_annotation(&id)?;
                    println!("Annotation {} unhidden", id);
                }
            },
            HypothesisCLI::Groups { cmd } => match cmd {
                GroupsCommand::List { filters } => {
                    let groups = client.get_groups(&filters)?;
                }
                GroupsCommand::Create { name, description } => {
                    let group = client.create_group(&name, description.as_deref())?;
                }
                GroupsCommand::Fetch { id, expand } => {
                    let group = client.fetch_group(&id, expand)?;
                }
                GroupsCommand::Update {
                    id,
                    name,
                    description,
                } => {
                    let group =
                        client.update_group(&id, name.as_deref(), description.as_deref())?;
                }
                GroupsCommand::Members { id } => {
                    let members = client.get_group_members(&id)?;
                }
                GroupsCommand::Leave { id } => {
                    client.leave_group(&id)?;
                }
            },
            HypothesisCLI::Profile { cmd } => match cmd {
                ProfileCommand::User => {
                    let profile = client.fetch_user_profile()?;
                }
                ProfileCommand::Groups => {
                    let groups = client.fetch_user_groups()?;
                }
            },
        }
        Ok(())
    }
}
