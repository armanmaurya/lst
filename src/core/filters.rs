use walkdir::DirEntry;
use std::ffi::OsStr;

/// Check if a directory entry is hidden (starts with '.' but not '.' or '..')
pub fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.') && s != "." && s != "..")
        .unwrap_or(false)
}

/// Filter predicate for walkdir that respects the show_hidden flag
pub fn should_show_entry(entry: &DirEntry, show_hidden: bool) -> bool {
    // Always skip common heavy directories
    if is_common_skip_os(entry.file_name()) {
        return false;
    }
    show_hidden || !is_hidden(entry)
}

pub fn is_common_skip_name(name: &str) -> bool {
    matches!(
        name,

        // Programming languages
        "node_modules" | "target" | ".cargo" | "registry" |
        "go" | "pkg" | "mod" | "gopath" |
        "__pycache__" | "env" | "venv" | ".venv" |

        // Flutter SDK
        "flutter" | "bin" | "dev" | "examples" | "packages" |
        "engine" | "tool" | "web_sdk" |

        // Dart / Flutter iOS junk
        "ios" | "Runner" | "Assets.xcassets" | "Scenarios" |
        "android" | "android_embedding_bundle" |

        // Unity
        "Unity" | "Editor" | "Library" | "PackageCache" |
        "PlaybackEngines" | "Data" | "il2cpp" | "external" |
        "WebGLSupport" | "Emscripten" | "third_party" |

        // Windows
        "scoop" | "apps" | "buckets" |

        // Browsers
        "Tor Browser" | "Browser" | "TorBrowser" |

        // Common install dirs
        "Program Files" | "Programs" | "Adobe" |

        // Build/cache directories
        ".git" | ".hg" | ".svn" |
        ".vscode" | ".idea" | ".cache" |
        "dist" | "build" | "out" | ".next" | ".nuxt" | ".vercel"
    )
}




/// Same check for OsStr names
pub fn is_common_skip_os(name: &OsStr) -> bool {
    name.to_str().map(is_common_skip_name).unwrap_or(false)
}
