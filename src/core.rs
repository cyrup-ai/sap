use crate::color::Colors;
use crate::display;
use crate::flags::{
    ColorOption, Flags, HyperlinkOption, Layout, Literal, SortOrder, ThemeOption,
};
use crate::icon::Icons;

use crate::meta::Meta;
use crate::{print_output, sort, ExitCode};
use std::path::PathBuf;

#[cfg(not(target_os = "windows"))]
use std::io;
#[cfg(not(target_os = "windows"))]
use std::os::unix::io::AsRawFd;

use crate::git_theme::GitTheme;
#[cfg(target_os = "windows")]
use terminal_size::terminal_size;

pub struct Core {
    flags: Flags,
    icons: Icons,
    colors: Colors,
    git_theme: GitTheme,
    sorters: Vec<(SortOrder, sort::SortFn)>,
}

impl Core {
    pub fn new(mut flags: Flags) -> Self {
        // Check through libc if stdout is a tty. Unix specific so not on windows.
        // Determine color output availability (and initialize color output (for Windows 10))
        #[cfg(not(target_os = "windows"))]
        let tty_available = unsafe { libc::isatty(io::stdout().as_raw_fd()) == 1 };

        #[cfg(not(target_os = "windows"))]
        let console_color_ok = true;

        #[cfg(target_os = "windows")]
        let tty_available = terminal_size().is_some(); // terminal_size allows us to know if the stdout is a tty or not.

        #[cfg(target_os = "windows")]
        let console_color_ok = crossterm::ansi_support::supports_ansi();

        let color_theme = match (tty_available && console_color_ok, flags.color.when) {
            (_, ColorOption::Never) | (false, ColorOption::Auto) => ThemeOption::NoColor,
            _ => flags.color.theme.clone(),
        };

        let icon_when = flags.icons.when;
        let icon_theme = flags.icons.theme.clone();

        // TODO: Rework this so that flags passed downstream does not
        // have Auto option for any (icon, color, hyperlink).
        if matches!(flags.hyperlink, HyperlinkOption::Auto) {
            flags.hyperlink = if tty_available {
                HyperlinkOption::Always
            } else {
                HyperlinkOption::Never
            }
        }

        let icon_separator = flags.icons.separator.0.clone();

        // The output is not a tty, this means the command is piped. e.g.
        //
        // lsd -l | less
        //
        // Most of the programs does not handle correctly the ansi colors
        // or require a raw output (like the `wc` command).
        if !tty_available {
            // we should not overwrite the tree layout
            if flags.layout != Layout::Tree {
                flags.layout = Layout::OneLine;
            }

            flags.literal = Literal(true);
        };

        let sorters = sort::assemble_sorters(&flags);

        Self {
            flags,
            colors: Colors::new(color_theme),
            icons: Icons::new(tty_available, icon_when, icon_theme, icon_separator),
            git_theme: GitTheme::new(),
            sorters,
        }
    }

    pub async fn run(self, paths: Vec<PathBuf>) -> ExitCode {
        // Determine traversal depth based on flags (copied from fetch() logic)
        let depth = match self.flags.layout {
            Layout::Tree => self.flags.recursion.depth,
            _ if self.flags.recursion.enabled => self.flags.recursion.depth,
            _ => 1,
        };

        // Build streaming pipeline
        let file_stream = crate::stream::FileStream::new(
            paths.clone(),
            depth,
            &self.flags.ignore_globs,
            self.flags.display,
        );

        // Route to appropriate output mode
        if self.flags.llm.is_enabled() {
            self.display_llm_stream(file_stream).await
        } else if self.flags.layout == Layout::Tree {
            self.display_tree_stream(file_stream, &paths).await
        } else {
            // Grid/OneLine modes: buffer temporarily (can optimize with GridAccumulator later)
            self.display_buffered(file_stream).await
        }
    }

