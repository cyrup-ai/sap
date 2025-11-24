use super::Configurable;
use crate::app::Cli;
use crate::config_file::Config;
use crate::print_error;

use std::convert::TryFrom;

/// A struct to hold a [Vec] of [Block]s and to provide methods to create it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blocks(pub Vec<Block>);

impl Blocks {
    /// Returns a Blocks struct for the long format.
    ///
    /// It contains the [Block]s [Permission](Block::Permission), [User](Block::User),
    /// [Group](Block::Group), [Size](Block::Size), [Date](Block::Date) and [Name](Block::Name).
    fn long() -> Self {
        Self(vec![
            Block::Permission,
            Block::User,
            Block::Group,
            Block::Size,
            Block::Date,
            Block::Name,
        ])
    }

    /// Checks whether `self` already contains a [Block] of variant [INode](Block::INode).
    fn contains_inode(&self) -> bool {
        self.0.contains(&Block::INode)
    }

    /// Prepends a [Block] of variant [INode](Block::INode) to `self`.
    fn prepend_inode(&mut self) {
        self.0.insert(0, Block::INode);
    }

    /// Prepends a [Block] of variant [INode](Block::INode), if `self` does not already contain a
    /// Block of that variant.
    fn optional_prepend_inode(&mut self) {
        if !self.contains_inode() {
            self.prepend_inode()
        }
    }

    #[allow(dead_code)] // Used by old code path
    pub fn displays_size(&self) -> bool {
        self.0.contains(&Block::Size)
    }

    /// Inserts a [Block] of variant [Context](Block::Context), if `self` does not already contain a
    /// [Block] of that variant. The positioning will be a best-effort approximation of coreutils
    /// ls position for a security context.
    fn optional_insert_context(&mut self) {
        if self.0.contains(&Block::Context) {
            return;
        }
        let pos = self
            .0
            .iter()
            .position(|elem| *elem == Block::Group)
            .or_else(|| self.0.iter().position(|elem| *elem == Block::User));
        match pos {
            Some(pos) => self.0.insert(pos + 1, Block::Context),
            None => self.0.insert(0, Block::Context),
        }
    }

    /// Checks whether `self` already contains a [Block] of variant [GitStatus](Block::GitStatus).
    fn contains_git_status(&self) -> bool {
        self.0.contains(&Block::GitStatus)
    }

    /// Inserts a [Block] of variant [GitStatus](Block::GitStatus) to the left of [Block::Name] in `self`.
    fn add_git_status(&mut self) {
        if let Some(position) = self.0.iter().position(|&b| b == Block::Name) {
            self.0.insert(position, Block::GitStatus);
        } else {
            self.0.push(Block::GitStatus);
        }
    }

    /// Inserts a [Block] of variant [GitStatus](Block::GitStatus), if `self` does not already contain a
    /// Block of that variant.
    fn optional_add_git_status(&mut self) {
        if !self.contains_git_status() {
            self.add_git_status()
        }
    }
}

impl Configurable<Self> for Blocks {
    /// Returns a value from either [Cli], a [Config] or a default value.
    /// Unless the "long" argument is passed, this returns [Default::default]. Otherwise the first
    /// value, that is not [None], is used. The order of precedence for the value used is:
    /// - [from_cli](Blocks::from_cli)
    /// - [from_config](Blocks::from_config)
    /// - [long](Blocks::long)
    ///
    /// No matter if the "long" argument was passed, if the "inode" argument is passed and the
    /// `Blocks` does not contain a [Block] of variant [INode](Block::INode) yet, one is prepended
    /// to the returned value.
    fn configure_from(cli: &Cli, config: &Config) -> Self {
        let mut blocks = if cli.long {
            Self::long()
        } else {
            Default::default()
        };

        if cli.long
            && let Some(value) = Self::from_config(config) {
                blocks = value;
            }

        if let Some(value) = Self::from_cli(cli) {
            blocks = value;
        }

        if cli.context {
            blocks.optional_insert_context();
        }
        if cli.inode {
            blocks.optional_prepend_inode();
        }

        if cli.git && cli.long {
            blocks.optional_add_git_status();
        }

        blocks
    }

    /// Get a potential `Blocks` struct from [Cli].
    ///
    /// If the "blocks" argument is passed, then this returns a `Blocks` containing the parameter
    /// values in a [Some]. Otherwise this returns [None].
    fn from_cli(cli: &Cli) -> Option<Self> {
        if cli.blocks.is_empty() {
            return None;
        }

        let blocks: Vec<Block> = cli
            .blocks
            .iter()
            .filter_map(|b| {
                match Block::try_from(b.as_str()) {
                    Ok(block) => Some(block),
                    Err(e) => {
                        eprintln!("Warning: {}", e);
                        None
                    }
                }
            })
            .collect();
        
        if blocks.is_empty() {
            None
        } else {
            Some(Self(blocks))
        }
    }

    /// Get a potential `Blocks` struct from a [Config].
    ///
    /// If the [Config] contains an array of blocks values,
    /// its [String] values is returned as `Blocks` in a [Some].
    /// Otherwise it returns [None].
    fn from_config(config: &Config) -> Option<Self> {
        if let Some(c) = &config.blocks {
            let mut blocks: Vec<Block> = Vec::with_capacity(c.len());
            for b in c.iter() {
                match Block::try_from(b.as_str()) {
                    Ok(block) => blocks.push(block),
                    Err(err) => print_error!("{}.", err),
                }
            }
            if blocks.is_empty() {
                None
            } else {
                Some(Self(blocks))
            }
        } else {
            None
        }
    }
}

/// The default value for `Blocks` contains a [Vec] of [Name](Block::Name).
impl Default for Blocks {
    fn default() -> Self {
        Self(vec![Block::Name])
    }
}

/// A block of data to show.
#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Block {
    Permission,
    User,
    Group,
    Context,
    Size,
    SizeValue,
    Date,
    Name,
    INode,
    Links,
    GitStatus,
}

impl Block {
    pub fn get_header(&self) -> &'static str {
        match self {
            Block::INode => "INode",
            Block::Links => "Links",
            Block::Permission => "Permissions",
            Block::User => "User",
            Block::Group => "Group",
            Block::Context => "Context",
            Block::Size => "Size",
            Block::SizeValue => "SizeValue",
            Block::Date => "Date Modified",
            Block::Name => "Name",
            Block::GitStatus => "Git",
        }
    }
}

impl TryFrom<&str> for Block {
    type Error = String;

    fn try_from(string: &str) -> Result<Self, Self::Error> {
        match string {
            "permission" => Ok(Self::Permission),
            "user" => Ok(Self::User),
            "group" => Ok(Self::Group),
            "context" => Ok(Self::Context),
            "size" => Ok(Self::Size),
            "size_value" => Ok(Self::SizeValue),
            "date" => Ok(Self::Date),
            "name" => Ok(Self::Name),
            "inode" => Ok(Self::INode),
            "links" => Ok(Self::Links),
            "git" => Ok(Self::GitStatus),
            _ => Err(format!("Not a valid block name: {string}")),
        }
    }
}
