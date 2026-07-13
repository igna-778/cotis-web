use std::env;
use std::ffi::{CStr, CString, c_char};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::slice;
use std::sync::OnceLock;

use directories::ProjectDirs;
use fs_extra::dir::{CopyOptions, copy};
use log::{debug, error};
use serde::{Deserialize, Serialize};

pub(crate) static PROJ_DIRS: OnceLock<ProjectDirs> = OnceLock::new();

const GIT_REPO: &str = "https://github.com/igna-778/cotis-web.git";

#[derive(Debug)]
pub enum WebCotisError {
    Io(std::io::Error),
    WasmPackFailed,
    CopyFailed(fs_extra::error::Error),
    GitCloneFailed,
    GitPullFailed,
    InvalidPkgJson(serde_json::Error),
    MissingWasmGlueMain,
}

impl From<std::io::Error> for WebCotisError {
    fn from(err: std::io::Error) -> Self {
        WebCotisError::Io(err)
    }
}

#[derive(Debug, Clone)]
pub struct WebCotisOptions {
    pub output: PathBuf,
    pub features: Vec<String>,
}

impl Default for WebCotisOptions {
    fn default() -> Self {
        Self {
            output: PathBuf::from("./target/cotis_web"),
            features: Vec::new(),
        }
    }
}

pub fn execute_web_build(opts: WebCotisOptions) -> Result<(), WebCotisError> {
    init_proj_dirs();
    fetch_canvas_resources()?;

    debug!("Building web wasm to {}", opts.output.display());

    // wasm-pack build --target web -d <output>/pkg
    let mut command = Command::new("wasm-pack");
    command.args(["build", "--target", "web"]);
    command.stdout(Stdio::inherit());
    command.stderr(Stdio::inherit());
    command.args(["-d", opts.output.join("pkg").to_str().unwrap()]);
    if !opts.features.is_empty() {
        command.arg("--features");
        command.arg(opts.features.join(","));
    }

    let status = command.status()?;
    if !status.success() {
        error!("wasm-pack failed: {status}");
        return Err(WebCotisError::WasmPackFailed);
    }

    copy_resources(&opts.output)?;
    sync_wasm_snippet_renderer(&opts.output)?;
    patch_index_wasm_import_path(&opts.output)?;
    Ok(())
}

/// wasm-pack writes `pkg/package.json` with a `main` entry (Rust lib name, e.g. `web_example.js`).
const WASM_IMPORT_PATH_PLACEHOLDER: &str = "__COTIS_WEB_WASM_IMPORT_PATH__";

/// Bundled with the crate so `index.html` is available without a local monorepo or git cache layout.
const DEFAULT_ON_ROOT_INDEX_HTML: &str = include_str!("../resources/on-root/index.html");

#[derive(Deserialize)]
struct WasmPkgJson {
    main: String,
}

fn wasm_import_path_from_pkg(output: &Path) -> Result<String, WebCotisError> {
    let pkg_json = output.join("pkg/package.json");
    let raw = fs::read_to_string(&pkg_json)?;
    let meta: WasmPkgJson = serde_json::from_str(&raw).map_err(WebCotisError::InvalidPkgJson)?;
    if meta.main.is_empty() {
        return Err(WebCotisError::MissingWasmGlueMain);
    }
    Ok(format!("./pkg/{}", meta.main))
}

fn patch_index_wasm_import_path(output: &Path) -> Result<(), WebCotisError> {
    let index_path = output.join("index.html");
    if !index_path.exists() {
        return Ok(());
    }
    let mut html = fs::read_to_string(&index_path)?;
    if !html.contains(WASM_IMPORT_PATH_PLACEHOLDER) {
        return Ok(());
    }
    let path = wasm_import_path_from_pkg(output)?;
    html = html.replace(WASM_IMPORT_PATH_PLACEHOLDER, &path);
    fs::write(&index_path, html)?;
    Ok(())
}

fn init_proj_dirs() {
    if PROJ_DIRS.get().is_some() {
        return;
    }
    if let Some(proj_dirs) = ProjectDirs::from("", "cotis-web", "cotis-web-builder") {
        let _ = PROJ_DIRS.set(proj_dirs);
    }
}