    async fn display_llm_stream(
        &self,
        file_stream: crate::stream::FileStream,
    ) -> ExitCode {
        use futures::StreamExt;
        use crate::stream::AggregatedChatStream;

        let chat_stream = AggregatedChatStream::new(
            file_stream,
            self.flags.llm.objective.clone(),
            self.flags.llm.current_task.clone(),
        );

        let mut stream = Box::pin(chat_stream);
        let mut exit_code = ExitCode::OK;

        // If objective/task provided, could use FileSystemAgent here
        // For now, just output JSONL directly
        while let Some(result) = stream.next().await {
            match result {
                Ok(json_line) => println!("{}", json_line),
                Err(e) => {
                    eprintln!("Stream error: {}", e);
                    exit_code.set_if_greater(ExitCode::MinorIssue);
                }
            }
        }

        exit_code
    }

    async fn display_tree_stream(
        &self,
        file_stream: crate::stream::FileStream,
        _paths: &[PathBuf],
    ) -> ExitCode {
        use futures::StreamExt;
        use std::collections::HashMap;

        // Buffer all entries and organize hierarchically
        let mut entries = Vec::new();
        let mut exit_code = ExitCode::OK;

        let mut stream = Box::pin(file_stream);
        while let Some(result) = stream.next().await {
            match result {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    eprintln!("Stream error: {}", e);
                    exit_code.set_if_greater(ExitCode::MinorIssue);
                }
            }
        }

        // Sort by depth descending so we process deepest children first
        // This ensures children have their descendants before being cloned to parents
        entries.sort_by(|a, b| b.depth.cmp(&a.depth));

        // Convert entries to Meta and build hierarchy
        let mut meta_map: HashMap<PathBuf, Meta> = HashMap::new();
        for entry in &entries {
            let meta = entry.to_meta(self.flags.permission);
            meta_map.insert(entry.path.clone(), meta);
        }

        // Build tree structure by attaching children to parents
        // First pass: identify which paths have parents in the map
        let mut child_paths = std::collections::HashSet::new();
        for entry in &entries {
            if let Some(parent_path) = entry.path.parent() {
                if meta_map.contains_key(parent_path) {
                    child_paths.insert(entry.path.clone());
                }
            }
        }

        // Second pass: build parent-child relationships
        for entry in &entries {
            if let Some(parent_path) = entry.path.parent() {
                if child_paths.contains(&entry.path) {
                    let child_meta = meta_map.get(&entry.path).unwrap().clone();
                    if let Some(parent_meta) = meta_map.get_mut(parent_path) {
                        if parent_meta.content.is_none() {
                            parent_meta.content = Some(Vec::new());
                        }
                        if let Some(content) = &mut parent_meta.content {
                            content.push(child_meta);
                        }
                    }
                }
            }
        }

        // Third pass: collect root metas (those not in child_paths)
        let mut root_metas = Vec::new();
        for entry in &entries {
            if !child_paths.contains(&entry.path) {
                if let Some(meta) = meta_map.get(&entry.path) {
                    root_metas.push(meta.clone());
                }
            }
        }

        // Sort root metas
        self.sort(&mut root_metas);

        // Display using existing tree display logic
        let output = display::tree(
            &root_metas,
            &self.flags,
            &self.colors,
            &self.icons,
            &self.git_theme,
        );

        print_output!("{}", output);
        exit_code
    }

    async fn display_buffered(
        &self,
        file_stream: crate::stream::FileStream,
    ) -> ExitCode {
        use futures::StreamExt;

        // Buffer entries from stream
        let mut entries = Vec::new();
        let mut exit_code = ExitCode::OK;

        let mut stream = Box::pin(file_stream);
        while let Some(result) = stream.next().await {
            match result {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    eprintln!("Stream error: {}", e);
                    exit_code.set_if_greater(ExitCode::MinorIssue);
                }
            }
        }

        // Convert FileEntry to Meta
        let mut metas: Vec<Meta> = entries
            .iter()
            .map(|entry| entry.to_meta(self.flags.permission))
            .collect();

        // Sort using configured sorters
        self.sort(&mut metas);

        // Display using existing grid/oneline display logic
        let output = display::grid(
            &metas,
            &self.flags,
            &self.colors,
            &self.icons,
            &self.git_theme,
        );

        print_output!("{}", output);
        exit_code
    }



    fn sort(&self, metas: &mut Vec<Meta>) {
        metas.sort_unstable_by(|a, b| sort::by_meta(&self.sorters, a, b));

        for meta in metas {
            if let Some(ref mut content) = meta.content {
                self.sort(content);
            }
        }
    }




}
