use std::path::PathBuf;
use std::process::Command;

fn bin_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // remove test binary name
    path.pop(); // remove "deps"
    path.push("uncoauthor");
    path
}

struct TestRepo {
    dir: PathBuf,
}

impl TestRepo {
    fn new(name: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("uncoauthor-test-{}-{}", name, std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).unwrap();
        }
        std::fs::create_dir_all(&dir).unwrap();

        let repo = TestRepo { dir };
        repo.git(&["init"]);
        repo.git(&["config", "user.name", "Test"]);
        repo.git(&["config", "user.email", "test@test.com"]);
        repo
    }

    fn git(&self, args: &[&str]) -> std::process::Output {
        Command::new("git")
            .args(args)
            .current_dir(&self.dir)
            .output()
            .unwrap()
    }

    fn commit(&self, msg: &str) {
        let file = self.dir.join("file.txt");
        let content = if file.exists() {
            let mut c = std::fs::read_to_string(&file).unwrap();
            c.push_str("more\n");
            c
        } else {
            "init\n".to_string()
        };
        std::fs::write(&file, content).unwrap();
        self.git(&["add", "file.txt"]);
        self.git(&["commit", "-m", msg]);
    }

    fn run_tool(&self, args: &[&str]) -> std::process::Output {
        Command::new(bin_path())
            .args(args)
            .current_dir(&self.dir)
            .output()
            .unwrap()
    }

    fn log_messages(&self) -> String {
        let out = self.git(&["log", "--format=%B", "--reverse"]);
        String::from_utf8(out.stdout).unwrap()
    }
}

impl Drop for TestRepo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn test_strips_coauthors() {
    let repo = TestRepo::new("strips");

    // Initial commit (will be the base)
    repo.commit("initial commit");

    // Tag the base
    repo.git(&["tag", "base"]);

    // Commits with co-author trailers
    repo.commit("feat: add feature\n\nCo-authored-by: Alice <alice@example.com>");
    repo.commit("fix: bug fix\n\nCo-authored-by: Bob <bob@example.com>");
    repo.commit("docs: update readme");

    let output = repo.run_tool(&["base"]);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(
        output.status.success(),
        "tool should succeed, stderr: {stderr}"
    );
    assert!(
        stdout.contains("Rewrote 2 commits"),
        "should report 2 commits rewritten, got: {stdout}"
    );

    // Verify no co-author lines remain
    let log = repo.log_messages();
    assert!(
        !log.to_lowercase().contains("co-authored-by"),
        "co-author lines should be stripped, log:\n{log}"
    );
}

#[test]
fn test_no_coauthors() {
    let repo = TestRepo::new("no-coauthors");

    repo.commit("initial commit");
    repo.git(&["tag", "base"]);
    repo.commit("feat: clean commit");
    repo.commit("fix: another clean commit");

    let output = repo.run_tool(&["base"]);
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    assert!(
        stdout.contains("Rewrote 0 commits"),
        "should report 0 commits rewritten, got: {stdout}"
    );
}

#[test]
fn test_invalid_ref() {
    let repo = TestRepo::new("invalid-ref");
    repo.commit("initial commit");

    let output = repo.run_tool(&["nonexistent-ref"]);
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr.contains("invalid base ref"),
        "should mention invalid ref, got: {stderr}"
    );
}

#[test]
fn test_dirty_tree() {
    let repo = TestRepo::new("dirty");
    repo.commit("initial commit");
    repo.git(&["tag", "base"]);
    repo.commit("second commit");

    // Dirty the tree
    std::fs::write(repo.dir.join("file.txt"), "dirty").unwrap();

    let output = repo.run_tool(&["base"]);
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr.contains("not clean"),
        "should mention dirty tree, got: {stderr}"
    );
}

#[test]
fn test_no_commits_in_range() {
    let repo = TestRepo::new("no-range");
    repo.commit("initial commit");

    let output = repo.run_tool(&["HEAD"]);
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr.contains("no commits"),
        "should mention no commits, got: {stderr}"
    );
}

