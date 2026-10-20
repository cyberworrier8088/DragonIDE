use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct GitFile {
    pub path: String,
    pub full_path: String,
    pub status: String,
    pub staged: bool,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct GitStatus {
    pub branch: String,
    pub ahead: u32,
    pub behind: u32,
    pub files: Vec<GitFile>,
}

fn git(root: &str, args: &[&str]) -> Result<String, String> {
    let output = crate::command("git")
        // a folder's own .git/config can name a program for git to run, so switch that off
        .args(["-c", "core.fsmonitor=false"])
        .args(args)
        .current_dir(root)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_LITERAL_PATHSPECS", "1") // file names like [id].js are not patterns
        .env("GIT_OPTIONAL_LOCKS", "0") // background status must not lock the index
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "Git is not installed. Get it from https://git-scm.com".to_string()
            } else {
                format!("Failed to run git: {}", e)
            }
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        Ok(if stdout.is_empty() { stderr } else { stdout })
    } else if stderr.is_empty() {
        Err(stdout)
    } else {
        Err(stderr)
    }
}

fn status(root: &str) -> Result<Option<GitStatus>, String> {
    let prefix = match git(root, &["rev-parse", "--show-prefix"]) {
        Ok(prefix) => prefix,
        Err(error) if error.contains("not a git repository") => return Ok(None),
        Err(error) => return Err(error),
    };

    let text = git(
        root,
        &[
            "status",
            "--porcelain=v1",
            "--branch",
            "-z",
            "-uall",
            "--",
            ".",
        ],
    )?;
    Ok(Some(parse_status(&text, &prefix, Path::new(root))))
}

fn parse_status(text: &str, prefix: &str, root: &Path) -> GitStatus {
    let mut status = GitStatus {
        branch: String::new(),
        ahead: 0,
        behind: 0,
        files: Vec::new(),
    };

    let mut entries = text.split('\0');

    while let Some(entry) = entries.next() {
        if let Some(head) = entry.strip_prefix("## ") {
            parse_branch(head, &mut status);
            continue;
        }

        if entry.len() < 4 {
            continue;
        }

        let x = entry.as_bytes()[0] as char;
        let y = entry.as_bytes()[1] as char;
        let path = entry[3..].strip_prefix(prefix).unwrap_or(&entry[3..]);

        // a rename or copy is followed by its old name, in the index (x) or work tree (y)
        if matches!(x, 'R' | 'C') || matches!(y, 'R' | 'C') {
            entries.next();
        }

        let mut add = |code: char, staged: bool| {
            status.files.push(GitFile {
                path: path.to_string(),
                full_path: full_path(root, path),
                status: code.to_string(),
                staged,
            })
        };

        match (x, y) {
            ('?', '?') => add('U', false),
            ('!', '!') => {}
            ('U', _) | (_, 'U') | ('A', 'A') | ('D', 'D') => add('!', false),

            _ => {
                if x != ' ' {
                    add(x, true);
                }
                if y != ' ' {
                    add(y, false);
                }
            }
        }
    }
    status
}

fn parse_branch(head: &str, status: &mut GitStatus) {
    let head = head.strip_prefix("No commits yet on ").unwrap_or(head);
    let (name, rest) = head.split_once(' ').unwrap_or((head, ""));
    status.branch = name.split("...").next().unwrap_or(name).to_string();

    for part in rest.trim_matches(['[', ']']).split(", ") {
        if let Some(n) = part.strip_prefix("ahead ") {
            status.ahead = n.parse().unwrap_or(0);
        } else if let Some(n) = part.strip_prefix("behind ") {
            status.behind = n.parse().unwrap_or(0);
        }
    }
}

fn full_path(root: &Path, relative: &str) -> String {
    let mut path = PathBuf::from(root);
    for part in relative.split('/') {
        path.push(part);
    }
    path.to_string_lossy().to_string()
}

fn with_paths<'a>(command: &[&'a str], paths: &'a [String]) -> Vec<&'a str> {
    let mut args = command.to_vec();
    args.push("--");

    if paths.is_empty() {
        args.push("."); // no paths = everything
    } else {
        args.extend(paths.iter().map(String::as_str));
    }
    args
}