fn fetch_canvas_resources() -> Result<(), WebCotisError> {
    let original_dir = env::current_dir()?;
    let cotis_web_root = PROJ_DIRS
        .get()
        .expect("ProjectDirs not initialized")
        .cache_dir()
        .to_path_buf();
    let repo_path_str = cotis_web_root
        .join("cotis-web")
        .to_str()
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "cache path is not UTF-8")
        })?
        .to_string();

    let repo_path = PathBuf::from(&repo_path_str);
    if !repo_path.exists() {
        fs::create_dir_all(&cotis_web_root)?;
        env::set_current_dir(&cotis_web_root)?;

        let out = Command::new("git")
            .args(["clone", GIT_REPO])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;
        if !out.success() {
            env::set_current_dir(&original_dir)?;
            return Err(WebCotisError::GitCloneFailed);
        }
    }

    // Always force-sync the cache to latest origin default branch.
    let fetch_status = Command::new("git")
        .args(["-C", &repo_path_str, "fetch", "--prune", "origin"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if !fetch_status.success() {
        env::set_current_dir(&original_dir)?;
        return Err(WebCotisError::GitPullFailed);
    }

    let head_ref = Command::new("git")
        .args([
            "-C",
            &repo_path_str,
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()?;
    if !head_ref.status.success() {
        env::set_current_dir(&original_dir)?;
        return Err(WebCotisError::GitPullFailed);
    }
    let head_ref = String::from_utf8_lossy(&head_ref.stdout).trim().to_string();
    let reset_status = Command::new("git")
        .args(["-C", &repo_path_str, "reset", "--hard", &head_ref])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if !reset_status.success() {
        env::set_current_dir(&original_dir)?;
        return Err(WebCotisError::GitPullFailed);
    }

    let clean_status = Command::new("git")
        .args(["-C", &repo_path_str, "clean", "-fd"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if !clean_status.success() {
        env::set_current_dir(&original_dir)?;
        return Err(WebCotisError::GitPullFailed);
    }
    env::set_current_dir(&original_dir)?;
    Ok(())
}

fn copy_resources(output: &Path) -> Result<(), WebCotisError> {
    let cache_root = PROJ_DIRS
        .get()
        .expect("ProjectDirs not initialized")
        .cache_dir()
        .to_path_buf();
    let candidate_roots = [
        PathBuf::from("../cotis-web/resources"),
        PathBuf::from("./cotis-web/resources"),
        cache_root.join("cotis-web/cotis-web/resources"),
        cache_root.join("cotis_web/cotis-web/resources"),
        cache_root.join("cotis_web/resources"),
    ];
    let resources_root = candidate_roots
        .iter()
        .find(|p| p.is_dir())
        .cloned()
        .unwrap_or_else(|| candidate_roots[0].clone());
    let on_root = resources_root.join("on-root");
    let web_renderer = resources_root.join("web-renderer");
    let local = Path::new("./web_resources");
    if on_root.is_dir() {
        let mut options = CopyOptions::new();
        options.copy_inside = true;
        options.content_only = true;
        options.overwrite = true;
        copy(&on_root, output, &options).map_err(WebCotisError::CopyFailed)?;
    } else {
        fs::create_dir_all(output)?;
        fs::write(output.join("index.html"), DEFAULT_ON_ROOT_INDEX_HTML)?;
    }
    if web_renderer.is_dir() {
        let mut options = CopyOptions::new();
        options.copy_inside = true;
        options.content_only = true;
        options.overwrite = true;
        let output_web_renderer = output.join("web-renderer");
        fs::create_dir_all(&output_web_renderer)?;
        copy(&web_renderer, &output_web_renderer, &options).map_err(WebCotisError::CopyFailed)?;
    }

    if local.exists() {
        let mut options = CopyOptions::new();
        options.copy_inside = true;
        options.content_only = true;
        options.overwrite = true;
        copy(local, output, &options).map_err(WebCotisError::CopyFailed)?;
    }
    Ok(())
}

fn sync_wasm_snippet_renderer(output: &Path) -> Result<(), WebCotisError> {
    let cache_root = PROJ_DIRS
        .get()
        .expect("ProjectDirs not initialized")
        .cache_dir()
        .to_path_buf();
    let candidate_roots = [
        PathBuf::from("../cotis-web/resources"),
        PathBuf::from("./cotis-web/resources"),
        cache_root.join("cotis-web/cotis-web/resources"),
        cache_root.join("cotis_web/cotis-web/resources"),
        cache_root.join("cotis_web/resources"),
    ];
    let resources_root = candidate_roots
        .iter()
        .find(|p| p.is_dir())
        .cloned()
        .unwrap_or_else(|| candidate_roots[0].clone());
    let cache_renderer = resources_root.join("web-renderer/renderer.js");
    let snippets_root = output.join("pkg/snippets");
    let mut snippet_renderer_files = Vec::new();
    collect_snippet_renderer_files(&snippets_root, &mut snippet_renderer_files)?;
    if !cache_renderer.exists() || snippet_renderer_files.is_empty() {
        return Ok(());
    }
    let content = fs::read(&cache_renderer)?;
    for target in snippet_renderer_files {
        fs::write(&target, &content)?;
    }
    Ok(())
}

fn collect_snippet_renderer_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), WebCotisError> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_snippet_renderer_files(&path, out)?;
            continue;
        }
        let path_string = path.to_string_lossy().replace('\\', "/");
        if path_string.ends_with("resources/web-renderer/renderer.js") {
            out.push(path);
        }
    }
    Ok(())
}

// -------------------------------------------------------------------------------------------------
// Plugin ABI exports
// -------------------------------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn cotis_plugin_api_version() -> u32 {
    1
}

