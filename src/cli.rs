#[cfg(feature = "cli")]
use crate::annotations::AnnotationMaker;
use crate::annotations::{Order, SearchQuery, Sort};
use crate::errors::CLIError;
use crate::groups::{Expand, GroupFilters};
use crate::{AnnotationID, GroupID, Hypothesis};
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::{fs, io};
use structopt::clap::AppSettings;
use structopt::clap::Shell;
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

    /// Generate shell completions
    Complete {
        #[structopt(possible_values = & Shell::variants())]
        shell: Shell,
    },
}

#[derive(StructOpt, Debug)]
pub enum AnnotationsCommand {
    /// Create a new annotation (TODO: add Target somehow)
    Create {
        #[structopt(flatten)]
        annotation: AnnotationMaker,
        /// write created annotation to this file in JSON format
        #[structopt(parse(from_os_str), short = "o", long)]
        file: Option<PathBuf>,
    },

    /// Update an existing annotation
    Update {
        /// unique ID of the annotation to update
        id: AnnotationID,
        #[structopt(flatten)]
        annotation: AnnotationMaker,
        /// write updated annotation to this file in JSON format
        #[structopt(parse(from_os_str), short = "o", long)]
        file: Option<PathBuf>,
    },

    /// Search for annotations with optional filters
    Search {
        #[structopt(flatten)]
        query: SearchQuery,
        /// json file to write search results to, writes to stdout if not given
        #[structopt(parse(from_os_str), short = "o", long)]
        file: Option<PathBuf>,
    },
    /// Fetch annotation by ID
    Fetch {
        /// unique ID of the annotation to fetch
        id: AnnotationID,
        /// json file to write annotation to, writes to stdout if not given
        #[structopt(parse(from_os_str), short = "o", long)]
        file: Option<PathBuf>,
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
        /// json file to write filtered groups to, writes to stdout if not given
        #[structopt(parse(from_os_str), short = "o", long)]
        file: Option<PathBuf>,
    },
    /// Create a new, private group for the currently-authenticated user.
    Create {
        /// group name
        name: String,
        /// group description
        description: Option<String>,
        /// write created group to this file in JSON format
        #[structopt(parse(from_os_str), short = "o", long)]
        file: Option<PathBuf>,
    },
    /// Fetch a single Group resource.
    Fetch {
        /// unique Group ID
        id: GroupID,
        /// Expand the organization, scope, or both
        #[structopt(long, short)]
        expand: Vec<Expand>,
        /// write group to this file in JSON format
        #[structopt(parse(from_os_str), short = "o", long)]
        file: Option<PathBuf>,
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
        /// write updated group to this file in JSON format
        #[structopt(parse(from_os_str), short = "o", long)]
        file: Option<PathBuf>,
    },
    /// Fetch a list of all members (users) in a group.
    ///
    /// Returned user resource only contains public-facing user data.
    /// Authenticated user must have read access to the group. Does not require authentication for reading members of
    /// public groups. Returned members are unsorted.
    Members {
        /// unique Group ID
        id: GroupID,
        /// json file to write groups members to, writes to stdout if not given
        #[structopt(parse(from_os_str), short = "o", long)]
        file: Option<PathBuf>,
    },
    /// Remove yourself from a group.
    Leave { id: GroupID },
}

#[derive(StructOpt, Debug)]
pub enum ProfileCommand {
    /// Fetch profile information for the currently-authenticated user.
    User {
        /// json file to write user profile to, writes to stdout if not given
        #[structopt(parse(from_os_str), short = "o", long)]
        file: Option<PathBuf>,
    },
    /// Fetch the groups for which the currently-authenticated user is a member.
    Groups {
        /// json file to write groups to, writes to stdout if not given
        #[structopt(parse(from_os_str), short = "o", long)]
        file: Option<PathBuf>,
    },
}