async fn in_background<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn git_status(root: String) -> Result<Option<GitStatus>, String> {
    in_background(move || status(&root)).await
}

#[tauri::command]
pub async fn git_stage(root: String, paths: Vec<String>) -> Result<String, String> {
    in_background(move || git(&root, &with_paths(&["add", "-A"], &paths))).await
}

#[tauri::command]
pub async fn git_unstage(root: String, paths: Vec<String>) -> Result<String, String> {
    in_background(move || git(&root, &with_paths(&["reset", "-q"], &paths))).await
}

#[tauri::command]
pub async fn git_commit(root: String, message: String) -> Result<String, String> {
    in_background(move || git(&root, &["commit", "-m", &message])).await
}

#[tauri::command]
pub async fn git_push(root: String) -> Result<String, String> {
    in_background(move || git(&root, &["push"])).await
}

#[tauri::command]
pub async fn git_pull(root: String) -> Result<String, String> {
    in_background(move || git(&root, &["pull", "--ff-only"])).await
}

#[tauri::command]
pub async fn git_init(root: String) -> Result<String, String> {
    in_background(move || git(&root, &["init"])).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_branch_ahead_and_behind() {
        let status = parse_status(
            "## main...origin/main [ahead 2, behind 1]\0",
            "",
            Path::new("/p"),
        );

        assert_eq!(
            (status.branch.as_str(), status.ahead, status.behind),
            ("main", 2, 1)
        );

        let status = parse_status("## No commits yet on dev\0", "", Path::new("/p"));

        assert_eq!(
            (status.branch.as_str(), status.ahead, status.behind),
            ("dev", 0, 0)
        );
    }

    #[test]
    fn reads_staged_unstaged_new_renamed_and_conflict_files() {
        let text = "## main\0M  a.rs\0 M b.rs\0MM c.rs\0?? new.txt\0R  new.rs\0old.rs\0UU both.rs\0 D gone.rs\0";
        let files = parse_status(text, "", Path::new("/p")).files;

        let short: Vec<(&str, &str, bool)> = files
            .iter()
            .map(|f| (f.path.as_str(), f.status.as_str(), f.staged))
            .collect();

        assert_eq!(
            short,
            [
                ("a.rs", "M", true),
                ("b.rs", "M", false),
                ("c.rs", "M", true),
                ("c.rs", "M", false),
                ("new.txt", "U", false),
                ("new.rs", "R", true),
                ("both.rs", "!", false),
                ("gone.rs", "D", false),
            ]
        );
    }

    #[test]
    fn paths_are_relative_to_the_opened_folder() {
        let files =
            parse_status("## main\0 M app/src/main.rs\0", "app/", Path::new("/p/app")).files;
        assert_eq!(files[0].path, "src/main.rs");
        assert_eq!(
            PathBuf::from(&files[0].full_path),
            Path::new("/p/app").join("src").join("main.rs")
        );
    }

    struct TempRepo(PathBuf);
    impl TempRepo {
        fn new(name: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("dragonide_git_{}_{}", name, std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            TempRepo(dir)
        }

        fn root(&self) -> String {
            self.0.to_string_lossy().to_string()
        }

        fn write(&self, relative: &str, text: &str) {
            let path = self.0.join(relative);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }

        fn init(&self) {
            git(&self.root(), &["init", "-q"]).unwrap();
            git(&self.root(), &["config", "user.name", "Test"]).unwrap();
            git(&self.root(), &["config", "user.email", "test@example.com"]).unwrap();
        }

        fn files(&self) -> Vec<(String, String, bool)> {
            status(&self.root())
                .unwrap()
                .unwrap()
                .files
                .into_iter()
                .map(|f| (f.path, f.status, f.staged))
                .collect()
        }
    }

    impl Drop for TempRepo {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn file(path: &str, status: &str, staged: bool) -> (String, String, bool) {
        (path.to_string(), status.to_string(), staged)
    }

    #[test]
    fn stage_unstage_and_commit() {
        let repo = TempRepo::new("flow");
        repo.init();
        repo.write("src/main.rs", "fn main() {}");
        repo.write("notes.txt", "hi");

        assert_eq!(
            repo.files(),
            [
                file("notes.txt", "U", false),
                file("src/main.rs", "U", false)
            ]
        );

        git(
            &repo.root(),
            &with_paths(&["add", "-A"], &["src/main.rs".to_string()]),
        )
        .unwrap();
        assert_eq!(
            repo.files(),
            [
                file("src/main.rs", "A", true),
                file("notes.txt", "U", false)
            ]
        );

        git(&repo.root(), &with_paths(&["reset", "-q"], &[])).unwrap(); // unstage all, before the first commit

        assert_eq!(
            repo.files(),
            [
                file("notes.txt", "U", false),
                file("src/main.rs", "U", false)
            ]
        );

        git(&repo.root(), &with_paths(&["add", "-A"], &[])).unwrap();
        git(&repo.root(), &["commit", "-m", "first \"quoted\" commit"]).unwrap();
        assert!(repo.files().is_empty());
        assert_eq!(
            git(&repo.root(), &["log", "-1", "--format=%s"]).unwrap(),
            "first \"quoted\" commit"
        );

        repo.write("notes.txt", "changed");
        std::fs::remove_file(repo.0.join("src/main.rs")).unwrap();
        assert_eq!(
            repo.files(),
            [
                file("notes.txt", "M", false),
                file("src/main.rs", "D", false)
            ]
        );

        git(
            &repo.root(),
            &with_paths(&["add", "-A"], &["src/main.rs".to_string()]),
        )
        .unwrap();
        assert_eq!(
            repo.files(),
            [
                file("notes.txt", "M", false),
                file("src/main.rs", "D", true)
            ]
        );
    }

    #[test]
    fn opened_subfolder_only_shows_its_own_files() {
        let repo = TempRepo::new("sub");
        repo.init();
        repo.write("app/src/lib.rs", "");
        repo.write("other/readme.txt", "");

        let app = repo.0.join("app").to_string_lossy().to_string();
        let found = status(&app).unwrap().unwrap().files;

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].path, "src/lib.rs");
        assert_eq!(
            PathBuf::from(&found[0].full_path),
            repo.0.join("app").join("src").join("lib.rs")
        );
    }

    #[test]
    fn commit_with_nothing_staged_shows_git_message() {
        let repo = TempRepo::new("empty");
        repo.init();
        let error = git(&repo.root(), &["commit", "-m", "nothing"]).unwrap_err();

        assert!(error.contains("nothing to commit"), "{}", error);
    }

    #[test]
    fn skips_the_old_name_of_a_work_tree_rename() {
        let files = parse_status("## main\0 R new.txt\0old.txt\0", "", Path::new("/p")).files;

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "new.txt");
    }

    #[test]
    fn staging_a_wildcard_file_name_leaves_its_neighbour() {
        let repo = TempRepo::new("wildcard");
        repo.init();
        repo.write("[id].js", "a");
        repo.write("i.js", "a");
        git(&repo.root(), &with_paths(&["add", "-A"], &[])).unwrap();
        git(&repo.root(), &["commit", "-m", "first"]).unwrap();

        repo.write("[id].js", "b");
        repo.write("i.js", "b");
        let only = ["[id].js".to_string()];
        git(&repo.root(), &with_paths(&["add", "-A"], &only)).unwrap();

        assert_eq!(
            repo.files(),
            [file("[id].js", "M", true), file("i.js", "M", false)]
        );
    }

    #[test]
    fn status_does_not_run_programs_named_in_the_repo_config() {
        let repo = TempRepo::new("fsmonitor");
        repo.init();
        repo.write("a.txt", "a");
        git(&repo.root(), &with_paths(&["add", "-A"], &[])).unwrap();
        git(&repo.root(), &["commit", "-m", "first"]).unwrap();
        git(
            &repo.root(),
            &["config", "core.fsmonitor", "echo ran > fsmonitor_ran.txt; echo"],
        )
        .unwrap();

        status(&repo.root()).unwrap();

        assert!(!repo.0.join("fsmonitor_ran.txt").exists());
    }
}
