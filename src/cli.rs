#[cfg(feature = "application")]
use crate::annotations::AnnotationMaker;
use crate::annotations::SearchQuery;
use crate::groups::{Expand, GroupFilters};
use crate::{AnnotationID, GroupID};
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