impl HypothesisCLI {
    pub fn run(self, client: Hypothesis) -> color_eyre::Result<()> {
        match self {
            HypothesisCLI::Annotations { cmd } => match cmd {
                AnnotationsCommand::Create { annotation, file } => {
                    let annotation = client.create_annotation(&annotation)?;
                    println!("Annotation {} created", annotation.id);
                    if let Some(file) = file {
                        let writer: Box<dyn io::Write> = Box::new(fs::File::open(file)?);
                        let mut buffered = io::BufWriter::new(writer);
                        writeln!(buffered, "{}", serde_json::to_string(&annotation)?)?;
                    }
                }
                AnnotationsCommand::Update {
                    id,
                    annotation,
                    file,
                } => {
                    let annotation = client.update_annotation(&id, &annotation)?;
                    println!("Annotation {} updated", annotation.id);
                    if let Some(file) = file {
                        let writer: Box<dyn io::Write> = Box::new(fs::File::open(file)?);
                        let mut buffered = io::BufWriter::new(writer);
                        writeln!(buffered, "{}", serde_json::to_string(&annotation)?)?;
                    }
                }
                AnnotationsCommand::Search { query, file } => {
                    let annotations = client.search_annotations(&query)?;
                    let writer: Box<dyn io::Write> = match file {
                        Some(file) => Box::new(fs::File::open(file)?),
                        None => Box::new(io::stdout()),
                    };
                    let mut buffered = io::BufWriter::new(writer);
                    for annotation in annotations {
                        writeln!(buffered, "{}", serde_json::to_string(&annotation)?)?;
                    }
                }
                AnnotationsCommand::Fetch { id, file } => {
                    let annotation = client.fetch_annotation(&id)?;
                    let writer: Box<dyn io::Write> = match file {
                        Some(file) => Box::new(fs::File::open(file)?),
                        None => Box::new(io::stdout()),
                    };
                    let mut buffered = io::BufWriter::new(writer);
                    writeln!(buffered, "{}", serde_json::to_string(&annotation)?)?;
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
                GroupsCommand::List { filters, file } => {
                    let groups = client.get_groups(&filters)?;
                    let writer: Box<dyn io::Write> = match file {
                        Some(file) => Box::new(fs::File::open(file)?),
                        None => Box::new(io::stdout()),
                    };
                    let mut buffered = io::BufWriter::new(writer);
                    for group in groups {
                        writeln!(buffered, "{}", serde_json::to_string(&group)?)?;
                    }
                }
                GroupsCommand::Create {
                    name,
                    description,
                    file,
                } => {
                    let group = client.create_group(&name, description.as_deref())?;
                    println!("Created Group {}", group.id);
                    if let Some(file) = file {
                        let writer: Box<dyn io::Write> = Box::new(fs::File::open(file)?);
                        let mut buffered = io::BufWriter::new(writer);
                        writeln!(buffered, "{}", serde_json::to_string(&group)?)?;
                    }
                }
                GroupsCommand::Fetch { id, expand, file } => {
                    let group = client.fetch_group(&id, expand)?;
                    let writer: Box<dyn io::Write> = match file {
                        Some(file) => Box::new(fs::File::open(file)?),
                        None => Box::new(io::stdout()),
                    };
                    let mut buffered = io::BufWriter::new(writer);
                    writeln!(buffered, "{}", serde_json::to_string(&group)?)?;
                }
                GroupsCommand::Update {
                    id,
                    name,
                    description,
                    file,
                } => {
                    let group =
                        client.update_group(&id, name.as_deref(), description.as_deref())?;
                    println!("Updated group {}", group.id);
                    if let Some(file) = file {
                        let writer: Box<dyn io::Write> = Box::new(fs::File::open(file)?);
                        let mut buffered = io::BufWriter::new(writer);
                        writeln!(buffered, "{}", serde_json::to_string(&group)?)?;
                    }
                }
                GroupsCommand::Members { id, file } => {
                    let members = client.get_group_members(&id)?;
                    let writer: Box<dyn io::Write> = match file {
                        Some(file) => Box::new(fs::File::open(file)?),
                        None => Box::new(io::stdout()),
                    };
                    let mut buffered = io::BufWriter::new(writer);
                    for member in members {
                        writeln!(buffered, "{}", serde_json::to_string(&member)?)?;
                    }
                }
                GroupsCommand::Leave { id } => {
                    client.leave_group(&id)?;
                    println!("You've left Group {}", id);
                }
            },
            HypothesisCLI::Profile { cmd } => match cmd {
                ProfileCommand::User { file } => {
                    let profile = client.fetch_user_profile()?;
                    let writer: Box<dyn io::Write> = match file {
                        Some(file) => Box::new(fs::File::open(file)?),
                        None => Box::new(io::stdout()),
                    };
                    let mut buffered = io::BufWriter::new(writer);
                    writeln!(buffered, "{}", serde_json::to_string(&profile)?)?;
                }
                ProfileCommand::Groups { file } => {
                    let groups = client.fetch_user_groups()?;
                    let writer: Box<dyn io::Write> = match file {
                        Some(file) => Box::new(fs::File::open(file)?),
                        None => Box::new(io::stdout()),
                    };
                    let mut buffered = io::BufWriter::new(writer);
                    for group in groups {
                        writeln!(buffered, "{}", serde_json::to_string(&group)?)?;
                    }
                }
            },
            HypothesisCLI::Complete { shell } => {
                // Generates shell completions
                HypothesisCLI::clap().gen_completions_to("hypothesis", shell, &mut io::stdout());
            }
        }
        Ok(())
    }
}

impl FromStr for Sort {
    type Err = CLIError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "created" => Ok(Sort::Created),
            "updated" => Ok(Sort::Updated),
            "id" => Ok(Sort::Id),
            "group" => Ok(Sort::Group),
            "user" => Ok(Sort::User),
            _ => Err(CLIError::ParseError {
                name: "sort".into(),
                types: vec![
                    "created".into(),
                    "updated".into(),
                    "id".into(),
                    "group".into(),
                    "user".into(),
                ],
            }),
        }
    }
}

impl Sort {
    /// A list of possible variants in `&'static str` form
    pub fn variants() -> [&'static str; 5] {
        ["created", "updated", "id", "group", "user"]
    }
}

impl FromStr for Order {
    type Err = CLIError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "desc" => Ok(Order::Desc),
            "asc" => Ok(Order::Asc),
            _ => Err(CLIError::ParseError {
                name: "order".into(),
                types: vec!["asc".into(), "desc".into()],
            }),
        }
    }
}

impl Order {
    /// A list of possible variants in `&'static str` form
    pub fn variants() -> [&'static str; 2] {
        ["asc", "desc"]
    }
}

impl FromStr for Expand {
    type Err = CLIError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "organization" => Ok(Expand::Organization),
            "scopes" => Ok(Expand::Scopes),
            _ => Err(CLIError::ParseError {
                name: "expand".into(),
                types: vec!["organization".into(), "scopes".into()],
            }),
        }
    }
}

impl Expand {
    /// A list of possible variants in `&'static str` form
    pub fn variants() -> [&'static str; 2] {
        ["organization", "scopes"]
    }
}