#[derive(Serialize)]
struct Descriptor<'a> {
    name: &'a str,
    version: &'a str,
}

#[unsafe(no_mangle)]
pub extern "C" fn cotis_plugin_descriptor_json() -> *mut c_char {
    let d = Descriptor {
        name: "cotis-web-builder",
        version: "0.1.0-alpha",
    };
    let json = serde_json::to_string(&d).unwrap();
    CString::new(json).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn cotis_plugin_help() -> *mut c_char {
    let msg = "Web Cotis Builder Routine\n\
               Builds the Web/WASM package via wasm-pack.\n\
               Args (plugin/standalone): --output <path> --features <f1,f2,...>\n";
    CString::new(msg).unwrap().into_raw()
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn cotis_plugin_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(ptr));
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn cotis_plugin_run(argc: i32, argv: *const *const c_char) -> i32 {
    let mut args = Vec::new();
    unsafe {
        if !argv.is_null() {
            let slice = slice::from_raw_parts(argv, argc as usize);
            for &arg_ptr in slice {
                if !arg_ptr.is_null()
                    && let Ok(s) = CStr::from_ptr(arg_ptr).to_str()
                {
                    args.push(s.to_string());
                }
            }
        }
    }
    match run_from_args(&args) {
        Ok(()) => 0,
        Err(_) => 1,
    }
}

fn run_from_args(args: &[String]) -> Result<(), WebCotisError> {
    // args[0] is routine name when invoked by cotis-cli; keep parsing tolerant.
    let mut opts = WebCotisOptions::default();
    let mut it = args.iter().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--output" => {
                if let Some(v) = it.next() {
                    opts.output = PathBuf::from(v);
                }
            }
            "--features" => {
                if let Some(v) = it.next() {
                    append_features(&mut opts.features, v);
                }
            }
            _ => {}
        }
    }
    execute_web_build(opts)
}

fn append_features(features: &mut Vec<String>, raw: &str) {
    for feature in raw.split(',') {
        let trimmed = feature.trim().trim_matches('"').trim_matches('\'');
        if trimmed.is_empty() {
            continue;
        }
        if !features.iter().any(|existing| existing == trimmed) {
            features.push(trimmed.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_output_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("cotis-web-builder-test-{unique}"))
    }

    #[test]
    fn patch_index_wasm_import_path_replaces_placeholder() {
        let output = temp_output_dir();
        fs::create_dir_all(output.join("pkg")).unwrap();
        fs::write(
            output.join("pkg/package.json"),
            r#"{"main":"web_example.js"}"#,
        )
        .unwrap();
        fs::write(
            output.join("index.html"),
            format!(
                "import init from '{WASM_IMPORT_PATH_PLACEHOLDER}';\nrun('{WASM_IMPORT_PATH_PLACEHOLDER}');"
            ),
        )
        .unwrap();

        patch_index_wasm_import_path(&output).unwrap();

        let html = fs::read_to_string(output.join("index.html")).unwrap();
        assert!(!html.contains(WASM_IMPORT_PATH_PLACEHOLDER));
        assert!(html.contains("./pkg/web_example.js"));
        let _ = fs::remove_dir_all(&output);
    }

    #[test]
    fn bundled_index_html_uses_same_placeholder() {
        assert!(DEFAULT_ON_ROOT_INDEX_HTML.contains(WASM_IMPORT_PATH_PLACEHOLDER));
    }
}
